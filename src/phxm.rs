use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr, fs::File, io::{self, Result}, path::Path
};
use zip::ZipArchive;

use crate::map::{DifficultyName, Map, MapInfo, MapSerde};

pub struct PXHMSerde;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PHXMMetadata {
    #[serde(rename = "ID")]
    pub id: String,
    pub length: u32,
    pub title: String,
    pub artist: String,
    pub mappers: Vec<String>,
    pub difficulty: u8,
    pub difficulty_name: String,
    pub has_audio: bool,
    pub audio_ext: String,
    pub has_video: bool,
    pub has_cover: bool,
    pub rating: f32,
    pub artist_link: Option<String>,
    pub artist_platform: Option<String>,
}

impl PHXMMetadata {
    #[allow(unused)]
    pub fn to_map_info(self) -> MapInfo {
        let mut _difficulty_name: DifficultyName;

        MapInfo {
            title: self.title,
            mappers: self.mappers,
            artist: self.artist,
            length: self.length,
            difficulty_name: DifficultyName::from_u8(self.difficulty)
                .unwrap_or(DifficultyName::None("N/A".to_string())),
            audio_buf: None,
            cover_buf: None,
            video_buf: None,
            note_count: 0,
            object_count: 0,
            artist_link: self.artist_link,
            artist_platform: self.artist_platform,
        }
    }
}

impl MapSerde for PXHMSerde {
    fn from_file(path: &Path) -> Result<Map> {
        if path.extension() != Some(OsStr::new("phxm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "Extension needs to be .phxm",
            ));
        }

        let reader = File::open(path)?;
        let mut archive = ZipArchive::new(reader)?;

        let metadata_file = archive.by_name("metadata.json")?;

        let _metadata: PHXMMetadata = serde_json::from_reader(metadata_file)?;
        Ok(Default::default())
    }

    #[allow(unused)]
    fn to_file(path: &Path, map: &Map) -> Result<()> {
        if path.extension() != Some(OsStr::new("phxm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "Extension needs to be .phxm",
            ));
        }

        let writer = File::create(path)?;
        Ok(Default::default())
    }
}
