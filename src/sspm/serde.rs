use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::usize;

use num::Zero;

use super::{SSPMReader, SSPMWriter};
use crate::Vector2;
use crate::map::{DifficultyName, Map, MapInfo, MapMetadata, MapObjects, MapSerde};
use crate::objects::note::Note;
use crate::objects::{ObjectDefinition, ObjectType, TimelineObject};

pub struct SSPMSerde;

fn round_to_places(value: f32, places: u32) -> f32 {
    let factor = 10f32.powi(places as i32);
    (value * factor).round() / factor
}

impl ObjectType {
    pub fn from_sspm(value: u8) -> io::Result<ObjectType> {
        match value {
            0x01 => Ok(ObjectType::U8(None)),
            0x02 => Ok(ObjectType::U16(None)),
            0x03 => Ok(ObjectType::U32(None)),
            0x04 => Ok(ObjectType::U64(None)),
            0x05 => Ok(ObjectType::F32(None)),
            0x06 => Ok(ObjectType::F64(None)),
            0x07 => Ok(ObjectType::Vec2(None)),
            0x08 => Ok(ObjectType::Buf(None)),
            0x09 => Ok(ObjectType::String(None)),
            0x0A => Ok(ObjectType::LongBuf(None)),
            0x0B => Ok(ObjectType::LongString(None)),
            0x0C => Ok(ObjectType::Vec(None)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid object type",
            )),
        }
    }
}

impl MapSerde for SSPMSerde {
    fn to_file(path: &Path, map: &Map) -> Result<()> {
        if path.extension() != Some(OsStr::new("sspm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "Extension needs to be .sspm",
            ));
        }

        let writer = File::create(path)?;
        SSPMSerde::write_sspm(writer, map)?;
        Ok(())
    }

    fn to_writer<T: Write + Seek>(writer: T, map: &Map) -> Result<()> {
        SSPMSerde::write_sspm(writer, map)?;
        Ok(())
    }

    fn from_file(path: &Path) -> io::Result<Map> {
        if path.extension() != Some(OsStr::new("sspm")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "extension needs to be .sspm",
            ));
        }

        let reader = File::open(path)?;

        Self::read_sspm(SSPMReader::new(reader))
    }

    fn from_reader<T: Read + Seek>(reader: T) -> Result<Map> {
        Self::read_sspm(SSPMReader::new(reader))
    }
}

