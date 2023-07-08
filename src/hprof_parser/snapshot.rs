use crate::hprof_parser::constant;
use crate::{Error, Result};
use byteorder::{ByteOrder, ReadBytesExt, BE, LE};
use std::cell::{Cell, RefCell};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::str;

pub trait HprofRead {
    fn remain(&self) -> usize;

    fn id_size(&self) -> usize;

    fn read_u8(&self) -> Result<u8>;

    fn read_u16(&self) -> Result<u16>;

    fn read_u32(&self) -> Result<u32>;

    fn read_u64(&self) -> Result<u64>;

    fn read_id(&self) -> Result<u64>;

    fn read_u8_array(&self, size: usize) -> Result<&[u8]>;

    fn read_utf8(&self, size: usize) -> Result<&str>;

    fn skip(&self, n: usize) -> Result<()>;

    fn read_f32(&self) -> Result<f32>;

    fn read_f64(&self) -> Result<f64>;

    fn read_i8(&self) -> Result<i8> {
        let v = self.read_u8()?;
        Ok(v as _)
    }

    fn read_i16(&self) -> Result<i16> {
        let v = self.read_u16()?;
        Ok(v as _)
    }

    fn read_i32(&self) -> Result<i32> {
        let v = self.read_u32()?;
        Ok(v as _)
    }

    fn read_i64(&self) -> Result<i64> {
        let v = self.read_u64()?;
        Ok(v as _)
    }
}

struct Slice<'a> {
    buf: &'a [u8],
    id_size: u32,
    n: Cell<usize>,
}

impl<'a> Slice<'a> {
    fn new(buf: &'a [u8], size: u32) -> Self {
        Self {
            buf,
            id_size: size,
            n: Cell::new(0),
        }
    }

    fn check(&self, need: usize) -> Result<()> {
        let remain = self.remain();
        if remain < need {
            return Err(Error::IndexOutOfBounds {
                request: need,
                remain,
            });
        }
        Ok(())
    }
}

impl<'a> HprofRead for Slice<'a> {
    fn remain(&self) -> usize {
        self.buf.len() - self.n.get()
    }

    fn id_size(&self) -> usize {
        self.id_size as _
    }

    fn read_u8(&self) -> Result<u8> {
        self.check(1)?;
        let n = self.n.get();
        let b = self.buf[n];
        self.n.set(n + 1);
        Ok(b)
    }

    fn read_u16(&self) -> Result<u16> {
        self.check(2)?;
        let n = self.n.get();
        let i = (&self.buf[n..]).read_u16::<LE>()?;
        self.n.set(n + 2);
        Ok(i)
    }

    fn read_u32(&self) -> Result<u32> {
        self.check(4)?;
        let n = self.n.get();
        let i = (&self.buf[n..]).read_u32::<BE>()?;
        self.n.set(n + 4);
        Ok(i)
    }

    fn read_u64(&self) -> Result<u64> {
        self.check(8)?;
        let n = self.n.get();
        let i = (&self.buf[n..]).read_u64::<LE>()?;
        self.n.set(n + 8);
        Ok(i)
    }

    fn read_id(&self) -> Result<u64> {
        let id_size = self.id_size();
        self.check(id_size)?;
        let n = self.n.get();
        let id = (&self.buf[n..]).read_uint::<LE>(id_size)?;
        self.n.set(n + id_size);
        Ok(id)
    }

    fn read_u8_array(&self, size: usize) -> Result<&[u8]> {
        self.check(size)?;
        let n = self.n.get();
        let arr = &self.buf[n..n + size];
        self.n.set(n + size);
        return Ok(arr);
    }

    fn read_utf8(&self, size: usize) -> Result<&str> {
        let char_array = self.read_u8_array(size)?;
        Ok(str::from_utf8(char_array)?)
    }

    fn skip(&self, size: usize) -> Result<()> {
        self.check(size)?;
        let n = self.n.get();
        self.n.set(n + size);
        Ok(())
    }

    fn read_f32(&self) -> Result<f32> {
        self.check(4)?;
        let n = self.n.get();
        let v = (&self.buf[n..]).read_f32::<LE>()?;
        self.n.set(n + 4);
        Ok(v)
    }

    fn read_f64(&self) -> Result<f64> {
        self.check(8)?;
        let n = self.n.get();
        let v = (&self.buf[n..]).read_f64::<LE>()?;
        self.n.set(n + 8);
        Ok(v)
    }
}

pub struct Snapshot<'a> {
    header: HprofHeader,
    slice: Slice<'a>,
    records: RefCell<Vec<Record<'a>>>,
}

impl<'a> Debug for Snapshot<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("header", &self.header)
            .field("records", &self.records.borrow())
            .finish()
    }
}

