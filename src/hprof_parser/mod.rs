use crate::hprof_parser::snapshot::Snapshot;
use memmap::Mmap;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::result::Result as StdResult;

mod constant;
mod parser;
mod snapshot;

mod errors;

pub use errors::Error;
pub type Result<T> = StdResult<T, Error>;

pub struct HprofParser<'a> {
    snapshot: &'a Snapshot<'a>,
}

#[derive(Default, Debug)]
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
    pub fn parse(file: &File) -> Result<()> {
        let mut result = HprofResult::default();
        let file_bytes = file.metadata().unwrap().len() as usize;
        let mapped_file = unsafe { Mmap::map(&file) }?;

        let snapshot = &Snapshot {
            input: &mapped_file[0..file_bytes],
            id_size: Cell::new(0),
            current_position: Cell::new(0),
            max_size: file_bytes,
        };
        let parser = &HprofParser { snapshot };
        let header = parser.parse_header();
        match header {
            Err(err) => {
                println!("{}", err);
            }
            Ok(hprof_header) => {
                println!("hrpof version = {}", hprof_header.version);
            }
        }
        let _ = parser.parse_record(&mut result);

        Ok(())
    }

    /// header format
    /// 18 byte | 4 byte(idSize) | 4 byte | 4 byte |
    fn parse_header(&self) -> Result<HprofHeader> {
        // read version, 18(version string) + 1 byte(NULL)
        let snapshot = self.snapshot;
        let version = String::from_utf8(
            snapshot.read_u8_array(constant::HPROF_HEADER_VERSION_SIZE as usize)?[0..18].to_vec(),
        )
        .unwrap();
        let id_size = snapshot.read_u32()? as usize;
        snapshot.id_size.set(id_size);
        // don't care next 8 byte
        let _ = snapshot.read_u32();
        let _ = snapshot.read_u32();
        return Ok(HprofHeader { version });
    }

    /// record format
    /// 1 byte(tag) | 4 byte(ts) | 4 byte(length) | length byte(real content) |
    fn parse_record(&self, result: &mut HprofResult<'hr>) -> Result<bool> {
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
                    println!("tag string start = {}", snapshot.current_position.get());
                    match parser::load_string(snapshot, length) {
                        Err(err) => {
                            // TODO
                            println!("{}", err);
                            break;
                        }
                        Ok(record) => {
                            result.string_map.insert(record.id, record.content);
                        }
                    }
                    println!("tag string end = {}", snapshot.current_position.get());
                }
                constant::TAG_LOAD_CLASS => {
                    println!("tag load class start = {}", snapshot.current_position.get());
                    match parser::load_class(snapshot, &result.string_map) {
                        Err(err) => {
                            // TODO
                            println!("{}", err);
                            break;
                        }
                        Ok(record) => {
                            result.class_map_by_id.insert(record.class_id, record.name);
                            result
                                .class_map_by_serial
                                .insert(record.serial_id, record.name);
                        }
                    }
                    println!("tag load class end = {}", snapshot.current_position.get());
                }
                constant::TAG_STACK_FRAME => {
                    println!(
                        "tag stack frame start = {}",
                        snapshot.current_position.get()
                    );
                    match parser::load_stack_frame(snapshot) {
                        Err(err) => {
                            // TODO
                            println!("{}", err);
                            break;
                        }
                        _ => {}
                    }
                    println!("tag stack frame end = {}", snapshot.current_position.get());
                }
                constant::TAG_STACK_TRACE => {
                    println!(
                        "tag stack trace start = {}",
                        snapshot.current_position.get()
                    );
                    match parser::load_stack_trace(snapshot) {
                        Err(err) => {
                            // TODO
                            println!("{}", err);
                            break;
                        }
                        _ => {}
                    }
                    println!("tag stack trace end = {}", snapshot.current_position.get());
                }
                constant::TAG_HEAP_DUMP | constant::TAG_HEAP_DUMP_SEGMENT => {
                    println!("tag heap dump start = {}", snapshot.current_position.get());
                    match parser::load_heap(snapshot, length, result) {
                        Err(err) => {
                            // TODO
                            println!("{}", err);
                            break;
                        }
                        _ => {}
                    }
                    println!("tag heap dump end = {}\n", snapshot.current_position.get());
                }
                _ => {
                    // other just skip
                    match snapshot.read_u8_array(length) {
                        Err(err) => {
                            println!("{}", err);
                            break;
                        }
                        _ => {}
                    }
                }
            };
        }
        println!(
            "current postion = {}, max_size = {}",
            snapshot.current_position.get(),
            snapshot.max_size
        );
        Ok(true)
    }
}

pub struct HprofHeader {
    version: String,
}
