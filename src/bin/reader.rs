use clap::Parser;
use std::fs::File;
use yp_parser::{BinaryParser, Format};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(value_enum)]
    format: Format,

    #[arg(short)]
    path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.path)?;

    let output = match args.format {
        Format::Binary => BinaryParser::from_read(&mut file)?,
        _ => todo!(),
    };

    println!("TRANSACTIONS\n");
    for tx in output {
        println!("{tx}");
    }

    Ok(())
}
