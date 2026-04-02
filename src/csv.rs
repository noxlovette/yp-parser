use crate::{Parser, ReaderError, ReaderResult, Transaction, TransactionPartial};
pub struct CsvParser;

const HEADER: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

impl Parser for CsvParser {
    fn from_read<R: std::io::Read>(r: &mut R) -> crate::ReaderResult<Vec<crate::Transaction>> {
        let mut buf = String::new();
        let mut output: Vec<Transaction> = Vec::new();

        r.read_to_string(&mut buf)?;

        let mut iter = buf.lines();

        if iter.next().is_some_and(|l| l == HEADER) {
            for line in iter.filter(|l| !l.is_empty()) {
                let fields = line.split(',');
                output.push(Transaction::from_fields(fields.enumerate())?)
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
        todo!()
    }
}

impl Transaction {
    fn from_fields<'a, I>(mut fields: I) -> ReaderResult<Self>
    where
        I: Iterator<Item = (usize, &'a str)>,
    {
        let mut tx = TransactionPartial::default();
        while let Some((i, f)) = fields.next() {
            match i {
                0 => {
                    tx.tx_id(f.parse()?);
                }
                1 => {
                    tx.tx_type(f.parse()?);
                }
                2 => {
                    tx.from_user_id(f.parse()?);
                }
                3 => {
                    tx.to_user_id(f.parse()?);
                }
                4 => {
                    tx.amount(f.parse()?);
                }
                5 => {
                    tx.timestamp(f.parse()?);
                }
                6 => {
                    tx.status(f.parse()?);
                }
                7 => {
                    tx.description(Some(f.to_string()));
                }
                _ => return Err(ReaderError::CsvCorrupt),
            }
        }

        tx.try_into()
    }
}
