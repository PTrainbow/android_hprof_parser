/// record type
pub const TAG_STRING: u8 = 0x01;
pub const TAG_LOAD_CLASS: u8 = 0x02;
// pub const TAG_UNLOAD_CLASS: u8 = 0x03;
pub const TAG_STACK_FRAME: u8 = 0x04;
pub const TAG_STACK_TRACE: u8 = 0x05;
// pub const TAG_ALLOC_SITES: u8 = 0x06;
// pub const TAG_HEAP_SUMMARY: u8 = 0x07;
// pub const TAG_START_THREAD: u8 = 0x0A;
// pub const TAG_END_THREAD: u8 = 0x0B;
pub const TAG_HEAP_DUMP: u8 = 0x0C;
// pub const TAG_CPU_SAMPLES: u8 = 0x0D;
// pub const TAG_CONTROL_SETTINGS: u8 = 0x0E;
pub const TAG_HEAP_DUMP_SEGMENT: u8 = 0x1C;
// pub const TAG_HEAP_DUMP_END: u8 = 0x2C;

/// subRecord type
pub const ROOT_JNI_GLOBAL: u8 = 0x01;
pub const ROOT_JNI_LOCAL: u8 = 0x02;
pub const ROOT_JAVA_FRAME: u8 = 0x03;
pub const ROOT_NATIVE_STACK: u8 = 0x04;
pub const ROOT_STICKY_CLASS: u8 = 0x05;
pub const ROOT_THREAD_BLOCK: u8 = 0x06;
pub const ROOT_MONITOR_USED: u8 = 0x07;
pub const ROOT_THREAD_OBJECT: u8 = 0x08;
pub const CLASS_DUMP: u8 = 0x20;
pub const INSTANCE_DUMP: u8 = 0x21;
pub const OBJECT_ARRAY_DUMP: u8 = 0x22;
pub const PRIMITIVE_ARRAY_DUMP: u8 = 0x23;
pub const ROOT_UNKNOWN: u8 = 0xFF;
pub const ROOT_INTERNED_STRING: u8 = 0x89;
pub const ROOT_FINALIZING: u8 = 0x8A;
pub const ROOT_DEBUGGER: u8 = 0x8B;
pub const ROOT_REFERENCE_CLEANUP: u8 = 0x8C;
pub const ROOT_VM_INTERNAL: u8 = 0x8D;
pub const ROOT_JNI_MONITOR: u8 = 0x8E;
pub const HEAP_DUMP_INFO: u8 = 0xFE;
pub const HEAP_UNREACHABLE: u8 = 0x90;

/// field type
pub const OBJECT: u8 = 2;
pub const BOOLEAN: u8 = 4;
pub const CHAR: u8 = 5;
pub const FLOAT: u8 = 6;
pub const DOUBLE: u8 = 7;
pub const BYTE: u8 = 8;
pub const SHORT: u8 = 9;
pub const INT: u8 = 10;
pub const LONG: u8 = 11;

pub const HPROF_HEADER_VERSION_SIZE: u8 = 19;