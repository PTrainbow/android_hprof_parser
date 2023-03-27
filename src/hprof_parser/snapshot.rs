use std::cell::Cell;
use std::{error, fmt};

pub struct Snapshot<'a> {
    pub(crate) input: &'a [u8],
    pub(crate) id_size: Cell<usize>,
    pub(crate) current_position: Cell<usize>,
    pub(crate) max_size: usize,
}

impl<'a> Snapshot<'a> {
    pub fn available(&self) -> bool {
        return self.current_position.get() < self.max_size;
    }

    pub fn read_u8(&self) -> Result<u8, IndexOutOfBoundsError> {
        if !self.available() {
            return Err(IndexOutOfBoundsError {
                length: self.max_size,
                index: self.current_position.get(),
            });
        }
        let result = self.input[self.current_position.get()];
        self.current_position.set(self.current_position.get() + 1);
        return Ok(result);
    }

    pub fn read_u16(&self) -> Result<u16, IndexOutOfBoundsError> {
        if self.is_out_of_bounds(2) {
            return Err(IndexOutOfBoundsError {
                length: self.max_size,
                index: self.current_position.get() + 2,
            });
        }
        let u8_array = &self.input[self.current_position.get()..self.current_position.get() + 2];
        self.current_position.set(self.current_position.get() + 2);
        return Ok(transform_u8_array_to_u16(u8_array));
    }

    pub fn read_u32(&self) -> Result<u32, IndexOutOfBoundsError> {
        if self.is_out_of_bounds(4) {
            return Err(IndexOutOfBoundsError {
                length: self.max_size,
                index: self.current_position.get() + 4,
            });
        }
        let u8_array = &self.input[self.current_position.get()..self.current_position.get() + 4];
        self.current_position.set(self.current_position.get() + 4);
        return Ok(transform_u8_array_to_u32(u8_array));
    }

    pub fn read_bytes_by_id_size(&self) -> Result<usize, IndexOutOfBoundsError> {
        if self.is_out_of_bounds(self.id_size.get()) {
            return Err(IndexOutOfBoundsError {
                length: self.max_size,
                index: self.current_position.get() + self.id_size.get(),
            });
        }
        let u8_array = &self.input[self.current_position.get()..self.current_position.get() + self.id_size.get()];
        self.current_position.set(self.current_position.get() + self.id_size.get());
        let length = u8_array.len();
        return match length {
            4 => {
                Ok(transform_u8_array_to_u32(u8_array) as usize)
            }
            8 => {
                Ok(transform_u8_array_to_u64(u8_array) as usize)
            }
            _ => {
                panic!("error id_size {}", self.id_size.get())
            }
        };
    }

    pub fn read_u8_array(&self, size: usize) -> Result<&[u8], IndexOutOfBoundsError> {
        if self.is_out_of_bounds(size) {
            return Err(IndexOutOfBoundsError {
                length: self.max_size,
                index: self.current_position.get() + size,
            });
        }
        let result = &self.input[self.current_position.get()..self.current_position.get() + size];
        self.current_position.set(self.current_position.get() + size);
        return Ok(result);
    }

    fn is_out_of_bounds(&self, size: usize) -> bool {
        return self.current_position.get() + size >= self.max_size;
    }
}

fn transform_u8_array_to_u32(b: &[u8]) -> u32 {
    if b.len() != 4 {
        panic!("input {:?} is not 4 bytes", b)
    }
    let b0: u32 = u32::from(b[0]) << 24;
    let b1: u32 = u32::from(b[1]) << 16;
    let b2: u32 = u32::from(b[2]) << 8;
    let b3: u32 = u32::from(b[3]);
    return b0 + b1 + b2 + b3;
}

fn transform_u8_array_to_u64(b: &[u8]) -> u64 {
    let length = b.len();
    if length != 8 {
        panic!("input {:?} is not 8 bytes", b)
    }
    let mut result: u64 = 0;
    for i in 0..length {
        result += u64::from(b[i]) << (8 * (length - i - 1))
    }
    return result;
}

fn transform_u8_array_to_u16(b: &[u8]) -> u16 {
    if b.len() != 2 {
        panic!("input {:?} is not 2 bytes", b)
    }
    let b0: u16 = u16::from(b[0]) << 8;
    let b1: u16 = u16::from(b[1]);
    return b0 + b1;
}

#[derive(Debug)]
pub struct IndexOutOfBoundsError {
    length: usize,
    index: usize,
}

impl error::Error for IndexOutOfBoundsError {

}

impl fmt::Display for IndexOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IndexOutOfBounds! length = {}, index = {}", self.length, self.index)
    }
}