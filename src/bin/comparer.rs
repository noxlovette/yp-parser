use clap::Parser as ClapParser;
use std::{collections::HashMap, fs::File, io::BufReader};
use yp_parser::{BinaryParser, CsvParser, Format, Parser, TextParser, Transaction};

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(long)]
    file1: String,
    #[arg(long)]
    format1: Format,
    #[arg(long)]
    file2: String,
    #[arg(long)]
    format2: Format,
}

fn main() -> anyhow::Result<()> {
    use Format::*;
    let args = Args::parse();

    let file1 = File::open(&args.file1)?;
    let mut buf1 = BufReader::new(file1);

    let file2 = File::open(&args.file2)?;
    let mut buf2 = BufReader::new(file2);

    let decoded1 = match args.format1 {
        Binary => BinaryParser::from_read(&mut buf1)?,
        Txt => TextParser::from_read(&mut buf1)?,
        Csv => CsvParser::from_read(&mut buf1)?,
    };

    let decoded2 = match args.format2 {
        Binary => BinaryParser::from_read(&mut buf2)?,
        Txt => TextParser::from_read(&mut buf2)?,
        Csv => CsvParser::from_read(&mut buf2)?,
    };

    let counts1 = transaction_counts(decoded1);
    let counts2 = transaction_counts(decoded2);
    let diff = transaction_differences(&counts1, &counts2);
    if diff.is_empty() {
        println!(
            "The items in files {} and {} are identical.",
            args.file1, args.file2
        )
    } else {
        println!("The following transactions differ in given files:\n");
        for (tx, count1, count2) in &diff {
            println!("{tx}");
            println!("COUNT_IN_FILE1: {count1}");
            println!("COUNT_IN_FILE2: {count2}\n");
        }
        println!("Number of differing items: {}", diff.len())
    }

    Ok(())
}

fn transaction_counts(transactions: Vec<Transaction>) -> HashMap<Transaction, usize> {
    let mut counts = HashMap::new();
    for tx in transactions {
        *counts.entry(tx).or_insert(0) += 1;
    }
    counts
}

fn transaction_differences(
    counts1: &HashMap<Transaction, usize>,
    counts2: &HashMap<Transaction, usize>,
) -> Vec<(Transaction, usize, usize)> {
    let mut diff = Vec::new();

    for (tx, count1) in counts1 {
        let count2 = counts2.get(tx).copied().unwrap_or(0);
        if *count1 != count2 {
            diff.push((tx.clone(), *count1, count2));
        }
    }

    for (tx, count2) in counts2 {
        if !counts1.contains_key(tx) {
            diff.push((tx.clone(), 0, *count2));
        }
    }

    diff
}
