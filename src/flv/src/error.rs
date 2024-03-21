use thiserror::Error;

#[derive(Error, Debug)]
pub enum TagReaderError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Parsing error at byte {0}")]
    ParseError(usize),

    #[error("The tag is invalid: {0}")]
    InvalidTag(String),

    #[error("The operation was cancelled")]
    Cancelled,

    #[error("The amf parse is error: {0}")]
    AmfParseError(String),
    #[error("unknown tag Type : {0}")]
    UnknownTagType(u8),
}