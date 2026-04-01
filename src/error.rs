use std::{array::TryFromSliceError, io, str::Utf8Error};
use thiserror::Error;
pub type ParserResult<T> = Result<T, ParserError>;
pub type WriterResult<T> = Result<T, WriterError>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("reader error")]
    Io(#[from] io::Error),
    #[error("slice error")]
    Slice(#[from] TryFromSliceError),
    #[error("transaction error")]
    Transaction,
    #[error("TxType parsing error")]
    TxType,
    #[error("TxStatus parsing error")]
    TxStatus,
    #[error("utf8 error")]
    Utf(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("writer error")]
    Io(#[from] io::Error),
}
