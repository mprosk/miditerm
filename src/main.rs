pub mod midi;
// mod ui;

use crate::midi::MidiParser;
use anyhow::Context;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    ///
    #[structopt(long, parse(from_os_str))]
    file: Option<PathBuf>,

    /// Name or path of the serial device to open
    #[structopt(long)]
    port: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::from_args();
    println!("{:?}", args);
    if let Some(filepath) = args.file {
        return read_from_file(filepath).context("Error parsing MIDI from file");
    } else if let Some(port) = args.port {
        return read_from_serial(port).context("Error parsing MIDI from serial port");
    }

    // ui::run_application()?;

    Ok(())
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

fn read_from_serial(port: String) -> Result<(), anyhow::Error> {
    let mut parser = MidiParser::new();
    let mut serial = serialport::new(port.clone(), midi::MIDI_BAUD_RATE)
        .open()
        .context(format!("Unable to open serial port `{}`", port))?;
    let mut buffer = [0_u8; 1];
    loop {
        let _ = serial
            .read(&mut buffer)
            .context("Error reading from serial port")?;
        display_midi(&mut parser, buffer[0]);
    }
}

fn display_midi(parser: &mut MidiParser, byte: u8) {
    print!("{:02X} ", byte);
    let (_message, analysis) = parser.parse_midi(byte);
    println!("{:?}", analysis);
}