impl SSPMSerde {
    fn write_sspm<T: Write + Seek>(writer: T, map: &Map) -> Result<()> {
        let mut writer = SSPMWriter::new(writer);

        // Header
        writer.write_all(b"SS+m")?; // File signature
        writer.write_all(&[0x02, 0x00])?; // Version 2
        writer.write_all(&[0u8; 4])?; // Unused bytes

        // Static Metadata
        writer.write_sha1(&[0u8; 20])?; // SHA1 is never used yet so ignore for now
        writer.write_u32(map.info.length)?;
        writer.write_u32(map.info.note_count)?;
        writer.write_u32(map.info.object_count)?;

        writer.write_u8(map.info.difficulty_name.to_u8())?;
        writer.write_u16(0)?; // Star rating is never used
        writer.write_bool(map.info.audio_buf.is_some())?;
        writer.write_bool(map.info.cover_buf.is_some())?;
        writer.write_bool(false)?;

        let data_offset = writer.stream_position()?;
        writer.write_all(&[0u8; 80])?; // Placeholder for data offsets and lengths

        writer.write_string(&map.id)?;
        writer.write_string(&map.info.title)?;
        writer.write_string(&map.info.title)?; // Song name is the same as title for now

        writer.write_u16(map.info.mappers.len() as u16)?;
        for mapper in map.info.mappers.iter() {
            writer.write_string(mapper)?;
        }

        let mut custom_data_offset: u64 = 0;
        let mut custom_data_length: u64 = 0;

        if !map.info.difficulty_name.is_default() {
            custom_data_offset = writer.stream_position()?;

            writer.write_u16(1)?; // One custom data field
            writer.write_string("difficulty_name")?;
            writer.write_u8(0x09)?; // String type
            writer.write_string(&map.info.difficulty_name.get_value())?;

            custom_data_length = writer.stream_position()? - custom_data_offset;
        } else {
            writer.write_u16(0)?; // zero custom data fields
        }

        let audio_offset = writer.stream_position()?;
        if let Some(audio) = &map.info.audio_buf {
            writer.write_all(&audio)?;
        }
        let audio_length = writer.stream_position()? - audio_offset;

        let mut cover_offset = 0;
        let mut cover_length = 0;

        if let Some(cover) = &map.info.cover_buf {
            cover_offset = writer.stream_position()?;
            writer.write_all(&cover)?;
            cover_length = writer.stream_position()? - cover_offset;
        }

        let object_definition_offset = writer.stream_position()?;
        writer.write_u8(1)?;
        writer.write_string("ssp_note")?;
        writer.write_all(&[0x01, 0x07, 0x00])?; // One definition of type Vec2
        let object_definition_length = writer.stream_position()? - object_definition_offset;

        let object_data_offset = writer.stream_position()?;

        for note in map.objects.notes.iter() {
            writer.write_u32(note.millisecond)?;
            writer.write_u8(0x00)?;

            let quantum = note.position.x.round() != round_to_places(note.position.x, 3)
                || note.position.y.round() != round_to_places(note.position.y, 3);

            writer.write_bool(quantum)?;

            let x = note.position.x + 1.0;
            let y = -note.position.y + 1.0;

            if quantum {
                writer.write_f32(x)?;
                writer.write_f32(y)?;
            } else {
                writer.write_u8(x as u8)?;
                writer.write_u8(y as u8)?;
            }
        }

        let object_data_length = writer.stream_position()? - object_data_offset;

        writer.seek(SeekFrom::Start(data_offset))?;
        writer.write_u64(custom_data_offset)?;
        writer.write_u64(custom_data_length)?;
        writer.write_u64(audio_offset)?;
        writer.write_u64(audio_length)?;
        writer.write_u64(cover_offset)?;
        writer.write_u64(cover_length)?;
        writer.write_u64(object_definition_offset)?;
        writer.write_u64(object_definition_length)?;
        writer.write_u64(object_data_offset)?;
        writer.write_u64(object_data_length)?;

        writer.seek(SeekFrom::End(0))?;
        writer.write_string(format!("Squircle Util - {}", env!("CARGO_PKG_VERSION")).as_str())?;

        Ok(())
    }

