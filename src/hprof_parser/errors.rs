use core::ffi::FromBytesUntilNulError;
use std::{io, str};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("date time parse error: {0}")]
    DateTime(i64),

    #[error("UnknownTag: {0:#x}")]
    UnknownTag(u8),

    #[error("The remaining data is not enough, request = {request}, remain = {remain}")]
    IndexOutOfBounds { request: usize, remain: usize },

    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Utf8(#[from] str::Utf8Error),

    #[error(transparent)]
    FromBytesUntilNul(#[from] FromBytesUntilNulError),

    #[error("unknown java type: {0}")]
    UnknownJavaType(u8),

    #[error("unknown subtag: {0}")]
    UnknownSubTag(u8),
}