impl<'a> Snapshot<'a> {
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        let header = HprofHeader::try_from(slice)?;
        Ok(Self {
            slice: Slice::new(&slice[header.size..], header.id_size),
            header,
            records: RefCell::new(Vec::new()),
        })
    }

    pub fn header(&self) -> &HprofHeader {
        &self.header
    }
}

impl<'a> HprofRead for Snapshot<'a> {
    fn remain(&self) -> usize {
        self.slice.remain()
    }

    fn id_size(&self) -> usize {
        self.slice.id_size()
    }

    fn read_u8(&self) -> Result<u8> {
        self.slice.read_u8()
    }

    fn read_u16(&self) -> Result<u16> {
        self.slice.read_u16()
    }

    fn read_u32(&self) -> Result<u32> {
        self.slice.read_u32()
    }

    fn read_u64(&self) -> Result<u64> {
        self.slice.read_u64()
    }

    fn read_id(&self) -> Result<u64> {
        self.slice.read_id()
    }

    fn read_u8_array(&self, size: usize) -> Result<&[u8]> {
        self.slice.read_u8_array(size)
    }

    fn read_utf8(&self, size: usize) -> Result<&str> {
        self.slice.read_utf8(size)
    }

    fn skip(&self, n: usize) -> Result<()> {
        self.slice.skip(n)
    }

    fn read_f32(&self) -> Result<f32> {
        self.slice.read_f32()
    }

    fn read_f64(&self) -> Result<f64> {
        self.slice.read_f64()
    }
}

#[derive(Debug)]
pub struct HprofHeader {
    pub version: String,
    pub id_size: u32,
    pub size: usize,
}

impl TryFrom<&[u8]> for HprofHeader {
    type Error = Error;

    fn try_from(mut slice: &[u8]) -> Result<Self> {
        let c_str = CStr::from_bytes_until_nul(slice)?;
        let n = c_str.to_bytes_with_nul().len();
        slice = &slice[n..];
        let id_size = BE::read_u32(slice);
        Ok(Self {
            version: c_str.to_str()?.to_string(),
            id_size,
            size: n + 12,
        })
    }
}

#[derive(Debug)]
pub enum Record<'a> {
    String {
        id: u64,
        content: &'a str,
    },

    LoadClass {
        serial_number: u32,
        object_id: u64,
        stack_trace_serial_number: u32,
        class_name_id: u64,
    },

    UnLoadClass(u32),

    StackFrame {
        id: u64,
        method_name_id: u64,
        method_signature_id: u64,
        source_file_name_id: u64,
        class_serial_number: u32,
        line_no: i32,
    },

    StackTrace {
        serial_number: u32,
        thread_serial_number: u32,
        stack_frame_ids: Vec<u64>,
    },

    HeapDump(Vec<SubTag<'a>>),

    Unknown {
        tag: u8,
        content: &'a [u8],
    },
}

impl<'a> Snapshot<'a> {
    pub fn parse_records(&'a self) -> Result<()> {
        let total = self.remain();
        let mut count = 0;

        while count < total {
            count += self.parse_record()?;
        }
        assert_eq!(count, total);

        Ok(())
    }

    fn parse_record(&'a self) -> Result<usize> {
        let before = self.remain();

        let tag = self.read_u8()?;
        // microseconds
        self.skip(4)?;
        let len = self.read_u32()? as usize;

        let record = match tag {
            constant::TAG_STRING => Record::String {
                id: self.read_id()?,
                content: self.read_utf8(len - self.id_size())?,
            },

            constant::TAG_LOAD_CLASS => Record::LoadClass {
                serial_number: self.read_u32()?,
                object_id: self.read_id()?,
                stack_trace_serial_number: self.read_u32()?,
                class_name_id: self.read_id()?,
            },

            constant::TAG_UNLOAD_CLASS => Record::UnLoadClass(self.read_u32()?),

            constant::TAG_STACK_FRAME => Record::StackFrame {
                id: self.read_id()?,
                method_name_id: self.read_id()?,
                method_signature_id: self.read_id()?,
                source_file_name_id: self.read_id()?,
                class_serial_number: self.read_u32()?,
                line_no: self.read_i32()?,
            },

            constant::TAG_STACK_TRACE => {
                let serial_number = self.read_u32()?;
                let thread_serial_number = self.read_u32()?;
                let len = self.read_u32()? as usize;
                let mut v = Vec::<u64>::with_capacity(len);

                for i in &mut v {
                    *i = self.read_id()?;
                }

                Record::StackTrace {
                    serial_number,
                    thread_serial_number,
                    stack_frame_ids: v,
                }
            }

            constant::TAG_HEAP_DUMP | constant::TAG_HEAP_DUMP_SEGMENT => {
                let mut ret: Vec<SubTag> = Vec::new();
                let mut count = 0;

                while count < len {
                    let (subtag, size) = self.parse_subtag()?;
                    count += size;
                    ret.push(subtag);
                }
                assert_eq!(count, len);

                Record::HeapDump(ret)
            }

            _ => Record::Unknown {
                tag,
                content: self.read_u8_array(len)?,
            },
        };
        self.records.borrow_mut().push(record);

        Ok(before - self.remain())
    }

