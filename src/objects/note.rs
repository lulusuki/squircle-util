use std::io::{self, Result};

use crate::Vector2;
use crate::objects::{HitObject, ObjectDefinition, ObjectType, TimelineObject};

#[derive(Debug, Default)]
pub struct Note {
    pub millisecond: u32,
    pub position: Vector2,
}

impl TimelineObject for Note {
    fn get_millisecond(&self) -> u32 {
        self.millisecond
    }

    fn from_sspm_definition(definition: ObjectDefinition) -> Result<Self> {
        if definition.name.as_str() != "ssp_note" || definition.data.len() < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid note params",
            ));
        }

        let pos = match definition.data[0] {
            ObjectType::Vec2(Some(value)) => value,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid note params",
                ));
            }
        };

        Ok(Note {
            millisecond: definition.millisecond,
            position: pos,
        })
    }
}

impl HitObject for Note {
    fn get_position(&self) -> &Vector2 {
        &self.position
    }
}
