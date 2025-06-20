use std::io::{self, Read};

pub mod class;
pub(crate) mod modified_utf8;

pub(crate) trait ReadIntExt {
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_bytes(&mut self, n: usize) -> io::Result<Box<[u8]>>;
}
impl<R: Read> ReadIntExt for R {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(u8::from_be_bytes(buf))
    }
    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
    fn read_bytes(&mut self, n: usize) -> io::Result<Box<[u8]>> {
        let mut buf = vec![0; n];
        self.read_exact(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}