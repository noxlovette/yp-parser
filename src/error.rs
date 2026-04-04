use std::{array::TryFromSliceError, io, num::ParseIntError, str::Utf8Error};
use thiserror::Error;

pub(crate) type WriterResult<T> = Result<T, WriterError>;
pub(crate) type ReaderResult<T> = Result<T, ReaderError>;

/// Ошибки записи транзакций.
#[derive(Debug, Error)]
pub enum WriterError {
    #[error("io error")]
    /// Ошибка IO
    Io(#[from] io::Error),
}

/// Ошибки чтения транзакций.
#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("IO error")]
    /// Ошибка IO
    Io(#[from] io::Error),
    #[error("Slice parsing error")]
    /// Ошибка конвертации из slice
    Slice(#[from] TryFromSliceError),
    #[error("Int parsing error")]
    /// Ошибка парсинга int
    Int(#[from] ParseIntError),
    #[error("Transaction parsing error at idx {idx:?} while trying to read field {field:?}")]
    /// Ошибка парсинга транзакции
    Transaction {
        /// Где произошла ошибка
        idx: usize,
        /// Какое поле пытались парсить
        field: String,
    },
    #[error("TxType parsing error")]
    /// Ошибка парсинга типа транзакции
    TxType,
    #[error("TxStatus parsing error")]
    /// Ошибка парсинга статуса транзакции
    TxStatus,
    #[error("UTF8 error")]
    /// Ошибка чтения UTF-8
    Utf(#[from] Utf8Error),
    #[error("Unknown field name")]
    /// Незнакомое поле
    Field,
    #[error("Corrupted TXT file")]
    /// Поврежденный .txt
    TextCorrupt,
    #[error("Corrupted CSV file")]
    /// Поврежденный .csv
    CsvCorrupt,
}
