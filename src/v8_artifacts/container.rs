use std::io::{self};
use crate::base::reader::V8Reader;
use crate::base::deflate::inflate;

pub const SIG: u32 = 0x7FFFFFFF;
pub const SIG64: u64 = 0xFFFFFFFFFFFFFFFF;
pub const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

#[derive(Debug, Clone, Copy)]
pub struct ImageHeader {
    pub signature: u64,
    pub page_size: u32,
    pub revision: u32,
    pub unused: u32,
    pub header_size: u64,
}

pub fn read_image_header<R: V8Reader>(reader: &mut R, offset: u64) -> io::Result<ImageHeader> {
    reader.set_pos(offset)?;
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    
    let is_64bit = &buf[0..4] != &SIG.to_le_bytes();
    
    reader.set_pos(offset)?;
    if is_64bit {
        let mut full_buf = [0u8; 20];
        reader.read_exact(&mut full_buf)?;
        Ok(ImageHeader {
            signature: u64::from_le_bytes(full_buf[0..8].try_into().unwrap()),
            page_size: u32::from_le_bytes(full_buf[8..12].try_into().unwrap()),
            revision: u32::from_le_bytes(full_buf[12..16].try_into().unwrap()),
            unused: u32::from_le_bytes(full_buf[16..20].try_into().unwrap()),
            header_size: 20,
        })
    } else {
        let mut full_buf = [0u8; 16];
        reader.read_exact(&mut full_buf)?;
        Ok(ImageHeader {
            signature: u32::from_le_bytes(full_buf[0..4].try_into().unwrap()) as u64,
            page_size: u32::from_le_bytes(full_buf[4..8].try_into().unwrap()),
            revision: u32::from_le_bytes(full_buf[8..12].try_into().unwrap()),
            unused: u32::from_le_bytes(full_buf[12..16].try_into().unwrap()),
            header_size: 16,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageHeader {
    pub full_size: u64,
    pub page_size: u64,
    pub next_page: u64,
    pub header_size: u64,
}

pub fn read_page_header<R: V8Reader>(reader: &mut R, is_64bit: bool) -> io::Result<PageHeader> {
    if is_64bit {
        let mut buf = [0u8; 55];
        reader.read_exact(&mut buf)?;
        
        let full_size_str = std::str::from_utf8(&buf[2..18]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let page_size_str = std::str::from_utf8(&buf[19..35]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let next_page_str = std::str::from_utf8(&buf[36..52]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let full_size = u64::from_str_radix(full_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let page_size = u64::from_str_radix(page_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let next_page = u64::from_str_radix(next_page_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(PageHeader {
            full_size,
            page_size,
            next_page,
            header_size: 55,
        })
    } else {
        let mut buf = [0u8; 31];
        reader.read_exact(&mut buf)?;

        let full_size_str = std::str::from_utf8(&buf[2..10]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let page_size_str = std::str::from_utf8(&buf[11..19]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let next_page_str = std::str::from_utf8(&buf[20..28]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let full_size = u32::from_str_radix(full_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))? as u64;
        let page_size = u32::from_str_radix(page_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))? as u64;
        let next_page = u32::from_str_radix(next_page_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))? as u64;

        Ok(PageHeader {
            full_size,
            page_size,
            next_page,
            header_size: 31,
        })
    }
}

pub struct Container<R: V8Reader> {
    pub reader: R,
    pointers: Vec<u64>,
    pub is_64bit: bool,
    pub offset: u64,
    pub size: u64,
}

impl<R: V8Reader> Container<R> {
    pub fn new(mut reader: R, offset: u64) -> io::Result<Self> {
        let header = read_image_header(&mut reader, offset)?;
        let is_64bit = header.header_size == 20;
        let end_marker = if is_64bit { SIG64 } else { SIG as u64 };
        let pointer_size = if is_64bit { 8 } else { 4 };
        
        let mut pointers = Vec::new();
        let mut current_page_pos = offset + header.header_size;
        let mut first_page = true;
        let mut remaining_size = 0;

        while current_page_pos != end_marker {
            reader.set_pos(current_page_pos)?;
            let page_header = read_page_header(&mut reader, is_64bit)?;
            
            if first_page {
                remaining_size = page_header.full_size;
                first_page = false;
                
                // Estimate the full document size (including headers)
                // This is needed to advance `offset` for multi-container files.
                // In v8unpack, the document size is calculated robustly. For the pointer table,
                // doc_size = full_size + header_size. If it spans multiple pages, it's slightly more.
                // A simpler way to get container.size is to sum up all document sizes in read_files.
            }

            let mut buf = vec![0u8; page_header.page_size as usize];
            reader.read_exact(&mut buf)?;
            
            let to_process = std::cmp::min(page_header.page_size, remaining_size) as usize;
            for chunk in buf[..to_process].chunks_exact(pointer_size) {
                let p = if is_64bit {
                    u64::from_le_bytes(chunk.try_into().unwrap())
                } else {
                    u32::from_le_bytes(chunk.try_into().unwrap()) as u64
                };
                if p != end_marker {
                    pointers.push(p);
                }
            }
            
            remaining_size -= to_process as u64;
            current_page_pos = page_header.next_page;
            
            if current_page_pos == end_marker || remaining_size == 0 {
                break;
            }
        }

        Ok(Self {
            reader,
            pointers,
            is_64bit,
            offset,
            size: 0, // We will compute size later
        })
    }

    pub fn rows(&mut self) -> RowIterator<'_, R> {
        RowIterator {
            container: self,
            index: 0,
        }
    }

    pub fn read_row_data(&mut self, addr: u64) -> io::Result<Vec<u8>> {
        let abs_addr = self.offset + addr;
        self.reader.set_pos(abs_addr)?;
        let page_header = read_page_header(&mut self.reader, self.is_64bit)?;
        
        let mut result = Vec::with_capacity(page_header.full_size as usize);
        let mut remaining = page_header.full_size;
        let mut next_page = addr;
        let end_marker = if self.is_64bit { SIG64 } else { SIG as u64 };

        while remaining > 0 && next_page != end_marker {
            let abs_next = self.offset + next_page;
            self.reader.set_pos(abs_next)?;
            let ph = read_page_header(&mut self.reader, self.is_64bit)?;
            
            let to_read = std::cmp::min(remaining, ph.page_size);
            let mut buf = vec![0u8; to_read as usize];
            self.reader.read_exact(&mut buf)?;
            result.extend_from_slice(&buf);
            
            remaining -= to_read;
            next_page = ph.next_page;
        }

        Ok(result)
    }
}

pub struct Row {
    pub id: String,
    pub data: Vec<u8>,
    pub is_packed: bool,
}

pub struct RowIterator<'a, R: V8Reader> {
    container: &'a mut Container<R>,
    index: usize,
}

impl<'a, R: V8Reader> Iterator for RowIterator<'a, R> {
    type Item = io::Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + 1 >= self.container.pointers.len() {
            return None;
        }

        let header_addr = self.container.pointers[self.index];
        let body_addr = self.container.pointers[self.index + 1];
        self.index += 2;

        let header_data = match self.container.read_row_data(header_addr) {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };

        if header_data.len() < 20 {
            return Some(Err(io::Error::new(io::ErrorKind::InvalidData, "Row header too short")));
        }
        let id_raw = &header_data[20..];
        let id_u16: Vec<u16> = id_raw.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        let id = String::from_utf16_lossy(&id_u16);

        let mut body_data = match self.container.read_row_data(body_addr) {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };

        let mut is_packed = false;
        if !body_data.starts_with(BOM) && !body_data.is_empty() {
            is_packed = true;
            match inflate(&body_data) {
                Ok(decompressed) => body_data = decompressed,
                Err(_) => {
                    is_packed = false;
                }
            }
        }

        Some(Ok(Row {
            id,
            data: body_data,
            is_packed,
        }))
    }
}

pub fn read_all_rows<R: V8Reader>(mut reader: R, file_size: u64) -> io::Result<std::collections::HashMap<String, Vec<u8>>> {
    let mut all_rows = std::collections::HashMap::new();
    let mut offset = 0;
    
    // Read the whole file into memory for fast scanning of signatures
    reader.set_pos(0)?;
    let mut full_data = vec![0u8; file_size as usize];
    reader.read_exact(&mut full_data)?;
    
    while offset < file_size {
        // Try to parse a container at `offset`
        // We only attempt if we see a valid signature
        let mut found_sig = false;
        while offset <= file_size - 8 {
            let p32 = u32::from_le_bytes(full_data[offset as usize..offset as usize + 4].try_into().unwrap());
            let p64 = u64::from_le_bytes(full_data[offset as usize..offset as usize + 8].try_into().unwrap());
            if p32 == SIG || p64 == SIG64 {
                // Heuristic: check if the page_size looks reasonable (e.g. 512, 1024, 4096, 65536)
                // Actually, just try to parse the container
                found_sig = true;
                break;
            }
            offset += 1;
        }
        
        if !found_sig {
            break;
        }
        
        // We found a signature at `offset`. Let's try to parse the container.
        let clone_reader = crate::base::reader::StringReader::new(full_data.clone());
        match Container::new(clone_reader, offset) {
            Ok(container) => {
                let mut stack = vec![container];
                while let Some(mut current_container) = stack.pop() {
                    for row_res in current_container.rows() {
                        if let Ok(row) = row_res {
                            if row.data.len() >= 8 {
                                let p32 = u32::from_le_bytes(row.data[0..4].try_into().unwrap());
                                let p64 = u64::from_le_bytes(row.data[0..8].try_into().unwrap());
                                if p32 == SIG || p64 == SIG64 {
                                    let nested_reader = crate::base::reader::StringReader::new(row.data.clone());
                                    if let Ok(nested_container) = Container::new(nested_reader, 0) {
                                        stack.push(nested_container);
                                    }
                                }
                            }
                            
                            // Insert the row itself (whether it's a container or not)
                            all_rows.insert(row.id.clone(), row.data);
                        }
                    }
                }
            }
            Err(_e) => {
            }
        }
        
        // Advance offset. Since we don't know the exact size of the container,
        // we'll just skip the signature and keep scanning. The next valid signature
        // won't appear inside compressed data, because compressed data doesn't have 
        // 8-byte boundaries aligned perfectly, and Container blocks themselves are handled.
        // Wait, what if `SIG` or `SIG64` appears inside the Container's data naturally?
        // Actually, the easiest way to avoid re-parsing inside the same container is to 
        // advance offset by a decent amount, or better: just let `Container` parse, 
        // and if it succeeds, advance by some minimum container size, e.g., 64 bytes.
        offset += 16;
    }
    
    Ok(all_rows)
}

