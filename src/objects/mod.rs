use std::io::Result;

use crate::{Vector2, Vector3};

pub mod note;

#[derive(Debug)]
pub enum ObjectType {
    U8(Option<u8>),
    I16(Option<i16>),
    U16(Option<u16>),
    I32(Option<i32>),
    U32(Option<u32>),
    I64(Option<i64>),
    U64(Option<u64>),
    F32(Option<f32>),
    F64(Option<f64>),
    Bool(Option<bool>),
    Vec2(Option<Vector2>),
    Buf(Option<Vec<u8>>),
    String(Option<String>),
    LongBuf(Option<Vec<u8>>),
    LongString(Option<String>),
    Vec3(Option<Vector3>),
    Vec(Option<Vec<ObjectType>>),
    LongVec(Option<Vec<ObjectType>>),
}

#[derive(Debug)]
pub enum QEMZObjectType {}

#[derive(Debug)]
pub struct ObjectDefinition {
    pub name: String,
    pub millisecond: u32,
    pub data: Vec<ObjectType>,
}

pub trait TimelineObject {
    fn get_millisecond(&self) -> u32;

    fn from_sspm_definition(definition: ObjectDefinition) -> Result<Self>
    where
        Self: Sized;
}

pub trait HitObject {
    fn get_position(&self) -> &Vector2;
}

#[allow(unused)]
trait Position {
    fn get_position(&self) -> Vector3;
}
