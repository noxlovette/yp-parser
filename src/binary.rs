use crate::{Format, ParserError, ParserResult, Transaction, TxStatus, TxType};
use std::io::{BufReader, Read};

const MAGIC: &[u8] = b"YPBN";

pub enum BinState {
    Magic,
    Length,
    Payload { len: usize },
    Done,
}

impl BinState {
    pub(crate) fn from_read<R: Read>(r: &mut R) -> ParserResult<Vec<Transaction>> {
        let mut output = Vec::new();
        let mut reader = BufReader::new(r);
        let mut buf = [0u8; 1024];
        let mut buf_len = 0;
        let mut state = Self::Magic;

        loop {
            let n = reader.read(&mut buf[buf_len..])?;
            let mut tx = Transaction::new();

            if n == 0 {
                // EOF
                break;
            }
            buf_len += n;
            let read = state.parse(&buf[buf_len..], &mut tx)?;
            output.push(tx);

            buf.copy_within(read..buf_len, 0);
            buf_len -= read;
        }

        Ok(output)
    }
    fn parse(&mut self, data: &[u8], tx: &mut Transaction) -> ParserResult<usize> {
        use BinState::*;
        let mut read = 0;

        while !matches!(self, Done) {
            let current_data = &data[read..];
            match self {
                Magic => {
                    let n = self.magic(current_data)?;
                    if n == 0 {
                        break;
                    }
                    read += n;
                }
                Length => {
                    let n = self.length(current_data)?;
                    if n == 0 {
                        break;
                    }
                    read += n;
                }
                Payload { len } => {
                    let (n, result) = Self::payload(current_data, *len)?;
                    if n == 0 {
                        break;
                    }
                    read += n;
                    if let Some(result) = result {
                        *tx = result;
                    } else {
                        return Err(ParserError::Transaction);
                    }
                }
                Done => break,
            }
        }

        Ok(read)
    }

    fn magic(&mut self, data: &[u8]) -> ParserResult<usize> {
        if let Some(i) = data.windows(MAGIC.len()).position(|b| b == MAGIC) {
            *self = BinState::Length;
            Ok(i)
        } else {
            Ok(0)
        }
    }

    fn length(&mut self, data: &[u8]) -> ParserResult<usize> {
        if data.len().ge(&4) {
            let slice: [u8; 4] = data[..4].try_into()?;
            *self = BinState::Payload {
                len: u32::from_be_bytes(slice) as usize,
            };
            Ok(4)
        } else {
            Ok(0)
        }
    }

    fn payload(data: &[u8], target: usize) -> ParserResult<(usize, Option<Transaction>)> {
        if data.len() != target {
            return Ok((0, None));
        }

        Ok((data.len(), Some(data.try_into()?)))
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = ParserError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        let tx_id = u64::from_be_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;
        let tx_type: TxType = bytes[cursor].try_into()?;
        cursor += 1;
        let from_user_id = u64::from_be_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;
        let to_user_id = u64::from_be_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;
        let amount = i64::from_be_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;
        let timestamp = u64::from_be_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;
        let status: TxStatus = bytes[cursor].try_into()?;
        cursor += 1;
        let desc_len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into()?);
        cursor += 4;
        let description = if desc_len != 0 {
            Some(str::from_utf8(&bytes[cursor..cursor + desc_len as usize])?.to_string())
        } else {
            None
        };

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
