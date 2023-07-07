use std::io;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {

    #[error("UnknownTag: {0:#x}")]
    UnknownTag(u8),

    #[error("IndexOutOfBounds: length = {length}, index = {index}")]
    IndexOutOfBounds {
        length: usize,
        index: usize,
    },

    #[error(transparent)]
    IO(#[from] io::Error),
}