    fn read_sspm<T: Read + Seek>(mut reader: SSPMReader<T>) -> Result<Map> {
        // Header structure:
        // The first 4 bytes are the file signature "SS+m"
        // The next 2 bytes are the version of the sspm (currently only version 2 is supported)
        // The rest of the header is unused
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;

        // Header signature must be "SS+m"
        if header[0..4] != *b"SS+m" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "incorrect file signature",
            ));
        }

        match header[4..6] {
            [0x01, 0x00] => SSPMSerde::read_sspm_v1(reader),
            [0x02, 0x00] => SSPMSerde::read_sspm_v2(reader),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported SSPM version",
            )),
        }
    }

    fn read_sspm_v1<T: Read + Seek>(mut reader: SSPMReader<T>) -> Result<Map> {
        let map_id = reader.read_newline_string()?; // Id of the map
        let map_name = reader.read_newline_string()?; // Name of the map
        let mappers = reader.read_newline_string()?; // Mappers
        let length = reader.read_u32()?; // Last object millisecond
        let note_count = reader.read_u32()?; // Note object count
        let difficulty = reader.read_u8()?; // Difficulty of the map
        let difficulty_name = DifficultyName::from_u8(difficulty).unwrap_or_default();

        let mut cover_buf: Option<Vec<u8>> = None;
        let mut audio_buf: Option<Vec<u8>> = None;

        if reader.read_u8()? == 0x02 {
            let cover_data_length = reader.read_u64()?;
            let mut buf = vec![0u8; cover_data_length as usize];
            reader.read_exact(&mut buf)?;
            cover_buf = Some(buf);
        }

        if reader.read_u8()? == 0x01 {
            let audio_data_length = reader.read_u64()?;
            let mut buf = vec![0u8; audio_data_length as usize];
            reader.read_exact(&mut buf)?;
            audio_buf = Some(buf);
        }

        let mut notes = Vec::<Note>::new();

        for _ in 0..note_count {
            let millisecond = reader.read_u32()?;
            let mut position = reader.read_vec2()?;

            position.x -= 1.0;
            position.y = -position.y + 1.0;

            notes.push(Note {
                millisecond,
                position,
            });
        }

        let info = MapInfo {
            title: map_name,
            artist: String::new(),
            difficulty_name: difficulty_name,
            mappers: mappers
                .split(", ")
                .collect::<String>()
                .split("& ")
                .map(|s| s.to_string())
                .collect(),
            length: length,
            cover_buf: cover_buf,
            audio_buf: audio_buf,
            note_count,
            object_count: note_count,
            ..Default::default()
        };

        let metadata = MapMetadata {
            ..Default::default()
        };

        let objects = MapObjects {
            notes,
            ..Default::default()
        };

        Ok(Map {
            id: map_id,
            info,
            metadata,
            objects,
        })
    }

    fn read_sspm_v2<T: Read + Seek>(mut reader: SSPMReader<T>) -> Result<Map> {
        reader.seek(SeekFrom::Current(2))?;
        let _hash = reader.read_sha1()?; // SHA1 hash of the file
        let millisecond = reader.read_u32()?; // Last object millisecond
        let note_count = reader.read_u32()?; // Note object count
        let object_count = reader.read_u32()?; // Total object count ( including notes )

        let difficulty = reader.read_u8()?; // Difficulty of the map
        let _star_rating = reader.read_u16()?; // never used ( Supposed to be for star rating )
        let has_audio = reader.read_bool()?; // Whether the map has audio data
        let has_cover = reader.read_bool()?; // Whether the map has cover data
        let _has_mod = reader.read_bool()?; // never used

        let custom_data_offset = reader.read_u64()?; // custom data for the map
        let _custom_data_length = reader.read_u64()?; // custom data length
        let audio_data_offset = reader.read_u64()?; // Length of audio data
        let audio_data_length = reader.read_u64()?; // Offset of audio data
        let cover_data_offset = reader.read_u64()?; // Length of cover data
        let cover_data_length = reader.read_u64()?; // Offset of cover data
        let _object_definition_offset = reader.read_u64()?; // Offset of object definitions
        let _object_definition_length = reader.read_u64()?; // Length of object definitions
        let object_data_offset = reader.read_u64()?; // Offset of object data
        let object_data_length = reader.read_u64()?; // Length of object data

        let map_id = reader.read_string()?; // Id of the map
        let map_name = reader.read_string()?; // Name of the map
        let _song_name = reader.read_string()?; // Song name
        let mappers_count = reader.read_u16()?; // Mappers count
        let mut mappers = Vec::<String>::new();

        let title_info: Vec<String> = map_name.split("-").map(|s| s.to_string()).collect();

        let artist = title_info[0].trim().to_string();
        let song_name = title_info.get(1).unwrap_or(&map_name).trim().to_string();

        for _ in 0..mappers_count {
            mappers.push(reader.read_string()?);
        }

        let mut difficulty_name =
            DifficultyName::from_u8(difficulty).unwrap_or(DifficultyName::None("N/A".to_string()));

        if let Ok(custom_data) = SSPMSerde::read_custom_data(&mut reader, custom_data_offset) {
            if let Some(ObjectType::String(Some(data))) = custom_data.get("difficulty_name") {
                difficulty_name.set_value(data.clone());
            }
        }

        let mut audio_buf = vec![0u8; audio_data_length as usize];
        let mut cover_buf = vec![0u8; cover_data_length as usize];

        let mut audio: Option<Vec<u8>> = None;
        let mut cover: Option<Vec<u8>> = None;

        if has_audio {
            reader.seek(SeekFrom::Start(audio_data_offset))?;
            reader.read_exact(&mut audio_buf)?;
            audio = Some(audio_buf);
        }

        if has_cover {
            reader.seek(SeekFrom::Start(cover_data_offset))?;
            reader.read_exact(&mut cover_buf)?;
            cover = Some(cover_buf);
        }

        let mut object_definitions = HashMap::<u8, ObjectDefinition>::new();
        let object_definition_count = reader.read_u8()?;

        for count in 0..object_definition_count {
            let name = reader.read_string()?;
            let values = reader.read_u8()?;

            let mut definitions = Vec::<ObjectType>::new();

            for _ in 0..values {
                let obj_type = reader.read_u8()?;
                let data = ObjectType::from_sspm(obj_type)?;

                definitions.push(data);
            }

            // There should be an empty byte after each object definition
            if let Ok(x) = reader.read_u8()
                && x != 0x00
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "missing empty byte on object definition",
                ));
            }

            object_definitions.insert(
                count,
                ObjectDefinition {
                    name,
                    millisecond: 0,
                    data: definitions,
                },
            );
        }

        reader.seek(SeekFrom::Start(object_data_offset))?;
        let object_section_end = reader.stream_position()? + object_data_length;

        let mut notes = Vec::<Note>::new();
        let mut objects = Vec::<ObjectDefinition>::new();

        while reader.stream_position()? < object_section_end {
            let ms = reader.read_u32()?;
            let definition = reader.read_u8()?;

            let object =
                SSPMSerde::read_definitions(&object_definitions[&definition], ms, &mut reader)?;

            // Matching objects to convertable objects based on the names
            match object.name.as_str() {
                "ssp_note" => notes.push(Note::from_sspm_definition(object)?),
                _ => objects.push(object),
            }
        }

        let info = MapInfo {
            title: song_name,
            artist,
            difficulty_name: difficulty_name,
            mappers,
            length: millisecond,
            cover_buf: cover,
            audio_buf: audio,
            note_count,
            object_count,
            ..Default::default()
        };

        let metadata = MapMetadata {
            ..Default::default()
        };

        let objects = MapObjects {
            notes,
            undefined: objects,
        };

        Ok(Map {
            id: map_id,
            info,
            metadata,
            objects,
        })
    }

    fn read_definitions<T: Read + Seek>(
        marker_definition: &ObjectDefinition,
        ms: u32,
        parser: &mut SSPMReader<T>,
    ) -> io::Result<ObjectDefinition> {
        let mut object_types = Vec::<ObjectType>::new();

        for def in marker_definition.data.iter() {
            match def {
                ObjectType::U8(_) => object_types.push(Self::read_u8(parser)?),
                ObjectType::U16(_) => object_types.push(Self::read_u16(parser)?),
                ObjectType::U32(_) => object_types.push(Self::read_u32(parser)?),
                ObjectType::U64(_) => object_types.push(Self::read_u64(parser)?),
                ObjectType::F32(_) => object_types.push(Self::read_f32(parser)?),
                ObjectType::F64(_) => object_types.push(Self::read_f64(parser)?),
                ObjectType::Vec2(_) => object_types.push(Self::read_vec2(parser)?),
                ObjectType::Buf(_) => object_types.push(Self::read_buf(parser)?),
                ObjectType::LongBuf(_) => object_types.push(Self::read_long_buf(parser)?),
                ObjectType::String(_) => object_types.push(Self::read_string(parser)?),
                ObjectType::LongString(_) => object_types.push(Self::read_long_string(parser)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid object type",
                    ));
                }
            }
        }

        Ok(ObjectDefinition {
            name: marker_definition.name.clone(),
            millisecond: ms,
            data: object_types,
        })
    }

    // Even though this follows the specification some maps have invalid custom data that causes errors
    // if custom data is invalid then just ignore custom data and continue parsing
    fn read_custom_data<T: Read + Seek>(
        mut reader: &mut SSPMReader<T>,
        offset: u64,
    ) -> Result<HashMap<String, ObjectType>> {
        let mut custom_data = HashMap::<String, ObjectType>::new();
        reader.seek(SeekFrom::Start(offset))?;

        let custom_data_fields = reader.read_u16()?;

        for _ in 0..custom_data_fields {
            let name = reader.read_string()?;
            let value = reader.read_u8()?;
            let data_type = ObjectType::from_sspm(value)?;
            let value = SSPMSerde::read_types(&data_type, &mut reader)?;

            custom_data.insert(name, value);
        }

        Ok(custom_data)
    }

    fn read_types<T: Read + Seek>(
        object_type: &ObjectType,
        parser: &mut SSPMReader<T>,
    ) -> io::Result<ObjectType> {
        match object_type {
            ObjectType::U8(_) => Self::read_u8(parser),
            ObjectType::U16(_) => Self::read_u16(parser),
            ObjectType::U32(_) => Self::read_u32(parser),
            ObjectType::U64(_) => Self::read_u64(parser),
            ObjectType::F32(_) => Self::read_f32(parser),
            ObjectType::F64(_) => Self::read_f64(parser),
            ObjectType::Vec2(_) => Self::read_vec2(parser),
            ObjectType::Buf(_) => Self::read_buf(parser),
            ObjectType::LongBuf(_) => Self::read_long_buf(parser),
            ObjectType::String(_) => Self::read_string(parser),
            ObjectType::LongString(_) => Self::read_long_string(parser),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid object type",
            )),
        }
    }

    fn read_u8<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::U8(Some(parser.read_u8()?)))
    }

    fn read_u16<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::U16(Some(parser.read_u16()?)))
    }

    fn read_u32<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::U32(Some(parser.read_u32()?)))
    }

    fn read_u64<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::U64(Some(parser.read_u64()?)))
    }

    fn read_f32<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::F32(Some(parser.read_f32()?)))
    }

    fn read_f64<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::F64(Some(parser.read_f64()?)))
    }

    fn read_vec2<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        let quantum = parser.read_bool()?;
        let mut pos = Vector2::zero();

        match quantum {
            true => {
                pos.x = parser.read_f32()?;
                pos.y = parser.read_f32()?;
            }
            false => {
                pos.x = parser.read_u8()? as f32;
                pos.y = parser.read_u8()? as f32;
            }
        };

        pos.x -= 1.0;
        pos.y = -pos.y + 1.0;

        Ok(ObjectType::Vec2(Some(pos)))
    }

    fn read_buf<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        let mut length = [0u8; 2];
        parser.read_exact(&mut length)?;
        let mut buffer = vec![0u8; u16::from_le_bytes(length) as usize];
        parser.read_exact(&mut buffer)?;

        Ok(ObjectType::Buf(Some(buffer)))
    }

    fn read_long_buf<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        let mut length = [0u8; 4];
        parser.read_exact(&mut length)?;
        let mut buffer = vec![0u8; u32::from_le_bytes(length) as usize];
        parser.read_exact(&mut buffer)?;

        Ok(ObjectType::LongBuf(Some(buffer)))
    }

    fn read_string<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::String(Some(parser.read_string()?)))
    }

    fn read_long_string<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        Ok(ObjectType::LongString(Some(parser.read_long_string()?)))
    }

    // TODO: parse vec types
    #[allow(unused)]
    fn read_vec<T: Read + Seek>(parser: &mut SSPMReader<T>) -> Result<ObjectType> {
        todo!()
    }
}