    fn parse_subtag(&self) -> Result<(SubTag, usize)> {
        let before = self.remain();
        match self.read_u8()? {
            0xFF => Ok(SubTag::RootUnknown(self.read_id()?)),

            0x01 => Ok(SubTag::RootJniGlobal {
                object_id: self.read_id()?,
                jni_global_ref_id: self.read_id()?,
            }),

            0x02 => Ok(SubTag::RootJniLocal(Object::parse(self)?)),
            0x03 => Ok(SubTag::RootJavaFrame(Object::parse(self)?)),

            0x04 => Ok(SubTag::RootNativeStack(NativeObject::parse(self)?)),

            0x05 => Ok(SubTag::RootStickyClass(self.read_id()?)),

            0x06 => Ok(SubTag::RootThreadBlock(NativeObject::parse(self)?)),

            0x07 => Ok(SubTag::RootMonitorUsed(self.read_id()?)),

            0x08 => Ok(SubTag::RootThreadObject(Object::parse(self)?)),

            0x20 => {
                let class_object_id = self.read_id()?;
                let stack_trace_serial_number = self.read_u32()?;
                let super_class_object_id = self.read_id()?;
                let class_loader_object_id = self.read_id()?;
                let signers_object_id = self.read_id()?;
                let protection_domain_object_id = self.read_id()?;
                let instance_size = self.read_u32()?;

                let count: usize = self.read_u16()? as _;
                let mut constants: Vec<Constant> = Vec::new();
                for _ in 0..count {
                    constants.push(Constant::parse(self)?);
                }

                let count: usize = self.read_u16()? as _;
                let mut static_fields: Vec<StaticField> = Vec::new();
                for _ in 0..count {
                    static_fields.push(StaticField::parse(self)?);
                }

                let count: usize = self.read_u16()? as _;
                let mut instant_fields: Vec<InstantField> = Vec::new();
                for _ in 0..count {
                    instant_fields.push(InstantField::parse(self)?);
                }

                Ok(SubTag::ClassDump {
                    class_object_id,
                    stack_trace_serial_number,
                    super_class_object_id,
                    class_loader_object_id,
                    signers_object_id,
                    protection_domain_object_id,
                    instance_size,
                    constants,
                    static_fields,
                    instant_fields,
                })
            }

            0x21 => {
                let object_id = self.read_id()?;
                let stack_trace_serial_number = self.read_u32()?;
                let class_object_id = self.read_id()?;
                let count: usize = self.read_u32()? as _;
                let instance_field_values = self.read_u8_array(count)?;

                Ok(SubTag::InstanceDump {
                    object_id,
                    stack_trace_serial_number,
                    class_object_id,
                    instance_field_values,
                })
            }

            0x22 => {
                let array_object_id = self.read_id()?;
                let stack_trace_serial_number = self.read_u32()?;
                let count = self.read_u32()?;
                let array_class_object_id = self.read_id()?;
                let mut elements = Vec::new();

                for _ in 0..count {
                    elements.push(self.read_id()?);
                }

                Ok(SubTag::ObjectArrayDump {
                    array_object_id,
                    stack_trace_serial_number,
                    array_class_object_id,
                    elements,
                })
            }

            0x23 => {
                let array_object_id = self.read_id()?;
                let stack_trace_serial_number = self.read_u32()?;
                let count = self.read_u32()?;
                let element_type: JavaType = self.read_u8()?.try_into()?;
                let mut elements: Vec<JavaValue> = Vec::new();

                for _ in 0..count {
                    elements.push(JavaValue::parse(self, element_type)?)
                }

                Ok(SubTag::PrimitiveArrayDump {
                    array_object_id,
                    stack_trace_serial_number,
                    element_type,
                    elements,
                })
            }

            tag => Err(Error::UnknownSubTag(tag)),
        }
        .map(|subtag| (subtag, before - self.remain()))
    }
}

