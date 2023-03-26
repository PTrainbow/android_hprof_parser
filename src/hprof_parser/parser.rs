use std::collections::HashMap;
use crate::hprof_parser::snapshot::{IndexOutOfBoundsError, Snapshot};
use crate::hprof_parser::{ClassRecord, StringRecord};

/// ID | content
/// id_size | content * size
pub(crate) fn load_string<'a>(snapshot: &'a Snapshot, length: usize) -> Result<StringRecord<'a>, IndexOutOfBoundsError> {
    let id = snapshot.read_bytes_by_id_size()?;
    let char_array = snapshot.read_u8_array(length - snapshot.id_size.get())?;
    let str = std::str::from_utf8(char_array).unwrap();
    return Ok(StringRecord {
        id,
        content: str,
    });
}

/// number | object id | number | str id
/// 4 byte |  id_size  | 4 byte | id_size
pub(crate) fn load_class<'a>(snapshot: &'a Snapshot, map: &HashMap<usize, &'a str>) -> Result<ClassRecord<'a> , IndexOutOfBoundsError>{
    // read number
    let serial_id = snapshot.read_u32()? as usize;
    // read object id
    let class_id = snapshot.read_bytes_by_id_size()?;
    // read number
    snapshot.read_u32()?;
    // read str id
    let str_id = snapshot.read_bytes_by_id_size()?;
    let name = match map.get(&str_id) {
        None => {
            "unknown"
        }
        Some(record) =>{
            *record
        }
    };
    return Ok(ClassRecord {
        class_id,
        serial_id,
        name,
    });
}


// complex
// pub(crate) fn load_heap<'a>(snapshot: &'a Snapshot, map: &HashMap<usize, &'a str>) {
//     // read number
//     let serial_id = snapshot.read_u32() as usize;
//     // read object id
//     let class_id = snapshot.read_bytes_by_id_size();
//     // read number
//     snapshot.read_u32();
//     // read str id
//     let str_id = snapshot.read_bytes_by_id_size();
//     let name = match map.get(&str_id) {
//         None => {
//             "unknown"
//         }
//         Some(record) =>{
//             *record
//         }
//     };
//
// }
