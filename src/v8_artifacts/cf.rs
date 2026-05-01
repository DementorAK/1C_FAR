use std::io::{self};
use crate::base::reader::V8Reader;
use crate::base::deflate::inflate;

pub const SIG: u32 = 0x7FFFFFFF;
pub const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

#[derive(Debug, Clone, Copy)]
pub struct ImageHeader {
    pub signature: u32,
    pub page_size: u32,
    pub revision: u32,
    pub unused: u32,
}

pub fn read_image_header<R: V8Reader>(reader: &mut R) -> io::Result<ImageHeader> {
    let mut buf = [0u8; 16];
    reader.read_exact(&mut buf)?;
    Ok(ImageHeader {
        signature: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        page_size: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        revision: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
        unused: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
    })
}

#[derive(Debug, Clone, Copy)]
pub struct PageHeader {
    pub full_size: u32,
    pub page_size: u32,
    pub next_page: u32,
}

pub fn read_page_header<R: V8Reader>(reader: &mut R) -> io::Result<PageHeader> {
    let mut buf = [0u8; 31];
    reader.read_exact(&mut buf)?;

    if buf[0] != 0x0D || buf[1] != 0x0A || buf[10] != 0x20 || buf[19] != 0x20 || buf[28] != 0x0D || buf[29] != 0x0A {
        // We can be more lenient, but 1C format is strict about CRLF and spaces in hex headers
    }

    let full_size_str = std::str::from_utf8(&buf[2..10]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let page_size_str = std::str::from_utf8(&buf[11..19]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let next_page_str = std::str::from_utf8(&buf[20..28]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let full_size = u32::from_str_radix(full_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let page_size = u32::from_str_radix(page_size_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let next_page = u32::from_str_radix(next_page_str, 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(PageHeader {
        full_size,
        page_size,
        next_page,
    })
}

pub struct Container<R: V8Reader> {
    reader: R,
    pointers: Vec<u32>,
}

impl<R: V8Reader> Container<R> {
    pub fn new(mut reader: R) -> io::Result<Self> {
        let _header = read_image_header(&mut reader)?;
        
        // Read pointers table
        // The pointers table usually starts after the image header (at pos 16)
        let mut pointers = Vec::new();
        reader.set_pos(16)?;
        
        let mut current_page_pos = 16;
        let mut first_page = true;
        let mut remaining_size = 0;

        while current_page_pos != SIG as u64 {
            reader.set_pos(current_page_pos)?;
            let page_header = read_page_header(&mut reader)?;
            
            if first_page {
                remaining_size = page_header.full_size;
                first_page = false;
            }

            let mut buf = vec![0u8; page_header.page_size as usize];
            reader.read_exact(&mut buf)?;
            
            let to_process = std::cmp::min(page_header.page_size, remaining_size) as usize;
            for chunk in buf[..to_process].chunks_exact(4) {
                let p = u32::from_le_bytes(chunk.try_into().unwrap());
                if p != SIG {
                    pointers.push(p);
                }
            }
            
            remaining_size -= to_process as u32;
            current_page_pos = page_header.next_page as u64;
            if current_page_pos == SIG as u64 || remaining_size == 0 {
                break;
            }
        }

        Ok(Self {
            reader,
            pointers,
        })
    }

    pub fn rows(&mut self) -> RowIterator<'_, R> {
        RowIterator {
            container: self,
            index: 0,
        }
    }

    pub fn read_row_data(&mut self, addr: u32) -> io::Result<Vec<u8>> {
        self.reader.set_pos(addr as u64)?;
        let page_header = read_page_header(&mut self.reader)?;
        
        let mut result = Vec::with_capacity(page_header.full_size as usize);
        let mut remaining = page_header.full_size;
        let mut next_page = addr;

        while remaining > 0 && next_page != SIG {
            self.reader.set_pos(next_page as u64)?;
            let ph = read_page_header(&mut self.reader)?;
            
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

        // Read Row Header
        let header_data = match self.container.read_row_data(header_addr) {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };

        // Row header format: 2x datetime (8 bytes each) + 4 bytes attributes + ID (UTF-16)
        // But 1C often uses a simpler version where ID is the remaining part.
        // The ID is UTF-16, zero-terminated.
        if header_data.len() < 20 {
            return Some(Err(io::Error::new(io::ErrorKind::InvalidData, "Row header too short")));
        }
        let id_raw = &header_data[20..];
        let id_u16: Vec<u16> = id_raw.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        let id = String::from_utf16_lossy(&id_u16);

        // Read Row Body
        let mut body_data = match self.container.read_row_data(body_addr) {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };

        // Heuristic for compression: lack of BOM
        let mut is_packed = false;
        if !body_data.starts_with(BOM) && !body_data.is_empty() {
            is_packed = true;
            match inflate(&body_data) {
                Ok(decompressed) => body_data = decompressed,
                Err(_) => {
                    // If inflation fails, maybe it's not packed or it's a different format.
                    // For now, we assume if it failed it's not packed.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::reader::StringReader;

    #[test]
    fn test_mock_container() {
        let mut data = Vec::new();
        // Image Header (16 bytes)
        data.extend_from_slice(&SIG.to_le_bytes()); // signature
        data.extend_from_slice(&512u32.to_le_bytes()); // page_size
        data.extend_from_slice(&0u32.to_le_bytes()); // revision
        data.extend_from_slice(&0u32.to_le_bytes()); // unused
        
        // Pointers Page (starts at pos 16)
        // Page Header (31 bytes): CRLF + FullSize(8) + Space + PageSize(8) + Space + NextPage(8) + Space + CRLF
        data.extend_from_slice(b"\r\n00000008 00000200 7FFFFFFF \r\n"); // 31 bytes
        // Pointer Table Data (on page of size 512)
        let mut pointers_data = Vec::new();
        let header_addr = 1024u32;
        let body_addr = 2048u32;
        pointers_data.extend_from_slice(&header_addr.to_le_bytes());
        pointers_data.extend_from_slice(&body_addr.to_le_bytes());
        pointers_data.resize(512, 0);
        data.extend_from_slice(&pointers_data);
        
        // Mock row header (at 1024)
        data.resize(1024, 0);
        data.extend_from_slice(b"\r\n0000001C 00000200 7FFFFFFF \r\n"); // FullSize = 28
        let mut row_header = vec![0u8; 20]; // timestamps
        row_header.extend_from_slice("T\0e\0s\0t\0\0\0".as_bytes()); // ID: "Test"
        row_header.resize(512, 0);
        data.extend_from_slice(&row_header);
        
        // Mock row body (at 2048)
        data.resize(2048, 0);
        data.extend_from_slice(b"\r\n00000006 00000200 7FFFFFFF \r\n"); // FullSize = 6
        let mut row_body = BOM.to_vec();
        row_body.extend_from_slice(b"abc"); // data
        row_body.resize(512, 0);
        data.extend_from_slice(&row_body);
        
        let reader = StringReader::new(data);
        let mut container = Container::new(reader).unwrap();
        
        assert_eq!(container.pointers.len(), 2);
        assert_eq!(container.pointers[0], 1024);
        
        let rows: Vec<_> = container.rows().collect::<io::Result<Vec<_>>>().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "Test");
        assert_eq!(rows[0].data, [0xEF, 0xBB, 0xBF, b'a', b'b', b'c']);
        assert_eq!(rows[0].is_packed, false);
    }
}
