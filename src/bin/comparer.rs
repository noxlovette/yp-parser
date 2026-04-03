use clap::Parser as ClapParser;
use std::{collections::HashSet, fs::File, io::BufReader};
use yp_parser::{BinaryParser, CsvParser, Format, Parser, TextParser};

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

    let set1: HashSet<_> = decoded1.into_iter().collect();
    let set2: HashSet<_> = decoded2.into_iter().collect();
    let diff: HashSet<_> = set1.symmetric_difference(&set2).collect();
    if diff.len() == 0 {
        println!(
            "The items in files {} and {} are identical.",
            args.file1, args.file2
        )
    } else {
        println!("The following transactions differ in given files:\n");
        for tx in &diff {
            println!("{tx}");
        }
        println!("Number of differing items: {}", diff.len())
    }

    Ok(())
}
