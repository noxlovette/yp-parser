use crate::{Parser, ReaderError, ReaderResult, Transaction, TxStatus, TxType, WriterResult};
use std::io::{Read, Write};

/// Парсер бинарного формата `.bin`.
pub struct BinaryParser;
const MAGIC: &[u8; 4] = b"YPBN";

impl Parser for BinaryParser {
    fn from_read<R: Read>(r: &mut R) -> ReaderResult<Vec<Transaction>> {
        let mut output = Vec::new();

        loop {
            let mut magic = [0u8; MAGIC.len()];
            match r.read(&mut magic[..1])? {
                0 => break,
                1 => r.read_exact(&mut magic[1..])?,
                _ => unreachable!(),
            }

            if magic != *MAGIC {
                return Err(ReaderError::Transaction);
            }

            let mut len = [0u8; 4];
            r.read_exact(&mut len)?;
            let payload_len = u32::from_be_bytes(len) as usize;

            let mut payload = vec![0u8; payload_len];
            r.read_exact(&mut payload)?;
            output.push(payload.as_slice().try_into()?);
        }

        Ok(output)
    }

    fn write_to<W: Write>(w: &mut W, input: &[Transaction]) -> WriterResult<()> {
        for tx in input {
            tx.write_bin(w)?;
        }
        w.flush()?;

        Ok(())
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = ReaderError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> ReaderResult<[u8; N]> {
            let slice = bytes
                .get(*cursor..*cursor + N)
                .ok_or(ReaderError::Transaction)?;
            *cursor += N;
            Ok(slice.try_into()?)
        }

        let tx_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let tx_type = *bytes.get(cursor).ok_or(ReaderError::Transaction)?;
        cursor += 1;
        let tx_type = tx_type.try_into()?;
        let from_user_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let to_user_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let amount = i64::from_be_bytes(take(bytes, &mut cursor)?);
        let timestamp = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let status = *bytes.get(cursor).ok_or(ReaderError::Transaction)?;
        cursor += 1;
        let status = status.try_into()?;
        let desc_len = u32::from_be_bytes(take(bytes, &mut cursor)?) as usize;
        let description = if desc_len == 0 {
            None
        } else {
            let desc_bytes = bytes
                .get(cursor..cursor + desc_len)
                .ok_or(ReaderError::Transaction)?;
            cursor += desc_len;
            Some(str::from_utf8(desc_bytes)?.trim_matches('"').to_string())
        };

        if cursor != bytes.len() {
            return Err(ReaderError::Transaction);
        }

        Ok(Self {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        })
    }
}

impl TryFrom<u8> for TxType {
    type Error = ReaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Withdrawal),
            _ => Err(ReaderError::TxType),
        }
    }
}

impl From<TxType> for u8 {
    fn from(value: TxType) -> Self {
        match value {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        }
    }
}

impl TryFrom<u8> for TxStatus {
    type Error = ReaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(ReaderError::TxStatus),
        }
    }
}

impl From<TxStatus> for u8 {
    fn from(value: TxStatus) -> Self {
        match value {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        }
    }
}

