pub mod midi;

use crate::midi::MidiParser;
use anyhow::Context;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    ///
    #[structopt(long, parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::from_args();
    println!("{:?}", args);
    if let Some(filepath) = args.file {
        if let Err(e) = read_from_file(filepath) {
            println!("Error parsing MIDI from file: {:?}", e);
        }
    }
}

fn read_from_file(filepath: PathBuf) -> Result<(), anyhow::Error> {
    let file =
        File::open(filepath.clone()).context(format!("Unable to open file `{:?}`", filepath))?;
    let reader = BufReader::new(file);
    let mut parser = MidiParser::new();
    for b in reader.bytes() {
        match b {
            Ok(byte) => {
                display_midi(&mut parser, byte);
            }
            Err(e) => {
                println!("IO Error while reading from file: {:?}", e);
            }
        }
    }
    println!("End of file");
    Ok(())
}

fn display_midi(parser: &mut MidiParser, byte: u8) {
    print!("{:02X} ", byte);
    if let Some(midi) = parser.parse_midi(byte) {
        println!("{:?}", midi)
    }
}
