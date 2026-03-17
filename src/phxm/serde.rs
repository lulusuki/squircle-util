use num::Zero;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Cursor, Error, ErrorKind, Read, Result, Seek, Write},
    path::Path,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    map::{DifficultyName, Map, MapFormat, MapInfo, MapMetadata, MapObjects, MapSerde},
    objects::note::Note,
    phxm::{PHXMReader, PHXMWriter},
    types::Vector2,
};

pub struct PHXMSerde;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            difficulty_name: DifficultyName::from_u8(self.difficulty).unwrap_or_default(),
            audio_buf: None,
            cover_buf: None,
            video_buf: None,
            note_count: 0,
            object_count: 0,
            rating: self.rating,
            artist_link: self.artist_link,
            artist_platform: self.artist_platform,
        }
    }

    pub fn from_map(map: Map) -> Result<Self> {
        let mut audio_ext = String::new();

        if let Some(buf) = &map.info.audio_buf {
            if !infer::is_audio(&buf) {
                return Err(Error::new(ErrorKind::InvalidData, "invalid audio data"));
            }

            match infer::get(&buf) {
                Some(d) => audio_ext = d.extension().to_string(),
                None => (),
            }
        }

        Ok(Self {
            id: map.id,
            title: map.info.title,
            mappers: map.info.mappers,
            artist: map.info.artist,
            length: map.info.length,
            difficulty: map.info.difficulty_name.to_u8(),
            difficulty_name: map.info.difficulty_name.get_value(),
            has_audio: map.info.audio_buf.is_some(),
            has_video: map.info.video_buf.is_some(),
            has_cover: map.info.cover_buf.is_some(),
            audio_ext,
            artist_link: map.info.artist_link,
            artist_platform: map.info.artist_platform,
            rating: map.info.rating,
        })
    }
}

impl MapSerde for PHXMSerde {
    fn from_file(path: &Path) -> Result<Map> {
        if path.extension() != Some(OsStr::new("phxm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "extension needs to be .phxm",
            ));
        }

        let reader = File::open(path)?;

        PHXMSerde::parse_phxm(reader)
    }

    fn from_reader<T: Read + Seek>(reader: T) -> Result<Map> {
        PHXMSerde::parse_phxm(reader)
    }

    fn to_file(path: &Path, map: &Map) -> Result<()> {
        if path.extension() != Some(OsStr::new("phxm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "extension needs to be .phxm",
            ));
        }

        let writer = File::create(path)?;
        PHXMSerde::encode_phxm(writer, map)?;
        Ok(Default::default())
    }

    fn to_writer<T: Write + Seek>(writer: T, map: &Map) -> Result<()> {
        PHXMSerde::encode_phxm(writer, map)?;
        Ok(())
    }
}

