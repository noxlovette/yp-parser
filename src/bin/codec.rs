use clap::{Parser as ClapParser, ValueEnum};
use std::fs::File;
use yp_parser::{BinaryParser, Format, Parser};

#[derive(Debug, Clone, ValueEnum)]
enum Codec {
    Read,
    Write,
}

#[derive(ClapParser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, value_enum)]
    format: Format,

    #[arg(short, value_enum)]
    codec: Codec,

    path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.path)?;

    match args.codec {
        Codec::Read => {
            let output = match args.format {
                Format::Binary => BinaryParser::from_read(&mut file)?,
                _ => todo!(),
            };

            println!("TRANSACTIONS\n");
            for tx in output {
                println!("{tx}");
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
