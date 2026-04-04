//! Библиотека для чтения и записи транзакций в форматах `binary`, `csv` и `txt`.
//!
//! Основная точка входа в API — трейт [`Parser`] и конкретные реализации:
//! [`BinaryParser`], [`CsvParser`] и [`TextParser`].
#![warn(missing_docs)]

use clap::ValueEnum;
use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

pub use binary::BinaryParser;
pub use csv::CsvParser;
pub use error::*;
pub use text::TextParser;
mod binary;
mod csv;
mod error;
mod text;

/// Общий трейт для чтения и записи набора транзакций.
pub trait Parser: Sized {
    /// Считывает все транзакции из входного потока.
    fn from_read<R: Read>(r: &mut R) -> ReaderResult<Vec<Transaction>>;

    /// Записывает все транзакции в указанный выходной поток.
    fn write_to<W: Write>(w: &mut W, input: &[Transaction]) -> WriterResult<()>;
}

/// Поддерживаемые форматы сериализации.
#[derive(Debug, Default, Clone, ValueEnum)]
pub enum Format {
    /// CSV-представление с заголовком.
    Csv,
    /// Бинарный формат с сигнатурой `YPBN`.
    #[default]
    Binary,
    /// Человекочитаемый текстовый формат.
    Txt,
}

impl Transaction {
    /// Создает транзакцию со значениями по умолчанию.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Транзакция в нормализованном внутреннем представлении.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    /// Уникальный идентификатор транзакции.
    tx_id: u64,
    /// Тип транзакции.
    tx_type: TxType,
    /// Идентификатор пользователя-отправителя. Для системных пополнений (`DEPOSIT`) может быть `0`.
    from_user_id: u64,
    /// Идентификатор пользователя-получателя. Для системных списаний (`WITHDRAWAL`) может быть `0`.
    to_user_id: u64,
    /// Сумма транзакции в наименьших единицах валюты (например, в центах).
    ///
    /// *Вопрос! Согласно спецификации, должен быть негативным i64 только для бинарного формата, и u64 для остальных двух.
    /// Можно в теории сделать еще одну структуру, но это дупликация кода, и цель задания все же парсинг*
    amount: i64,
    /// Время совершения транзакции в формате Unix-времени (миллисекунды с начала эпохи).
    timestamp: u64,
    /// Статус транзакции.
    status: TxStatus,
    /// Текстовое описание транзакции.
    description: Option<String>,
}

/// Статус транзакции.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxStatus {
    /// Транзакция завершилась успешно.
    Success,
    /// Транзакция завершилась ошибкой.
    Failure,
    /// Транзакция еще не завершена.
    #[default]
    Pending,
}

/// Тип транзакции.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxType {
    /// Пополнение счета.
    Deposit,
    /// Перевод между пользователями.
    #[default]
    Transfer,
    /// Списание средств.
    Withdrawal,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TX_ID: {}", self.tx_id)?;
        writeln!(f, "TX_TYPE: {}", self.tx_type)?;
        writeln!(f, "FROM_USER_ID: {}", self.from_user_id)?;
        writeln!(f, "TO_USER_ID: {}", self.to_user_id)?;
        writeln!(f, "AMOUNT: {}", self.amount)?;
        writeln!(f, "TIMESTAMP: {}", self.timestamp)?;
        writeln!(f, "STATUS: {}", self.status)?;
        if let Some(desc) = &self.description {
            writeln!(f, "DESCRIPTION: \"{desc}\"",)?;
        }
        Ok(())
    }
}

impl Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TxStatus::*;
        let txt = match self {
            Success => "SUCCESS",
            Failure => "FAILURE",
            Pending => "PENDING",
        };

        write!(f, "{txt}")
    }
}

impl TxStatus {
    fn as_str(&self) -> &'static str {
        use TxStatus::*;
        match self {
            Success => "SUCCESS",
            Failure => "FAILURE",
            Pending => "PENDING",
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TxType::*;
        let txt = match self {
            Deposit => "DEPOSIT",
            Transfer => "TRANSFER",
            Withdrawal => "WITHDRAWAL",
        };
        write!(f, "{txt}")
    }
}

impl TxType {
    fn as_str(&self) -> &'static str {
        use TxType::*;
        match self {
            Deposit => "DEPOSIT",
            Transfer => "TRANSFER",
            Withdrawal => "WITHDRAWAL",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct TransactionPartial {
    tx_id: Option<u64>,
    tx_type: Option<TxType>,
    from_user_id: Option<u64>,
    to_user_id: Option<u64>,
    amount: Option<i64>,
    timestamp: Option<u64>,
    status: Option<TxStatus>,
    description: Option<String>,
}

impl FromStr for TxType {
    type Err = ReaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TxType::*;
        let t = match s {
            "DEPOSIT" => Deposit,
            "TRANSFER" => Transfer,
            "WITHDRAWAL" => Withdrawal,
            _ => return Err(ReaderError::TxType),
        };
        Ok(t)
    }
}

impl FromStr for TxStatus {
    type Err = ReaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TxStatus::*;
        let t = match s {
            "SUCCESS" => Success,
            "FAILURE" => Failure,
            "PENDING" => Pending,
            _ => return Err(ReaderError::TxStatus),
        };
        Ok(t)
    }
}

impl TransactionPartial {
    fn is_empty(&self) -> bool {
        self.tx_id.is_none()
            && self.tx_type.is_none()
            && self.from_user_id.is_none()
            && self.to_user_id.is_none()
            && self.amount.is_none()
            && self.timestamp.is_none()
            && self.status.is_none()
            && self.description.is_none()
    }

    fn set_tx_id(&mut self, tx_id: u64) {
        self.tx_id = Some(tx_id);
    }
    fn set_tx_type(&mut self, tx_type: TxType) {
        self.tx_type = Some(tx_type);
    }
    fn set_from_user_id(&mut self, fui: u64) {
        self.from_user_id = Some(fui)
    }
    fn set_to_user_id(&mut self, tui: u64) {
        self.to_user_id = Some(tui)
    }
    fn set_amount(&mut self, a: i64) {
        self.amount = Some(a)
    }
    fn set_timestamp(&mut self, t: u64) {
        self.timestamp = Some(t)
    }
    fn set_status(&mut self, s: TxStatus) {
        self.status = Some(s)
    }
    fn set_description(&mut self, d: Option<String>) {
        self.description = d
    }
}

impl TryFrom<TransactionPartial> for Transaction {
    type Error = ReaderError;
    fn try_from(value: TransactionPartial) -> Result<Self, Self::Error> {
        let missing_field = |field: &'static str| ReaderError::Transaction {
            idx: 0,
            field: field.into(),
        };
        Ok(Self {
            tx_id: value.tx_id.ok_or_else(|| missing_field("tx_id"))?,
            tx_type: value.tx_type.ok_or_else(|| missing_field("tx_type"))?,
            from_user_id: value.from_user_id.ok_or_else(|| missing_field("from_user_id"))?,
            to_user_id: value.to_user_id.ok_or_else(|| missing_field("to_user_id"))?,
            amount: value.amount.ok_or_else(|| missing_field("amount"))?,
            timestamp: value.timestamp.ok_or_else(|| missing_field("timestamp"))?,
            status: value.status.ok_or_else(|| missing_field("status"))?,
            description: value.description,
        })
    }
}
