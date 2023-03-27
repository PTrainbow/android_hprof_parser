use std::collections::HashMap;
use std::{error, fmt};
use crate::hprof_parser::snapshot::{IndexOutOfBoundsError, Snapshot};
use crate::hprof_parser::{ClassRecord, constant, HprofResult, StringRecord};

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
pub(crate) fn load_class<'a>(snapshot: &'a Snapshot, map: &HashMap<usize, &'a str>) -> Result<ClassRecord<'a>, IndexOutOfBoundsError> {
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
        Some(record) => {
            *record
        }
    };
    return Ok(ClassRecord {
        class_id,
        serial_id,
        name,
    });
}

/// TODO parse
/// ID | ID | ID | ID | 4 byte | 4 byte
pub(crate) fn load_stack_frame(snapshot: &Snapshot) -> Result<bool, IndexOutOfBoundsError> {
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    snapshot.read_u32()?;
    return Ok(true);
}

/// TODO parse
/// 4byte | 4byte | 4byte(size) | ID * size
pub(crate) fn load_stack_trace(snapshot: &Snapshot) -> Result<bool, IndexOutOfBoundsError> {
    snapshot.read_u32()?;
    snapshot.read_u32()?;
    let size = snapshot.read_u32()?;
    for _ in 0..size {
        snapshot.read_bytes_by_id_size()?;
    }
    return Ok(true);
}

/// TODO parse
/// complex
/// too many subtag
pub(crate) fn load_heap<'a>(snapshot: &'a Snapshot, length: usize, _: &HprofResult) -> Result<bool, Box<dyn error::Error>> {
    let mut cursor = 0;
    while cursor < length {
        // read tag
        let tag = snapshot.read_u8()?;
        cursor += 1;
        match tag {
            constant::ROOT_UNKNOWN => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_JNI_GLOBAL => {
                cursor += load_basic_obj(snapshot)?;
                snapshot.read_bytes_by_id_size()?;
                cursor += snapshot.id_size.get();
            }
            constant::ROOT_JNI_LOCAL => {
                cursor += load_jni_local(snapshot)?;
            }
            constant::ROOT_JAVA_FRAME => {
                cursor += load_java_frame(snapshot)?;
            }
            constant::ROOT_NATIVE_STACK => {
                cursor += load_native_stack(snapshot)?;
            }
            constant::ROOT_STICKY_CLASS => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_THREAD_BLOCK => {
                cursor += load_thread_block(snapshot)?;
            }
            constant::ROOT_MONITOR_USED => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_THREAD_OBJECT => {
                cursor += load_thread_obj(snapshot)?;
            }
            constant::CLASS_DUMP => {
                cursor += load_class_dump(snapshot)?;
            }
            constant::INSTANCE_DUMP => {
                cursor += load_instance_dump(snapshot)?;
            }
            constant::OBJECT_ARRAY_DUMP => {
                cursor += load_object_array(snapshot)?;
            }
            constant::PRIMITIVE_ARRAY_DUMP => {
                cursor += load_primitive_array(snapshot)?;
            }
            constant::HEAP_DUMP_INFO => {
                cursor += load_head_dump_info(snapshot)?;
            }
            constant::ROOT_INTERNED_STRING => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_FINALIZING => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_DEBUGGER => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_REFERENCE_CLEANUP => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_VM_INTERNAL => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::ROOT_JNI_MONITOR => {
                cursor += load_basic_obj(snapshot)?;
            }
            constant::HEAP_UNREACHABLE => {
                cursor += load_basic_obj(snapshot)?;
            }
            _ => {
                println!("unknown tag = {}", tag);
                return Err(Box::new(UnknownTagError {
                    tag
                }));
            }
        }
    }
    return Ok(true);
}

fn load_basic_obj(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    // println!("load_basic_obj id = {}", root_id);
    return Ok(snapshot.id_size.get());
}

fn load_jni_local(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    snapshot.read_u32()?;
    // println!("load_jni_local id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4 * 2);
}

fn load_java_frame(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    snapshot.read_u32()?;
    // println!("load_java_frame id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4 * 2);
}

fn load_native_stack(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    // println!("load_native_stack id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4);
}

fn load_thread_block(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    // println!("load_thread_block id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4);
}

fn load_thread_obj(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    let _ = snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    snapshot.read_u32()?;
    // println!("load_java_frame id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4 * 2);
}

