use thiserror::Error;
use {
    std::{
        fmt, {io, string},
    },
};

#[derive(Error, Debug)]
pub enum TagReaderError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Parsing error at byte {0}")]
    ParseError(usize),

    #[error("Parsing tag error is {0} ")]
    ParseTagError(String),

    #[error("Parsing File Header error is {0} ")]
    ParseFileHeaderError(String),

    #[error("The tag is invalid: {0}")]
    InvalidTag(String),

    #[error("The operation was cancelled")]
    Cancelled,

    #[error("The amf parse is error: {0}")]
    AmfParseError(String),
    #[error("unknown tag Type : {0}")]
    UnknownTagType(u8),
    #[error("unknown tag size")]
    Incomplete
}

#[derive(Debug, Error)]
pub enum Amf0ReadError {
    #[error( "Encountered unknown marker: {0}")]
    UnknownMarker( u8 ),
    #[error( "parser string error: {0}")]
    StringParseError(string::FromUtf8Error),
    #[error( "wrong type")]
    WrongType,
    #[error( "UnexpectedObjectEnd")]
    UnexpectedObjectEnd,
    #[error( "OutOfRangeReference: {0}")]
    OutOfRangeReference(usize),
    #[error( "InvalidDate: {0}")]
    InvalidDate(f64),
    #[error( "CircularReference")]
    CircularReference,
    #[error( "Unsupported")]
    Unsupported,
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("IO error")]
    StringParser(#[from] string::FromUtf8Error),
}

#[derive(Debug, Error)]
pub enum AVCError {
    #[error( "NAL unit type not implemented: {0}")]
    UnknownNAL( u8 ),
    #[error("SubWidthC undefined")]
    SubWidthCUndefined,
    #[error("SubHeightC undefined")]
    SubHeightCUndefined,
    #[error("Parameter Length Inadequate")]
    ParameterLength,
    #[error("Read Bits Error")]
    ReadBitsError
}
