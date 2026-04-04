use crate::{Parser, ReaderError, ReaderResult, Transaction, TransactionPartial};

/// Парсер CSV-формата.
pub struct CsvParser;

const HEADER: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

fn parse_csv_line(line: &str) -> ReaderResult<Vec<String>> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        field.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else if field.is_empty() {
                    in_quotes = true;
                } else {
                    return Err(ReaderError::CsvCorrupt);
                }
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut field));
            }
            _ => field.push(ch),
        }
    }

    if in_quotes {
        return Err(ReaderError::CsvCorrupt);
    }

    fields.push(field);
    Ok(fields)
}

fn escape_csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

impl Parser for CsvParser {
    fn from_read<R: std::io::Read>(r: &mut R) -> crate::ReaderResult<Vec<crate::Transaction>> {
        let mut buf = String::new();
        let mut output: Vec<Transaction> = Vec::new();

        r.read_to_string(&mut buf)?;

        let mut iter = buf.lines();

        if iter.next().is_some_and(|l| l == HEADER) {
            for line in iter.filter(|l| !l.is_empty()) {
                let fields = parse_csv_line(line)?;
                output.push(Transaction::from_fields(fields.into_iter().enumerate())?)
            }
        } else {
            return Err(ReaderError::CsvCorrupt);
        }

        Ok(output)
    }

    fn write_to<W: std::io::Write>(
        w: &mut W,
        input: &[crate::Transaction],
    ) -> crate::WriterResult<()> {
        w.write_all(HEADER.as_bytes())?;
        w.write_all(b"\n")?;
        for tx in input {
            w.write_all(tx.tx_id.to_string().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.tx_type.as_str().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.from_user_id.to_string().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.to_user_id.to_string().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.amount.to_string().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.timestamp.to_string().as_bytes().as_ref())?;
            w.write_all(b",")?;
            w.write_all(tx.status.as_str().as_bytes())?;
            w.write_all(b",")?;
            let description = escape_csv_field(tx.description.as_deref().unwrap_or(""));
            w.write_all(description.as_bytes())?;
            w.write_all(b"\n")?;
        }
        w.flush()?;

        Ok(())
    }
}

impl Transaction {
    fn from_fields<I>(fields: I) -> ReaderResult<Self>
    where
        I: Iterator<Item = (usize, String)>,
    {
        let mut tx = TransactionPartial::default();
        for (i, f) in fields {
            match i {
                0 => {
                    tx.set_tx_id(f.parse()?);
                }
                1 => {
                    tx.set_tx_type(f.parse()?);
                }
                2 => {
                    tx.set_from_user_id(f.parse()?);
                }
                3 => {
                    tx.set_to_user_id(f.parse()?);
                }
                4 => {
                    tx.set_amount(f.parse()?);
                }
                5 => {
                    tx.set_timestamp(f.parse()?);
                }
                6 => {
                    tx.set_status(f.parse()?);
                }
                7 => {
                    tx.set_description(Some(f));
                }
                _ => return Err(ReaderError::CsvCorrupt),
            }
        }

        tx.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TxStatus, TxType};
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
        CsvParser::from_read(&mut Cursor::new(input))
    }

    fn write_bytes(input: &[Transaction]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        CsvParser::write_to(&mut cursor, input).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn parses_multiple_transactions_from_cursor() {
        let input = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,0,77,1500,1700000000,SUCCESS,initial deposit
2,TRANSFER,77,88,-250,1700000100,PENDING,
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
        assert_eq!(second.description.as_deref(), Some(""));
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
        assert_eq!(second.description.as_deref(), Some(""));
    }

    #[test]
    fn rejects_invalid_header() {
        let input = br#"TX_ID,TX_TYPE,FROM_USER_ID
1,DEPOSIT,0
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::CsvCorrupt));
    }

    #[test]
    fn rejects_extra_csv_field() {
        let input = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,0,77,1500,1700000000,SUCCESS,initial deposit,extra
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::CsvCorrupt));
    }

    #[test]
    fn rejects_incomplete_transaction() {
        let input = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,0,77,1500,1700000000
"#;

        let err = parse_bytes(input).unwrap_err();

        assert!(matches!(err, ReaderError::Transaction { .. }));
    }

    #[test]
    fn parses_quoted_description_with_comma() {
        let input = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,0,77,1500,1700000000,SUCCESS,"hello,world"
"#;

        let parsed = parse_bytes(input).unwrap();

        assert_eq!(parsed[0].description.as_deref(), Some("hello,world"));
    }

    #[test]
    fn round_trips_quoted_description() {
        let bytes = write_bytes(&[tx(
            11,
            TxType::Deposit,
            0,
            55,
            750,
            1_700_000_300,
            TxStatus::Success,
            Some("hello,\"world\""),
        )]);

        let rendered = String::from_utf8(bytes.clone()).unwrap();
        assert!(rendered.contains("\"hello,\"\"world\"\"\""));

        let parsed = parse_bytes(&bytes).unwrap();
        assert_eq!(parsed[0].description.as_deref(), Some("hello,\"world\""));
    }
}
