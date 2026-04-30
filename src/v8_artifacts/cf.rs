use std::io::{self, Read};

#[derive(Debug, Clone, Copy)]
pub struct ImageHeader {
    pub signature: u32,
    pub page_size: u32,
    pub revision: u32,
    pub unused: u32,
}

pub fn read_image_header<R: Read>(reader: &mut R) -> io::Result<ImageHeader> {
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

pub fn read_page_header<R: Read>(reader: &mut R) -> io::Result<PageHeader> {
    let mut buf = [0u8; 31];
    reader.read_exact(&mut buf)?;

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
