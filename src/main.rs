use crate::midi::MidiMessage;

pub mod midi;

fn main() {
    let mut parser = midi::MidiParser::new();
    for tv in 0xF8..=0xFF {
        println!("{:2X}: {:?}", tv, parser.parse_midi(tv));
    }
    println!(
        "{:?}",
        MidiMessage::SystemExclusive(vec![1, 2, 3, 4]).to_bytes()
    )
}
