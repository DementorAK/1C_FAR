use std::io::{self, Write};
use std::collections::HashMap;
use crate::base::deflate::deflate;

pub struct ContainerWriter {
    pub page_size: u32,
    pub is_64bit: bool,
    pub revision: u32,
    pub use_triplets: bool, // True for root EPF/CF, False for nested containers
    pub pad_pt_to_page: bool, // True for root EPF/CF to keep PT as full page
}

impl ContainerWriter {
    pub fn new(page_size: u32, is_64bit: bool) -> Self {
        Self { page_size, is_64bit, revision: 0, use_triplets: false, pad_pt_to_page: false }
    }

    pub fn write<W: Write, F: FnMut(usize, usize)>(&self, writer: &mut W, rows: &HashMap<String, (Vec<u8>, bool)>, mut progress: Option<F>) -> io::Result<()> {
        let mut docs = Vec::new();
        let total_rows = rows.len();
        for (i, (id, (data, is_packed))) in rows.iter().enumerate() {
            if let Some(ref mut cb) = progress {
                cb(i, total_rows);
            }
            let wide_id = crate::far::api::to_wide(id);
            let id_bytes: Vec<u8> = wide_id.iter()
                .flat_map(|&u| u.to_le_bytes().to_vec())
                .collect();
            let mut header = vec![0u8; 20];
            header.extend_from_slice(&id_bytes);

            // Align header data to 4 bytes (standard for 1C containers)
            let current_len = header.len();
            let aligned_len = (current_len + 3) & !3;
            if aligned_len > current_len {
                header.extend(vec![0u8; aligned_len - current_len]);
            }

            let body = if *is_packed {
                deflate(data)?
            } else {
                data.clone()
            };

            docs.push((header, body));
        }

        docs.sort_by(|a, b| {
            // Sort by UTF-16 code units in the header (starting at offset 20)
            let id_a_u16: Vec<u16> = a.0[20..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            let id_b_u16: Vec<u16> = b.0[20..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            id_a_u16.cmp(&id_b_u16)
        });

        let img_header_size = if self.is_64bit { 20u32 } else { 16u32 };
        let page_header_size = if self.is_64bit { 55u32 } else { 31u32 };
        let pointer_size = if self.is_64bit { 8 } else { 4 };
        let end_marker = if self.is_64bit { u64::MAX } else { crate::v8_artifacts::container::SIG as u64 };

        let pt_ptrs_per_row = if self.use_triplets { 3 } else { 2 };
        let mut pt_data_size = (docs.len() * pt_ptrs_per_row) as u32 * pointer_size;
        if self.pad_pt_to_page {
            pt_data_size = pt_data_size.max(self.page_size);
        }
        let mut next_doc_offset = img_header_size + page_header_size + pt_data_size;
        let mut pointers = Vec::new();
        let mut row_data_blobs = Vec::new();

        for (h, b) in &docs {
            let mut row_blob = Vec::new();
            let row_start = next_doc_offset;
            
            pointers.push(row_start as u64);
            let mut h_offset = row_start;
            let h_pages = self.serialize_document_into_pages(h, &mut h_offset, end_marker, None);
            for p in h_pages { row_blob.extend(p); }

            let body_start = h_offset;
            pointers.push(body_start as u64);
            let mut b_offset = body_start;
            let b_pages = self.serialize_document_into_pages(b, &mut b_offset, end_marker, Some(self.page_size));
            for p in b_pages { row_blob.extend(p); }

            if self.use_triplets {
                // Add third "SIG" pointer per row (required for root EPF/CF files)
                pointers.push(end_marker);
            }

            row_data_blobs.push(row_blob);
            next_doc_offset = row_start + row_data_blobs.last().unwrap().len() as u32;
        }
        // No need for an extra end_marker if the last row's triplet ends with one.
        // pointers.push(end_marker);

        let mut pointers_raw = Vec::with_capacity(pointers.len() * pointer_size as usize);
        for p in pointers {
            if self.is_64bit { pointers_raw.extend_from_slice(&p.to_le_bytes()); }
            else { pointers_raw.extend_from_slice(&(p as u32).to_le_bytes()); }
        }
        let mut pt_offset = img_header_size;
        let pt_force_size = if self.pad_pt_to_page { Some(self.page_size) } else { None };
        // Pointers table should force full page_size alignment only for root containers.
        let pt_pages = self.serialize_document_into_pages(&pointers_raw, &mut pt_offset, end_marker, pt_force_size);

        if self.is_64bit {
            writer.write_all(&crate::v8_artifacts::container::SIG64.to_le_bytes())?;
            writer.write_all(&self.page_size.to_le_bytes())?;
            writer.write_all(&self.revision.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
        } else {
            writer.write_all(&crate::v8_artifacts::container::SIG.to_le_bytes())?;
            writer.write_all(&self.page_size.to_le_bytes())?;
            writer.write_all(&self.revision.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
        }

        for p in pt_pages { writer.write_all(&p)?; }
        for blob in row_data_blobs { writer.write_all(&blob)?; }

        Ok(())
    }

    fn serialize_document_into_pages(&self, content: &[u8], current_offset: &mut u32, end_marker: u64, force_page_size: Option<u32>) -> Vec<Vec<u8>> {
        let mut pages = Vec::new();
        let full_size = content.len() as u64;
        let page_header_size = if self.is_64bit { 55u32 } else { 31u32 };
        let mut remaining = full_size;
        let mut current_data_offset = 0;

        if full_size == 0 {
            let page = self.format_page_header(0, 0, end_marker);
            *current_offset += page_header_size;
            return vec![page];
        }

        while remaining > 0 {
            let chunk_size = std::cmp::min(remaining, self.page_size as u64);
            let is_last = remaining <= self.page_size as u64;
            let next_page_abs = if is_last {
                end_marker
            } else {
                (*current_offset + page_header_size + chunk_size as u32) as u64
            };
            
            let target_chunk_size = if let Some(target_size) = force_page_size {
                target_size as u64
            } else {
                // Align every data chunk to 4 bytes for platform compatibility.
                // This doesn't modify the data itself (full_size stays chunk_size),
                // but ensures the next page starts at a 4-aligned boundary.
                (chunk_size + 3) & !3
            };

            let hdr_page_size = target_chunk_size;
            let mut page = self.format_page_header(chunk_size, hdr_page_size, next_page_abs);
            page.extend_from_slice(&content[current_data_offset..current_data_offset + chunk_size as usize]);
            
            if hdr_page_size > chunk_size {
                page.extend(vec![0u8; (hdr_page_size - chunk_size) as usize]);
            }
            
            pages.push(page);
            *current_offset += page_header_size + hdr_page_size as u32;
            remaining -= chunk_size;
            current_data_offset += chunk_size as usize;
        }
        pages
    }

    fn format_page_header(&self, full_size: u64, page_size: u64, next_page: u64) -> Vec<u8> {
        if self.is_64bit {
            format!("\r\n{:016x} {:016x} {:016x} \r\n", full_size, page_size, next_page).into_bytes()
        } else {
            format!("\r\n{:08x} {:08x} {:08x} \r\n", full_size as u32, page_size as u32, next_page as u32).into_bytes()
        }
    }
}
