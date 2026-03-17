use std::io::{self, Read, Result, Seek, SeekFrom, Write};

use num::Zero;

use crate::{Vector2, Vector3};

pub struct SSPMReader<T: Read + Seek> {
    reader: T,
}

pub struct SSPMWriter<T: Write + Seek> {
    writer: T,
}

impl<T: Seek + Read> SSPMReader<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.reader.seek(pos)
    }

    pub fn stream_position(&mut self) -> Result<u64> {
        self.reader.stream_position()
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0] == 0x01)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_string(&mut self) -> Result<String> {
        let buf = self.read_u16()?;
        let mut buffer = vec![0u8; buf as usize];
        self.reader.read_exact(&mut buffer)?;

        let str = String::from_utf8(buffer)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;

        Ok(str)
    }

    pub fn read_newline_string(&mut self) -> Result<String> {
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            let n = self.reader.read(&mut byte)?;

            if n == 0 {
                break;
            } else if byte[0] == b'\n' {
                break;
            }

            buffer.push(byte[0]);
        }

        let str = String::from_utf8(buffer)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;

        Ok(str)
    }

    pub fn read_long_string(&mut self) -> Result<String> {
        let buf = self.read_u32()?;
        let mut buffer = vec![0u8; buf as usize];
        self.reader.read_exact(&mut buffer)?;

        let str = String::from_utf8(buffer);

        match str {
            Ok(s) => Ok(s),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid UTF-8 sequence",
            )),
        }
    }

    pub fn read_sha1(&mut self) -> Result<[u8; 20]> {
        let mut buf = [0u8; 20];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_vec2(&mut self) -> Result<Vector2> {
        let quantum = self.read_bool()?;
        let mut pos = Vector2::zero();

        match quantum {
            true => {
                pos.x = self.read_f32()?;
                pos.y = self.read_f32()?;
            }
            false => {
                pos.x = self.read_u8()? as f32;
                pos.y = self.read_u8()? as f32;
            }
        }

        Ok(pos)
    }

    #[allow(unused)]
    pub fn read_vec3(&mut self) -> Result<Vector3> {
        let quantum = self.read_bool()?;
        let mut pos = Vector3::ZERO;

        match quantum {
            true => {
                pos.x = self.read_f32()?;
                pos.y = self.read_f32()?;
                pos.z = self.read_f32()?;
            }
            false => {
                pos.x = self.read_u8()? as f32;
                pos.y = self.read_u8()? as f32;
                pos.z = self.read_u8()? as f32;
            }
        }

        Ok(pos)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(buf)
    }
}

impl<T: Write + Seek> SSPMWriter<T> {
    pub fn new(writer: T) -> Self {
        Self { writer }
    }

    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        let byte = if value { 0x01 } else { 0x00 };
        self.writer.write_all(&[byte])
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.writer.write_all(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    #[allow(unused)]
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    #[allow(unused)]
    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_string(&mut self, value: &str) -> Result<()> {
        let len = value.len() as u16;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(value.as_bytes())
    }

    #[allow(unused)]
    pub fn write_long_string(&mut self, value: &str) -> Result<()> {
        let len = value.len() as u32;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(value.as_bytes())
    }

    pub fn write_sha1(&mut self, value: &[u8; 20]) -> Result<()> {
        self.writer.write_all(value)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)
    }

    pub fn stream_position(&mut self) -> Result<u64> {
        self.writer.stream_position()
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.writer.seek(pos)
    }
}
