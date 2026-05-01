use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;

/// Abstract positional reader trait for 1C artifact processing.
pub trait V8Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn set_pos(&mut self, pos: u64) -> io::Result<u64>;
    fn pos(&mut self) -> io::Result<u64>;
    fn size(&mut self) -> io::Result<u64>;

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.read(&mut buf[offset..])?;
            if n == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Failed to fill whole buffer"));
            }
            offset += n;
        }
        Ok(())
    }
}

/// A reader that works with an in-memory buffer.
pub struct StringReader {
    data: Vec<u8>,
    cursor: usize,
}

impl StringReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }
}

impl V8Reader for StringReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = (&self.data[self.cursor..]).read(buf)?;
        self.cursor += n;
        Ok(n)
    }

    fn set_pos(&mut self, pos: u64) -> io::Result<u64> {
        self.cursor = pos as usize;
        Ok(pos)
    }

    fn pos(&mut self) -> io::Result<u64> {
        Ok(self.cursor as u64)
    }

    fn size(&mut self) -> io::Result<u64> {
        Ok(self.data.len() as u64)
    }
}

/// A reader that works with a file on disk.
/// TODO: Implement buffering as described in the spec (2.2) if performance is an issue.
pub struct FileReader {
    file: File,
    size: u64,
}

impl FileReader {
    pub fn new(mut file: File) -> io::Result<Self> {
        let size = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;
        Ok(Self { file, size })
    }
}

impl V8Reader for FileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }

    fn set_pos(&mut self, pos: u64) -> io::Result<u64> {
        self.file.seek(SeekFrom::Start(pos))
    }

    fn pos(&mut self) -> io::Result<u64> {
        self.file.stream_position()
    }

    fn size(&mut self) -> io::Result<u64> {
        Ok(self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_reader() {
        let data = vec![1, 2, 3, 4, 5];
        let mut reader = StringReader::new(data);
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [1, 2]);
        assert_eq!(reader.pos().unwrap(), 2);
        
        reader.set_pos(4).unwrap();
        let mut buf2 = [0u8; 1];
        reader.read_exact(&mut buf2).unwrap();
        assert_eq!(buf2, [5]);
        assert_eq!(reader.size().unwrap(), 5);
    }
}
