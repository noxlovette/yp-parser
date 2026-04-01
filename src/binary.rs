use crate::{ParserError, ParserResult, Transaction, TxStatus, TxType};
use std::io::{BufReader, Read};

/// Parses the .bin format
pub struct BinaryParser;

impl BinaryParser {
    pub fn from_read<R: Read>(r: &mut R) -> ParserResult<Vec<Transaction>> {
        const MAGIC: [u8; 4] = *b"YPBN";
        let mut reader = BufReader::new(r);
        let mut output = Vec::new();

        loop {
            let mut magic = [0u8; MAGIC.len()];
            match reader.read_exact(&mut magic) {
                Ok(()) => {}
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    } else {
                        return Err(err.into());
                    }
                }
            }

            if magic != MAGIC {
                return Err(ParserError::Transaction);
            }

            let mut len = [0u8; 4];
            reader.read_exact(&mut len)?;
            let payload_len = u32::from_be_bytes(len) as usize;

            let mut payload = vec![0u8; payload_len];
            reader.read_exact(&mut payload)?;
            output.push(payload.as_slice().try_into()?);
        }

        Ok(output)
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = ParserError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> ParserResult<[u8; N]> {
            let slice = bytes
                .get(*cursor..*cursor + N)
                .ok_or(ParserError::Transaction)?;
            *cursor += N;
            Ok(slice.try_into()?)
        }

        let tx_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let tx_type = *bytes.get(cursor).ok_or(ParserError::Transaction)?;
        cursor += 1;
        let tx_type = tx_type.try_into()?;
        let from_user_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let to_user_id = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let amount = i64::from_be_bytes(take(bytes, &mut cursor)?);
        let timestamp = u64::from_be_bytes(take(bytes, &mut cursor)?);
        let status = *bytes.get(cursor).ok_or(ParserError::Transaction)?;
        cursor += 1;
        let status = status.try_into()?;
        let desc_len = u32::from_be_bytes(take(bytes, &mut cursor)?) as usize;
        let description = if desc_len == 0 {
            None
        } else {
            let desc_bytes = bytes
                .get(cursor..cursor + desc_len)
                .ok_or(ParserError::Transaction)?;
            cursor += desc_len;
            Some(str::from_utf8(desc_bytes)?.to_string())
        };

        if cursor != bytes.len() {
            return Err(ParserError::Transaction);
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
    type Error = ParserError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Withdrawal),
            _ => Err(ParserError::TxType),
        }
    }
}

impl TryFrom<u8> for TxStatus {
    type Error = ParserError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(ParserError::TxStatus),
        }
    }
}
