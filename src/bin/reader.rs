use clap::Parser as ClapParser;
use std::{fs::File, io::BufReader};
use yp_parser::{BinaryParser, CsvParser, Format, Parser, TextParser};

#[derive(ClapParser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(value_enum)]
    format: Format,

    path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = File::open(args.path)?;
    let mut buf = BufReader::new(file);

    let output = match args.format {
        Format::Binary => BinaryParser::from_read(&mut buf)?,
        Format::Txt => TextParser::from_read(&mut buf)?,
        Format::Csv => CsvParser::from_read(&mut buf)?,
    };

    println!("TRANSACTIONS\n");
    for tx in output {
        println!("{tx}");
    }

    Ok(())
}