impl PHXMSerde {
    fn encode_phxm<T: Write + Seek>(writer: T, map: &Map) -> Result<()> {
        let mut writer = ZipWriter::new(writer);

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        writer.start_file("metadata.json", options)?;
        let metadata = PHXMMetadata::from_map(map.clone())?;
        let data = serde_json::to_vec(&metadata)?;
        writer.write(&data)?;

        if let Some(buf) = &map.info.audio_buf {
            let audio_file = format!("audio.{}", metadata.audio_ext);
            writer.start_file(audio_file, options)?;
            writer.write(buf)?;
        }

        if let Some(buf) = &map.info.cover_buf {
            writer.start_file("cover.png", options)?;
            writer.write(buf)?;
        }

        if let Some(buf) = &map.info.video_buf {
            writer.start_file("video.mp4", options)?;
            writer.write(buf)?;
        }

        writer.start_file("objects.phxmo", options)?;

        let object_buf = Cursor::new(Vec::<u8>::new());
        let mut obj_writer = PHXMWriter::new(object_buf);

        obj_writer.write_u32(12)?;
        obj_writer.write_u32(map.info.note_count)?;

        for note in map.objects.notes.iter() {
            let quantum = note.position.x.floor() != note.position.x
                || note.position.y.floor() != note.position.y
                || note.position.x > 1.0
                || note.position.x < -1.0
                || note.position.y > 1.0
                || note.position.y < -1.0;

            obj_writer.write_u32(note.millisecond)?;
            obj_writer.write_bool(quantum)?;

            match quantum {
                true => {
                    obj_writer.write_f32(note.position.x)?;
                    obj_writer.write_f32(note.position.y)?;
                }
                false => {
                    obj_writer.write_u8((note.position.x + 1.0) as u8)?;
                    obj_writer.write_u8((note.position.y + 1.0) as u8)?;
                }
            }
        }

        obj_writer.write_u32(0)?; // timing point count
        obj_writer.write_u32(0)?; // brightness count
        obj_writer.write_u32(0)?; // contrast count
        obj_writer.write_u32(0)?; // saturation count
        obj_writer.write_u32(0)?; // blur count
        obj_writer.write_u32(0)?; // fov count
        obj_writer.write_u32(0)?; // tint count
        obj_writer.write_u32(0)?; // position count
        obj_writer.write_u32(0)?; // rotation count
        obj_writer.write_u32(0)?; // ar factor count
        obj_writer.write_u32(0)?; // text count

        writer.write(&obj_writer.into_inner().into_inner())?;

        Ok(())
    }
    fn parse_phxm<T: Read + Seek>(reader: T) -> Result<Map> {
        let mut archive = ZipArchive::new(reader)?;
        let mut cover_buf: Option<Vec<u8>> = None;
        let mut audio_buf: Option<Vec<u8>> = None;
        let mut video_buf: Option<Vec<u8>> = None;
        let mut object_buf = Vec::new();

        let metadata_file = archive.by_name("metadata.json")?;
        let phxm_meta: PHXMMetadata = serde_json::from_reader(metadata_file)?;
        let mut info = phxm_meta.clone().to_map_info();

        if phxm_meta.has_video {
            let mut cover = Vec::new();
            let mut cover_file = archive.by_name("cover.png")?;
            cover_file.read_to_end(&mut cover)?;
            cover_buf = Some(cover);
        }

        if phxm_meta.has_audio {
            let mut audio = Vec::new();
            let mut audio_file = archive.by_name(&format!("audio.{}", phxm_meta.audio_ext))?;
            audio_file.read_to_end(&mut audio)?;
            audio_buf = Some(audio);
        }

        if phxm_meta.has_video {
            let mut video = Vec::new();
            let mut video_file = archive.by_name("video.mp4")?;
            video_file.read_to_end(&mut video)?;
            video_buf = Some(video);
        }

        info.cover_buf = cover_buf;
        info.audio_buf = audio_buf;
        info.video_buf = video_buf;

        let mut object_file = archive.by_name("objects.phxmo")?;
        object_file.read_to_end(&mut object_buf)?;
        let object_cursor = Cursor::new(object_buf);

        let mut reader = PHXMReader::new(object_cursor);

        let _type_count = reader.read_u32()?;
        let note_count = reader.read_u32()?;

        let mut notes = Vec::<Note>::new();

        for _ in 0..note_count {
            let ms = reader.read_u32()?;
            let quantum = reader.read_bool()?;

            let mut pos = Vector2::zero();

            match quantum {
                true => {
                    pos.x = reader.read_f32()?;
                    pos.y = reader.read_f32()?;
                }
                false => {
                    pos.x = reader.read_u8()? as f32 - 1.0;
                    pos.y = reader.read_u8()? as f32 - 1.0;
                }
            };

            let note = Note {
                millisecond: ms,
                position: pos,
            };
            notes.push(note);
        }

        let metadata = MapMetadata {
            format: MapFormat::PHXM,
        };

        let objects = MapObjects {
            notes,
            ..Default::default()
        };

        Ok(Map {
            id: phxm_meta.id,
            info,
            metadata,
            objects,
        })
    }
}
