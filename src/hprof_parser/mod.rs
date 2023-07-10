use crate::hprof_parser::snapshot::Snapshot;
use memmap::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::result::Result as StdResult;

mod constant;
// mod parser;
mod snapshot;

mod errors;

pub use errors::Error;
pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct HprofParser<'a> {
    snapshot: Snapshot<'a>,
    result: HprofResult<'a>,
}

#[derive(Default, Debug)]
pub struct HprofResult<'a> {
    string_map: HashMap<u64, &'a str>,
    class_map_by_id: HashMap<u64, &'a str>,
    class_map_by_serial: HashMap<u32, &'a str>,
}

impl<'hp: 'hr, 'hr> HprofParser<'hp> {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<()> {
        let file = File::open(path)?;
        Self::parse_file(&file)
    }

    pub fn parse_file(file: &File) -> Result<()> {
        let file_bytes = file.metadata().unwrap().len() as usize;
        let mapped_file = unsafe { Mmap::map(&file) }?;
        let snapshot = Snapshot::new(&mapped_file[0..file_bytes])?;
        println!("header: {:?}", snapshot.header());
        let parser = HprofParser {
            snapshot,
            result: Default::default(),
        };
        let _ = parser.parse_record();
        println!("{:?}", parser);
        Ok(())
    }

    /// record format
    /// 1 byte(tag) | 4 byte(ts) | 4 byte(length) | length byte(real content) |
    fn parse_record(&'hp self) -> Result<()> {
        self.snapshot.parse_records()
    }
}
