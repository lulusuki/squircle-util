use std::{
    io::{self, Read, Result, Seek, Write},
    path::Path,
};

use crate::{
    objects::{ObjectDefinition, note::Note},
    phxm::PHXMSerde,
    sspm::SSPMSerde,
};

#[derive(Debug, Default, Clone)]
pub enum MapFormat {
    PHXM,
    #[default]
    SSPM,
}

#[derive(Debug, Default)]
pub struct MapSet {
    pub id: String,
    pub maps: Vec<Map>,
}

#[derive(Debug, Default, Clone)]
pub struct Map {
    pub id: String,
    pub info: MapInfo,
    pub metadata: MapMetadata,
    pub objects: MapObjects,
}

#[derive(Debug, Default)]
pub struct PartialMap {
    pub id: String,
    pub info: MapInfo,
    pub metadata: MapMetadata,
}

#[derive(Debug, Default, Clone)]
pub struct MapInfo {
    pub title: String,
    pub mappers: Vec<String>,
    pub artist: String,
    pub length: u32,
    pub difficulty_name: DifficultyName,
    pub audio_buf: Option<Vec<u8>>,
    pub cover_buf: Option<Vec<u8>>,
    pub video_buf: Option<Vec<u8>>,
    pub note_count: u32,
    pub object_count: u32,
    pub rating: f32,
    pub artist_link: Option<String>,
    pub artist_platform: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct MapMetadata {
    pub format: MapFormat,
}

#[derive(Debug, Clone)]
pub enum DifficultyName {
    None(String),
    Easy(String),
    Medium(String),
    Hard(String),
    Expert(String),
    Insane(String),
    Illogical(String),
}

impl Default for DifficultyName {
    fn default() -> Self {
        DifficultyName::None(String::from("N/A"))
    }
}

impl DifficultyName {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(DifficultyName::None("N/A".to_string())),
            1 => Some(DifficultyName::Easy("Easy".to_string())),
            2 => Some(DifficultyName::Medium("Medium".to_string())),
            3 => Some(DifficultyName::Hard("Hard".to_string())),
            4 => Some(DifficultyName::Expert("Expert".to_string())),
            5 => Some(DifficultyName::Insane("Insane".to_string())),
            6 => Some(DifficultyName::Illogical("Illogical".to_string())),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            DifficultyName::None(_) => 0,
            DifficultyName::Easy(_) => 1,
            DifficultyName::Medium(_) => 2,
            DifficultyName::Hard(_) => 3,
            DifficultyName::Expert(_) => 4,
            DifficultyName::Insane(_) => 5,
            DifficultyName::Illogical(_) => 6,
        }
    }

    pub fn get_value(&self) -> String {
        match self {
            DifficultyName::None(str) => str.clone(),
            DifficultyName::Easy(str) => str.clone(),
            DifficultyName::Medium(str) => str.clone(),
            DifficultyName::Hard(str) => str.clone(),
            DifficultyName::Expert(str) => str.clone(),
            DifficultyName::Insane(str) => str.clone(),
            DifficultyName::Illogical(str) => str.clone(),
        }
    }

    pub fn is_default(&self) -> bool {
        match self {
            DifficultyName::None(str) => str == "N/A",
            DifficultyName::Easy(str) => str == "Easy",
            DifficultyName::Medium(str) => str == "Medium",
            DifficultyName::Hard(str) => str == "Hard",
            DifficultyName::Expert(str) => str == "Expert",
            DifficultyName::Insane(str) => str == "Insane",
            DifficultyName::Illogical(str) => str == "Illogical",
        }
    }

    pub fn set_value(&mut self, value: String) {
        match self {
            DifficultyName::None(str)
            | DifficultyName::Easy(str)
            | DifficultyName::Medium(str)
            | DifficultyName::Hard(str)
            | DifficultyName::Expert(str)
            | DifficultyName::Insane(str)
            | DifficultyName::Illogical(str) => {
                *str = value;
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MapObjects {
    pub notes: Vec<Note>,
    pub undefined: Vec<ObjectDefinition>,
}

pub trait MapSetSerde {
    fn from_file(path: &Path) -> Result<MapSet>;

    fn from_reader<T: Read + Seek>(reader: T) -> Result<MapSet>;

    fn to_file(path: &Path) -> Result<()>;
}

pub trait MapSerde {
    fn from_file(path: &Path) -> Result<Map>;

    fn from_reader<T: Read + Seek>(reader: T) -> Result<Map>;

    fn to_file(path: &Path, map: &Map) -> Result<()>;

    fn to_writer<T: Write + Seek>(writer: T, map: &Map) -> Result<()>;
}

pub trait ObjectParser {
    fn from_definition(definition: ObjectDefinition) -> Result<Self>
    where
        Self: Sized;
}

impl Map {
    pub fn from_file(path: &Path) -> Result<Map> {
        let err = io::Error::new(
            io::ErrorKind::InvalidFilename,
            "could not get valid file extension",
        );

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap_or("");
            return match ext {
                "sspm" => Map::from_sspm_file(path),
                "phxm" => Map::from_phxm_file(path),
                _ => Err(err),
            };
        }

        Err(err)
    }

    pub fn from_reader<T: Read + Seek, S>(reader: T) -> Result<Map>
    where
        S: MapSerde,
    {
        S::from_reader(reader)
    }

    pub fn to_file<S>(path: &Path, map: &Map) -> Result<()>
    where
        S: MapSerde,
    {
        S::to_file(path, map)?;
        Ok(())
    }

    pub fn to_writer<T: Write + Seek, S>(writer: T, map: &Map) -> Result<()>
    where
        S: MapSerde,
    {
        S::to_writer(writer, map)?;
        Ok(())
    }

    fn from_phxm_file(path: &Path) -> Result<Self> {
        PHXMSerde::from_file(path)
    }

    fn from_sspm_file(path: &Path) -> Result<Self> {
        SSPMSerde::from_file(path)
    }
}