#[derive(Debug)]
pub enum SubTag<'a> {
    RootUnknown(u64),

    RootJniGlobal {
        object_id: u64,
        jni_global_ref_id: u64,
    },

    RootJniLocal(Object),
    RootJavaFrame(Object),

    RootNativeStack(NativeObject),

    RootStickyClass(u64),

    RootThreadBlock(NativeObject),

    RootMonitorUsed(u64),

    RootThreadObject(Object),

    ClassDump {
        class_object_id: u64,
        stack_trace_serial_number: u32,
        super_class_object_id: u64,
        class_loader_object_id: u64,
        signers_object_id: u64,
        protection_domain_object_id: u64,
        instance_size: u32,
        constants: Vec<Constant>,
        static_fields: Vec<StaticField>,
        instant_fields: Vec<InstantField>,
    },

    InstanceDump {
        object_id: u64,
        stack_trace_serial_number: u32,
        class_object_id: u64,
        instance_field_values: &'a [u8],
    },

    ObjectArrayDump {
        array_object_id: u64,
        stack_trace_serial_number: u32,
        array_class_object_id: u64,
        elements: Vec<u64>,
    },

    PrimitiveArrayDump {
        array_object_id: u64,
        stack_trace_serial_number: u32,
        element_type: JavaType,
        elements: Vec<JavaValue>,
    },
}

#[derive(Debug)]
pub struct InstantField {
    name_string_id: u64,
    java_type: JavaType,
}

impl InstantField {
    fn parse<R: HprofRead>(r: &R) -> Result<Self> {
        Ok(Self {
            name_string_id: r.read_id()?,
            java_type: r.read_u8()?.try_into()?,
        })
    }
}

#[derive(Debug)]
pub struct StaticField {
    name_string_id: u64,
    java_value: JavaValue,
}

impl StaticField {
    fn parse<R: HprofRead>(r: &R) -> Result<Self> {
        Ok(Self {
            name_string_id: r.read_id()?,
            java_value: JavaValue::parse_with_type(r)?,
        })
    }
}

#[derive(Debug)]
pub struct Constant {
    constant_pool_index: u32,
    java_value: JavaValue,
}

impl Constant {
    fn parse<R: HprofRead>(r: &R) -> Result<Self> {
        Ok(Self {
            constant_pool_index: r.read_u32()?,
            java_value: JavaValue::parse_with_type(r)?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum JavaType {
    Object = 2,
    Boolean = 4,
    Char = 5,
    Float = 6,
    Double = 7,
    Byte = 8,
    Short = 9,
    Int = 10,
    Long = 11,
}

impl TryFrom<u8> for JavaType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            2 => Ok(JavaType::Object),
            4 => Ok(JavaType::Boolean),
            5 => Ok(JavaType::Char),
            6 => Ok(JavaType::Float),
            7 => Ok(JavaType::Double),
            8 => Ok(JavaType::Byte),
            9 => Ok(JavaType::Short),
            10 => Ok(JavaType::Int),
            11 => Ok(JavaType::Long),
            ty => Err(Error::UnknownJavaType(ty)),
        }
    }
}

#[derive(Debug)]
pub enum JavaValue {
    // 2
    Object(u32),
    // 4
    Boolean(bool),
    // 5
    Char(u16),
    // 6
    Float(f32),
    // 7
    Double(f64),
    // 8
    Byte(i8),
    // 9
    Short(i16),
    // 10
    Int(i32),
    // 11
    Long(i64),
}

impl JavaValue {
    fn parse_with_type<R: HprofRead>(r: &R) -> Result<Self> {
        Self::parse(r, JavaType::try_from(r.read_u8()?)?)
    }

    fn parse<R: HprofRead>(r: &R, ty: JavaType) -> Result<Self> {
        Ok(match ty {
            JavaType::Object => JavaValue::Object(r.read_u32()?),
            JavaType::Boolean => JavaValue::Boolean(r.read_u8()? != 0),
            JavaType::Char => JavaValue::Char(r.read_u16()?),
            JavaType::Float => JavaValue::Float(r.read_f32()?),
            JavaType::Double => JavaValue::Double(r.read_f64()?),
            JavaType::Byte => JavaValue::Byte(r.read_i8()?),
            JavaType::Short => JavaValue::Short(r.read_i16()?),
            JavaType::Int => JavaValue::Int(r.read_i32()?),
            JavaType::Long => JavaValue::Long(r.read_i64()?),
        })
    }
}

#[derive(Debug)]
pub struct Object {
    object_id: u64,
    thread_serial_number: u32,
    frame_number_in_stack_trace: i32,
}

impl Object {
    fn parse<R: HprofRead>(r: &R) -> Result<Self> {
        Ok(Self {
            object_id: r.read_id()?,
            thread_serial_number: r.read_u32()?,
            frame_number_in_stack_trace: r.read_i32()?,
        })
    }
}

#[derive(Debug)]
pub struct NativeObject {
    object_id: u64,
    thread_serial_number: u32,
}

impl NativeObject {
    fn parse<R: HprofRead>(r: &R) -> Result<Self> {
        Ok(Self {
            object_id: r.read_id()?,
            thread_serial_number: r.read_u32()?,
        })
    }
}
