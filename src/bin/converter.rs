use clap::Parser as ClapParser;
use std::{fs::File, io::BufReader};
use yp_parser::{BinaryParser, CsvParser, Format, Parser, TextParser};

#[derive(Debug, ClapParser)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    input: String,

    #[arg(long)]
    input_format: Format,

    #[arg(long)]
    output_format: Format,
}

fn main() -> anyhow::Result<()> {
    use Format::*;
    let args = Args::parse();
    let file = File::open(args.input)?;
    let mut buf = BufReader::new(file);

    let decoded = match args.input_format {
        Binary => BinaryParser::from_read(&mut buf)?,
        Txt => TextParser::from_read(&mut buf)?,
        Csv => CsvParser::from_read(&mut buf)?,
    };

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    match args.output_format {
        Binary => BinaryParser::write_to(&mut handle, &decoded)?,
        Txt => TextParser::write_to(&mut handle, &decoded)?,
        Csv => CsvParser::write_to(&mut handle, &decoded)?,
    }

    Ok(())
}
