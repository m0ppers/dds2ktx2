use std::error::Error;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use structopt::StructOpt;

mod converter;
mod ktx2;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

fn run(opt: &Opt) -> Result<(), Box<dyn Error>> {
    let file = File::open(&opt.input)?;
    let mut reader = BufReader::new(file);

    let file = File::create(&opt.output)?;
    let mut writer = BufWriter::new(file);

    converter::convert(&mut reader, &mut writer)
}

fn main() {
    let opt = Opt::from_args();

    std::process::exit(match run(&opt) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
