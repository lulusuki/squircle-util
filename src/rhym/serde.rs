use std::io::{Error, ErrorKind, Result};

use crate::objects::ObjectType;

impl ObjectType {
    pub fn from_rhym(value: u8) -> Result<ObjectType> {
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
            0x0B => Ok(ObjectType::LongString(None)),
            0x0C => Ok(ObjectType::Buf(None)),
            0x0D => Ok(ObjectType::LongBuf(None)),
            0x0E => Ok(ObjectType::Vec(None)),
            0x0F => Ok(ObjectType::LongVec(None)),
            0x10 => Ok(ObjectType::HashMap(None)),
            0x11 => Ok(ObjectType::LongHashMap(None)),
            0x12 => Ok(ObjectType::Tuple(None)),
            _ => Err(Error::new(ErrorKind::InvalidData, "invalid object type")),
        }
    }
}
