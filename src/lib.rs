mod binary;
mod csv;
mod error;
mod text;

#[derive(Debug)]
pub enum Router {
    Csv,
    Binary,
    Txt,
}

#[derive(Debug, Default)]
pub struct Transaction {
    tx_id: u64,
    tx_type: TxType,
    /// 0 for deposits
    from_user_id: u64,
    /// 0 for withdrawals
    to_user_id: u64,
    amount: u64,
    timestamp: u64,
    status: TxStatus,
    description: String,
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
