use std::{array::TryFromSliceError, io, str::Utf8Error};
use thiserror::Error;
pub type ParserResult<T> = Result<T, ParserError>;
pub type WriterResult<T> = Result<T, WriterError>;
pub type ReaderResult<T> = Result<T, ReaderError>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("reader error")]
    Reader(#[from] ReaderError),
    #[error("writer error")]
    Writer(#[from] WriterError),
}

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("io error")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("io error")]
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
