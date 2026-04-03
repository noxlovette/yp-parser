use std::{array::TryFromSliceError, io, num::ParseIntError, str::Utf8Error};
use thiserror::Error;

pub(crate) type WriterResult<T> = Result<T, WriterError>;
pub(crate) type ReaderResult<T> = Result<T, ReaderError>;

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("io error")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Slice parsing error")]
    Slice(#[from] TryFromSliceError),
    #[error("Int parsing error")]
    Int(#[from] ParseIntError),
    #[error("Transaction parsing error")]
    Transaction,
    #[error("TxType parsing error")]
    TxType,
    #[error("TxStatus parsing error")]
    TxStatus,
    #[error("UTF8 error")]
    Utf(#[from] Utf8Error),
    #[error("Unknown field name")]
    Field,
    #[error("Corrupted TXT file")]
    TextCorrupt,
    #[error("Corrupted CSV file")]
    CsvCorrupt,
}
