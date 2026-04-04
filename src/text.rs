use crate::{Parser, ReaderError, Transaction, TransactionPartial, TxStatus, TxType};
use std::str::FromStr;

/// Парсер текстового формата.
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
            w.write_all("\n".as_bytes())?;
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
                self.set_tx_id(id);
            }
            TxType(t) => {
                self.set_tx_type(t);
            }
            FromUserId(id) => {
                self.set_from_user_id(id);
            }
            ToUserId(id) => {
                self.set_to_user_id(id);
            }
            Amount(a) => {
                self.set_amount(a);
            }
            Timestamp(t) => {
                self.set_timestamp(t);
            }
            Status(s) => {
                self.set_status(s);
            }
            Description(d) => {
                self.set_description(d);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[allow(clippy::too_many_arguments)]
    fn tx(
        tx_id: u64,
        tx_type: TxType,
        from_user_id: u64,
        to_user_id: u64,
        amount: i64,
        timestamp: u64,
        status: TxStatus,
        description: Option<&str>,
    ) -> Transaction {
        Transaction {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description: description.map(str::to_owned),
        }
    }

    fn parse_bytes(input: &[u8]) -> crate::ReaderResult<Vec<Transaction>> {
        TextParser::from_read(&mut Cursor::new(input))
    }

    fn write_bytes(input: &[Transaction]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        TextParser::write_to(&mut cursor, input).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn parses_multiple_transactions_from_cursor() {
        let input = br#"# comment before first transaction
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

        let parsed = parse_bytes(input).unwrap();

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
    fn round_trips_through_write_to_and_from_read() {
        let first = tx(
            9,
            TxType::Withdrawal,
            55,
            0,
            -500,
            1_700_000_200,
            TxStatus::Failure,
            Some("atm"),
        );
        let second = tx(
            10,
            TxType::Deposit,
            0,
            55,
            750,
            1_700_000_300,
            TxStatus::Success,
            None,
        );

        let bytes = write_bytes(&[first, second]);
        let parsed = parse_bytes(&bytes).unwrap();

        assert_eq!(parsed.len(), 2);

        let first = &parsed[0];
        assert_eq!(first.tx_id, 9);
        assert!(matches!(first.tx_type, TxType::Withdrawal));
        assert_eq!(first.from_user_id, 55);
        assert_eq!(first.to_user_id, 0);
        assert_eq!(first.amount, -500);
        assert_eq!(first.timestamp, 1_700_000_200);
        assert!(matches!(first.status, TxStatus::Failure));
        assert_eq!(first.description.as_deref(), Some("atm"));

        let second = &parsed[1];
        assert_eq!(second.tx_id, 10);
        assert!(matches!(second.tx_type, TxType::Deposit));
        assert_eq!(second.from_user_id, 0);
        assert_eq!(second.to_user_id, 55);
        assert_eq!(second.amount, 750);
        assert_eq!(second.timestamp, 1_700_000_300);
        assert!(matches!(second.status, TxStatus::Success));
        assert_eq!(second.description, None);
    }

    #[test]
    fn rejects_unknown_field_name() {
        let input = br#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 77
AMOUNT: 1500
TIMESTAMP: 1700000000
STATUS: SUCCESS
NOTE: nope
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::Field));
    }

    #[test]
    fn rejects_incomplete_transaction() {
        let input = br#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 77
AMOUNT: 1500
TIMESTAMP: 1700000000
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::Transaction { .. }));
    }

    #[test]
    fn rejects_corrupted_text_line() {
        let input = br#"TX_ID 1
TX_TYPE: DEPOSIT
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::TextCorrupt));
    }
}
