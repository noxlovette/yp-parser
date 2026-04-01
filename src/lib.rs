use clap::ValueEnum;
use std::fmt::Display;

pub use binary::*;
pub use error::*;
mod binary;
mod csv;
mod error;
mod text;

#[derive(Debug, Default, Clone, ValueEnum)]
pub enum Format {
    Csv,
    #[default]
    Binary,
    Txt,
}

impl Transaction {
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

#[derive(Debug, Default, Clone, Copy)]
pub enum TxStatus {
    Success,
    Failure,
    #[default]
    Pending,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TxType {
    Deposit,
    #[default]
    Transfer,
    Withdrawal,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TX ID: {}", self.tx_id)?;
        writeln!(f, "TX TYPE: {}", self.tx_type)?;
        writeln!(f, "FROM: {}", self.from_user_id)?;
        writeln!(f, "TO: {}", self.to_user_id)?;
        writeln!(f, "AMOUNT: {}", self.amount)?;
        writeln!(f, "TIMESTAMP: {}", self.timestamp)?;
        writeln!(f, "STATUS: {}", self.status)?;
        if let Some(desc) = &self.description {
            writeln!(f, "DESCRIPTION: {}", desc)?;
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
