use std::io::{self, Result};
use std::path::Path;
use crate::map::{Map, MapSet, MapSerde, MapSetSerde};
use crate::objects::{ObjectType};

pub struct NYAZSerde;

pub struct NYAMSerde;

impl ObjectType {
    pub fn from_nyam(value: u8) -> io::Result<ObjectType> {
        match value {
            0x00 => Ok(ObjectType::U8(None)),
            0x01 => Ok(ObjectType::I16(None)),
            0x02 => Ok(ObjectType::U16(None)),
            0x03 => Ok(ObjectType::I32(None)),
            0x04 => Ok(ObjectType::U32(None)),
            0x05 => Ok(ObjectType::I64(None)),
            0x06 => Ok(ObjectType::U64(None)),
            0x07 => Ok(ObjectType::F32(None)),
            0x08 => Ok(ObjectType::F64(None)),
            0x09 => Ok(ObjectType::Bool(None)),
            0x0A => Ok(ObjectType::String(None)),
            0x0B => Ok(ObjectType::Buf(None)),
            0x0C => Ok(ObjectType::LongBuf(None)),
            0x0D => Ok(ObjectType::Vec(None)),
            0x0E => Ok(ObjectType::LongVec(None)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid object type",
            )),
        }
    }
}

impl MapSetSerde for NYAZSerde {
    #[allow(unused)]
    fn from_file(path: &Path) -> Result<MapSet> {
        todo!()
    }

    #[allow(unused)]
    fn to_file(path: &Path) -> Result<()> {
        todo!()
    }
}

impl MapSerde for NYAMSerde {
    #[allow(unused)]
    fn from_file(path: &Path) -> Result<Map> {
        todo!()
    }

    #[allow(unused)]
    fn to_file(path: &Path, map: &Map) -> Result<()> {
        todo!()
    }
}
