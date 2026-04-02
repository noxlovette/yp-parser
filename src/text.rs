use crate::{Parser, ReaderError, Transaction, TransactionPartial, TxStatus, TxType};
use std::str::FromStr;

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

        if !tx.is_empty() {
            output.push(tx.try_into()?);
        }

        Ok(output)
    }

    fn write_to<W: std::io::Write>(
        w: &mut W,
        input: &[crate::Transaction],
    ) -> crate::WriterResult<()> {
        for tx in input {
            w.write_all(format!("{tx}").as_bytes())?;
            w.write_all("\n".to_string().as_bytes())?;
        }
        w.flush()?;

        Ok(())
    }
}

impl FromStr for Fields {
    type Err = ReaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Fields::*;

        let (identifier, value) = s.split_once(": ").ok_or(ReaderError::TextCorrupt)?;

        let t = match identifier {
            "TX_ID" => TxId(value.parse()?),
            "TX_TYPE" => TxType(value.parse()?),
            "FROM_USER_ID" => FromUserId(value.parse()?),
            "TO_USER_ID" => ToUserId(value.parse()?),
            "AMOUNT" => Amount(value.parse()?),
            "TIMESTAMP" => Timestamp(value.parse()?),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn parse(input: &str) -> crate::ReaderResult<Vec<Transaction>> {
        TextParser::from_read(&mut Cursor::new(input.as_bytes()))
    }

    #[test]
    fn parses_multiple_transactions_with_comments_and_description() {
        let input = r#"# comment before first transaction
TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 77
AMOUNT: 1500
TIMESTAMP: 1700000000
STATUS: SUCCESS
DESCRIPTION: "initial deposit"

# separator comment
TX_ID: 2
TX_TYPE: TRANSFER
FROM_USER_ID: 77
TO_USER_ID: 88
AMOUNT: -250
TIMESTAMP: 1700000100
STATUS: PENDING
"#;

        let parsed = parse(input).unwrap();

        assert_eq!(parsed.len(), 2);

        let first = &parsed[0];
        assert_eq!(first.tx_id, 1);
        assert!(matches!(first.tx_type, TxType::Deposit));
        assert_eq!(first.from_user_id, 0);
        assert_eq!(first.to_user_id, 77);
        assert_eq!(first.amount, 1500);
        assert_eq!(first.timestamp, 1_700_000_000);
        assert!(matches!(first.status, TxStatus::Success));
        assert_eq!(first.description.as_deref(), Some("initial deposit"));

        let second = &parsed[1];
        assert_eq!(second.tx_id, 2);
        assert!(matches!(second.tx_type, TxType::Transfer));
        assert_eq!(second.from_user_id, 77);
        assert_eq!(second.to_user_id, 88);
        assert_eq!(second.amount, -250);
        assert_eq!(second.timestamp, 1_700_000_100);
        assert!(matches!(second.status, TxStatus::Pending));
        assert_eq!(second.description, None);
    }

    #[test]
    fn parses_last_transaction_without_trailing_blank_line() {
        let input = r#"TX_ID: 9
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 55
TO_USER_ID: 0
AMOUNT: -500
TIMESTAMP: 1700000200
STATUS: FAILURE"#;

        let parsed = parse(input).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].tx_id, 9);
        assert!(matches!(parsed[0].tx_type, TxType::Withdrawal));
        assert!(matches!(parsed[0].status, TxStatus::Failure));
    }

    #[test]
    fn rejects_unknown_field_name() {
        let input = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 77
AMOUNT: 1500
TIMESTAMP: 1700000000
STATUS: SUCCESS
NOTE: nope
"#;

        let err = parse(input).unwrap_err();

        assert!(matches!(err, ReaderError::Field));
    }

    #[test]
    fn rejects_incomplete_transaction() {
        let input = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 77
AMOUNT: 1500
TIMESTAMP: 1700000000
"#;

        let err = parse(input).unwrap_err();

        assert!(matches!(err, ReaderError::Transaction));
    }

    #[test]
    fn rejects_corrupted_text_line() {
        let input = r#"TX_ID 1
TX_TYPE: DEPOSIT
"#;

        let err = parse(input).unwrap_err();

        assert!(matches!(err, ReaderError::TextCorrupt));
    }
}