/// TODO parse
/// complex
fn load_class_dump(snapshot: &Snapshot) -> Result<usize, Box<dyn error::Error>> {
    // let id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    // let class_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // let classloader_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // let instance_size = snapshot.read_u32()?;
    snapshot.read_u32()?;
    let mut bytes_read = (7 * snapshot.id_size.get()) + 4 + 4;
    // constant pool
    let constant_pool_size = snapshot.read_u16()?;
    bytes_read += 2;
    for _i in 0..constant_pool_size {
        snapshot.read_u16()?;
        bytes_read += 2;
        let field_type = snapshot.read_u8()?;
        bytes_read += 1;
        let size = get_type_size(field_type)?;
        snapshot.read_u8_array(size)?;
        bytes_read += size;
    }
    // static fields
    let static_field_size = snapshot.read_u16()?;
    bytes_read += 2;
    for _i in 0..static_field_size {
        snapshot.read_bytes_by_id_size()?;
        bytes_read += snapshot.id_size.get();
        let field_type = snapshot.read_u8()?;
        bytes_read += 1;
        let size = get_type_size(field_type)?;
        snapshot.read_u8_array(size)?;
        bytes_read += size;
    }
    // instance fields
    let instance_field_size = snapshot.read_u16()?;
    bytes_read += 2;
    for _i in 0..instance_field_size {
        snapshot.read_bytes_by_id_size()?;
        bytes_read += snapshot.id_size.get();
        // let field_type = snapshot.read_u8()?;
        snapshot.read_u8()?;
        bytes_read += 1;
    }
    // println!("load_class_dump id = {}, class_id = {}, classloader = {}, instance_size = {},\
    // constant_pool_size = {}, static_field_size = {}, instance_field_size = {}",
    //          id, class_id, classloader_id, instance_size,
    //          constant_pool_size, static_field_size, instance_field_size);
    return Ok(bytes_read);
}

/// TODO parse
/// ID | 4byte | ID | remaining
fn load_instance_dump(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    // let root_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    snapshot.read_u32()?;
    // let stack_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    let remaining = snapshot.read_u32()? as usize;
    snapshot.read_u8_array(remaining)?;
    // println!("load_instance_dump id = {}", root_id);
    return Ok(snapshot.id_size.get() * 2 + 4 * 2 + remaining);
}

/// TODO parse
/// ID | 4byte | 4byte(size) |ID | ID*size
fn load_object_array(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    // let root_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // let stack_id = snapshot.read_u32()?;
    snapshot.read_u32()?;
    let size = snapshot.read_u32()? as usize;
    // let class_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // println!("load_object_array id = {}", root_id);
    let remaining = snapshot.id_size.get() * size;
    snapshot.read_u8_array(remaining)?;
    return Ok(snapshot.id_size.get() * 2 + 4 * 2 + remaining);
}

/// TODO parse
/// ID | 4byte | 4byte(size) | 1byte(type) | size*type
fn load_primitive_array(snapshot: &Snapshot) -> Result<usize, Box<dyn error::Error>> {
    // let root_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // let stack_id = snapshot.read_u32()?;
    snapshot.read_u32()?;
    let size = snapshot.read_u32()? as usize;
    let filed_type = snapshot.read_u8()?;
    let type_size = get_type_size(filed_type)?;
    let remaining = size * type_size;
    snapshot.read_u8_array(remaining)?;
    // println!("load_primitive_array id = {}", root_id);
    return Ok(snapshot.id_size.get() + 4 * 2 + 1 + remaining);
}

/// TODO parse
/// 4 byte | ID
fn load_head_dump_info(snapshot: &Snapshot) -> Result<usize, IndexOutOfBoundsError> {
    // let heap_id = snapshot.read_u32()?;
    snapshot.read_u32()?;
    // let heap_name_id = snapshot.read_bytes_by_id_size()?;
    snapshot.read_bytes_by_id_size()?;
    // println!("load_head_dump_info id = {}", heap_id);
    return Ok(snapshot.id_size.get() + 4);
}

fn get_type_size(field_type: u8) -> Result<usize, UnknownTagError> {
    let mut size: usize = 0;
    match field_type {
        constant::OBJECT => {
            size += 4;
        }
        constant::BOOLEAN => {
            size += 1;
        }
        constant::CHAR => {
            size += 2;
        }
        constant::FLOAT => {
            size += 4;
        }
        constant::DOUBLE => {
            size += 8;
        }
        constant::BYTE => {
            size += 1;
        }
        constant::SHORT => {
            size += 2;
        }
        constant::INT => {
            size += 4;
        }
        constant::LONG => {
            size += 8;
        }
        _ => {
            println!("unknown filed type = {}", field_type);
            return Err(UnknownTagError {
                tag: field_type
            })
        }
    }
    Ok(size)
}

#[derive(Debug)]
pub struct UnknownTagError {
    tag: u8,
}

impl error::Error for UnknownTagError {

}

impl fmt::Display for UnknownTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnknownSubTagError! tag = {}", self.tag)
    }
}
