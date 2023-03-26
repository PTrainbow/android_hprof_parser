use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use memmap::Mmap;
use crate::hprof_parser::snapshot::{IndexOutOfBoundsError, Snapshot};

mod constant;
mod snapshot;
mod parser;

pub struct HprofParser<'a> {
    snapshot: &'a Snapshot<'a>,
}

pub struct HprofResult<'a> {
    string_map: HashMap<usize, &'a str>,
    class_map_by_id: HashMap<usize, &'a str>,
    class_map_by_serial: HashMap<usize, &'a str>,
}

pub struct StringRecord<'a> {
    id: usize,
    content: &'a str,
}

pub struct ClassRecord<'a> {
    class_id: usize,
    serial_id: usize,
    name: &'a str,
}

impl<'hp: 'hr, 'hr> HprofParser<'hp> {
    pub fn parse(file: &File) {
        let mut result = HprofResult {
            string_map: HashMap::new(),
            class_map_by_id: HashMap::new(),
            class_map_by_serial: HashMap::new(),
        };
        let file_bytes = file.metadata().unwrap().len() as usize;
        let mapped_file = unsafe { Mmap::map(&file) };
        let mapped_file = match mapped_file {
            Ok(data) => data,
            Err(error) => {
                panic!("mmap error: {:?}", error);
            }
        };
        let snapshot = &Snapshot {
            input: &mapped_file[0..file_bytes],
            id_size: Cell::new(0),
            current_position: Cell::new(0),
            max_size: file_bytes,
        };
        let parser = &HprofParser {
            snapshot
        };
        let _ = parser.parse_header();
        let _ = parser.parse_record(&mut result);
        print!("{}", result.string_map.len());
    }

    /// header format
    /// 18 byte | 4 byte(idSize) | 4 byte | 4 byte |
    fn parse_header(&self) -> Result<HprofHeader, IndexOutOfBoundsError> {
        // read version, 18(version string) + 1 byte(NULL)
        let snapshot = self.snapshot;
        let version = String::from_utf8(snapshot.read_u8_array(constant::HPROF_HEADER_VERSION_SIZE as usize)?[0..18].to_vec()).unwrap();
        let id_size = snapshot.read_u32()? as usize;
        snapshot.id_size.set(id_size);
        // don't care next 8 byte
        let _ = snapshot.read_u32();
        let _ = snapshot.read_u32();
        return Ok(HprofHeader {
            version
        });
    }

    /// record format
    /// 1 byte(tag) | 4 byte(ts) | 4 byte(length) | length byte(real content) |
    fn parse_record(&self, result: &mut HprofResult<'hr>) -> Result<bool, IndexOutOfBoundsError> {
        let snapshot = &self.snapshot;
        while snapshot.available() {
            // read tag
            let tag = snapshot.read_u8()?;
            // read ts, but don't care
            let _ = snapshot.read_u32()?;
            // read length
            let length = snapshot.read_u32()? as usize;
            // read content by tag
            match tag {
                constant::TAG_STRING => {
                    let record = parser::load_string(snapshot, length)?;
                    result.string_map.insert(record.id, record.content);
                }
                constant::TAG_LOAD_CLASS => {
                    let record = parser::load_class(snapshot, &result.string_map)?;
                    result.class_map_by_id.insert(record.class_id, record.name);
                    result.class_map_by_serial.insert(record.serial_id, record.name);
                }
                // constant::TAG_STACK_FRAME => {
                //     // TODO
                //     snapshot.read_u8_array(length);
                // }
                // constant::TAG_STACK_TRACE => {
                //     // TODO
                //     snapshot.read_u8_array(length);
                //
                // }
                // constant::TAG_HEAP_DUMP => {
                //     snapshot.read_u8_array(length);
                // }
                // constant::TAG_HEAP_DUMP_SEGMENT => {
                //     snapshot.read_u8_array(length);
                // }
                _ => {
                    // just skip
                    match snapshot.read_u8_array(length) {
                        Err(err) => {
                            break;
                        }
                        _ => {}
                    }
                }
            };
        }
        for (k, v) in &result.class_map_by_serial {
            println!("class name = {}", *v);
        }
        Ok(true)
    }
}

pub struct HprofHeader {
    version: String,
}