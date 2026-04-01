use std::str::FromStr;

use crate::{Parser, ReaderError, Transaction, TransactionPartial, TxStatus, TxType};

pub struct TextParser;

enum Fields {
    TxId(u64),
    TxType(TxType),
    FromUserId(u64),
    ToUserId(u64),
    Amount(i64),
    Timestamp(u64),
    Status(TxStatus),
    Description(Option<String>),
}

impl Parser for TextParser {
    fn from_read<R: std::io::Read>(r: &mut R) -> crate::ReaderResult<Vec<crate::Transaction>> {
        let mut buf = String::new();
        let mut output: Vec<Transaction> = Vec::new();
        r.read_to_string(&mut buf)?;

        let mut tx = TransactionPartial::default();
        for line in buf.lines() {
            if line.is_empty() {
                output.push(tx.try_into()?);
                tx = TransactionPartial::default();
                continue;
            }
            let trm = line.trim();
            if trm.starts_with("#") {
                continue;
            }
            tx.set_field(trm.parse()?);
        }

        Ok(output)
    }

    fn write_to<W: std::io::Write>(
        w: &mut W,
        input: &[crate::Transaction],
    ) -> crate::WriterResult<()> {
        todo!()
    }
}

impl FromStr for Fields {
    type Err = ReaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Fields::*;

        let (identifier, value) = s.split_once(": ").ok_or_else(|| ReaderError::TextCorrupt)?;

        let t = match identifier {
            "TX_ID" => TxId(u64::from_str(value)?),
            "TX_TYPE" => TxType(value.parse()?),
            "FROM_USER_ID" => FromUserId(u64::from_str(value)?),
            "TO_USER_ID" => ToUserId(u64::from_str(value)?),
            "AMOUNT" => Amount(i64::from_str(value)?),
            "TIMESTAMP" => Timestamp(u64::from_str(value)?),
            "STATUS" => Status(value.parse()?),
            "DESCRIPTION" => Description(Some(value.trim_matches('"').into())),
            _ => return Err(ReaderError::Field),
        };

        Ok(t)
    }
}

impl TransactionPartial {
    fn set_field(&mut self, field: Fields) {
        use Fields::*;
        match field {
            TxId(id) => {
                self.tx_id(id);
            }
            TxType(t) => {
                self.tx_type(t);
            }
            FromUserId(id) => {
                self.from_user_id(id);
            }
            ToUserId(id) => {
                self.to_user_id(id);
            }
            Amount(a) => {
                self.amount(a);
            }
            Timestamp(t) => {
                self.timestamp(t);
            }
            Status(s) => {
                self.status(s);
            }
            Description(d) => {
                self.description(d);
            }
        }
    }

    fn tx_id(&mut self, tx_id: u64) {
        self.tx_id = Some(tx_id);
    }
    fn tx_type(&mut self, tx_type: TxType) {
        self.tx_type = Some(tx_type);
    }
    fn from_user_id(&mut self, fui: u64) {
        self.from_user_id = Some(fui)
    }
    fn to_user_id(&mut self, tui: u64) {
        self.to_user_id = Some(tui)
    }
    fn amount(&mut self, a: i64) {
        self.amount = Some(a)
    }
    fn timestamp(&mut self, t: u64) {
        self.timestamp = Some(t)
    }
    fn status(&mut self, s: TxStatus) {
        self.status = Some(s)
    }
    fn description(&mut self, d: Option<String>) {
        self.description = d
    }
}

impl TryFrom<TransactionPartial> for Transaction {
    type Error = ReaderError;
    fn try_from(value: TransactionPartial) -> Result<Self, Self::Error> {
        use ReaderError::Transaction;
        Ok(Self {
            tx_id: value.tx_id.ok_or_else(|| Transaction)?,
            tx_type: value.tx_type.ok_or_else(|| Transaction)?,
            from_user_id: value.from_user_id.ok_or_else(|| Transaction)?,
            to_user_id: value.to_user_id.ok_or_else(|| Transaction)?,
            amount: value.amount.ok_or_else(|| Transaction)?,
            timestamp: value.timestamp.ok_or_else(|| Transaction)?,
            status: value.status.ok_or_else(|| Transaction)?,
            description: value.description,
        })
    }
}