impl Transaction {
    fn write_bin(&self, w: &mut impl Write) -> WriterResult<()> {
        let mut p = Vec::new();
        p.extend_from_slice(&self.tx_id.to_be_bytes());
        p.push(self.tx_type.into());
        p.extend_from_slice(&self.from_user_id.to_be_bytes());
        p.extend_from_slice(&self.to_user_id.to_be_bytes());
        p.extend_from_slice(&self.amount.to_be_bytes());
        p.extend_from_slice(&self.timestamp.to_be_bytes());
        p.push(self.status.into());

        let description = self.description.as_deref().unwrap_or_default().as_bytes();
        p.extend_from_slice(&(description.len() as u32).to_be_bytes());
        p.extend_from_slice(description);

        w.write_all(MAGIC)?;
        w.write_all(&(p.len() as u32).to_be_bytes())?;
        w.write_all(&p)?;

        Ok(())
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

    fn write_bytes(input: &[Transaction]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        BinaryParser::write_to(&mut cursor, input).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn parses_multiple_binary_transactions() {
        let first = tx(
            42,
            TxType::Deposit,
            0,
            101,
            5_000,
            1_700_000_000,
            TxStatus::Success,
            Some("initial deposit"),
        );
        let second = tx(
            43,
            TxType::Transfer,
            101,
            202,
            -250,
            1_700_000_100,
            TxStatus::Pending,
            None,
        );

        let bytes = write_bytes(&[first, second]);
        let mut cursor = Cursor::new(bytes);

        let parsed = BinaryParser::from_read(&mut cursor).unwrap();

        assert_eq!(parsed.len(), 2);

        let first = &parsed[0];
        assert_eq!(first.tx_id, 42);
        assert!(matches!(first.tx_type, TxType::Deposit));
        assert_eq!(first.from_user_id, 0);
        assert_eq!(first.to_user_id, 101);
        assert_eq!(first.amount, 5_000);
        assert_eq!(first.timestamp, 1_700_000_000);
        assert!(matches!(first.status, TxStatus::Success));
        assert_eq!(first.description.as_deref(), Some("initial deposit"));

        let second = &parsed[1];
        assert_eq!(second.tx_id, 43);
        assert!(matches!(second.tx_type, TxType::Transfer));
        assert_eq!(second.from_user_id, 101);
        assert_eq!(second.to_user_id, 202);
        assert_eq!(second.amount, -250);
        assert_eq!(second.timestamp, 1_700_000_100);
        assert!(matches!(second.status, TxStatus::Pending));
        assert_eq!(second.description, None);
    }

    #[test]
    fn rejects_invalid_record_magic() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Deposit,
            0,
            1,
            10,
            100,
            TxStatus::Success,
            None,
        )]);
        bytes[..4].copy_from_slice(b"NOPE");
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::Transaction));
    }

    #[test]
    fn rejects_truncated_payload() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Withdrawal,
            50,
            0,
            -10,
            100,
            TxStatus::Failure,
            Some("atm"),
        )]);
        bytes.pop();
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::Io(_)));
    }

    #[test]
    fn rejects_truncated_record_magic_at_end_of_stream() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Deposit,
            0,
            1,
            10,
            100,
            TxStatus::Success,
            None,
        )]);
        bytes.extend_from_slice(b"XYZ");
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::Io(_)));
    }

    #[test]
    fn rejects_invalid_transaction_type() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Withdrawal,
            50,
            0,
            -10,
            100,
            TxStatus::Failure,
            None,
        )]);
        bytes[16] = 9;
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::TxType));
    }

    #[test]
    fn rejects_invalid_status() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Transfer,
            50,
            60,
            10,
            100,
            TxStatus::Success,
            None,
        )]);
        bytes[49] = 9;
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::TxStatus));
    }

    #[test]
    fn rejects_trailing_bytes_in_transaction_payload() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Transfer,
            50,
            60,
            10,
            100,
            TxStatus::Success,
            None,
        )]);
        let len = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let new_len = len + 1;
        bytes[4..8].copy_from_slice(&new_len.to_be_bytes());
        bytes.push(0xff);
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::Transaction));
    }

    #[test]
    fn rejects_invalid_utf8_description() {
        let mut bytes = write_bytes(&[tx(
            1,
            TxType::Deposit,
            0,
            60,
            10,
            100,
            TxStatus::Success,
            Some("ok"),
        )]);
        bytes[50..54].copy_from_slice(&(2u32).to_be_bytes());
        bytes[54..56].copy_from_slice(&[0xff, 0xfe]);
        let mut cursor = Cursor::new(bytes);

        let err = BinaryParser::from_read(&mut cursor).unwrap_err();

        assert!(matches!(err, ReaderError::Utf(_)));
    }

    #[test]
    fn write_to_outputs_bytes_that_from_read_can_parse() {
        let input = [
            tx(
                7,
                TxType::Deposit,
                0,
                33,
                2500,
                1_700_000_001,
                TxStatus::Success,
                Some("paycheck"),
            ),
            tx(
                8,
                TxType::Withdrawal,
                33,
                0,
                -400,
                1_700_000_002,
                TxStatus::Pending,
                None,
            ),
        ];

        let bytes = write_bytes(&input);
        let parsed = BinaryParser::from_read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(parsed.len(), input.len());
        assert_eq!(parsed[0].tx_id, 7);
        assert!(matches!(parsed[0].tx_type, TxType::Deposit));
        assert_eq!(parsed[0].description.as_deref(), Some("paycheck"));
        assert_eq!(parsed[1].tx_id, 8);
        assert!(matches!(parsed[1].tx_type, TxType::Withdrawal));
        assert!(matches!(parsed[1].status, TxStatus::Pending));
        assert_eq!(parsed[1].description, None);
    }
}
