pub use error::*;
use std::io::{Read, Write};
mod binary;
mod csv;
mod error;
mod text;

#[derive(Debug, Default)]
pub enum Format {
    Csv,
    #[default]
    Binary,
    Txt,
}

impl Transaction {
    pub fn from_read<R: Read>(r: &mut R, fmt: Format) -> ParserResult<Self> {
        todo!()
    }
    pub fn write_to<W: Write>(&mut self, w: &mut W, fmt: Format) -> ParserResult<()> {
        todo!()
    }

    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct Transaction {
    tx_id: u64,
    tx_type: TxType,
    /// 0 for deposits
    from_user_id: u64,
    /// 0 for withdrawals
    to_user_id: u64,
    amount: i64,
    timestamp: u64,
    status: TxStatus,
    description: Option<String>,
}

#[derive(Debug, Default)]
pub enum TxStatus {
    Success,
    Failure,
    #[default]
    Pending,
}

#[derive(Debug, Default)]
pub enum TxType {
    Deposit,
    #[default]
    Transfer,
    Withdrawal,
}
