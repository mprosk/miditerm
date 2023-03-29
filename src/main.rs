mod midi;

fn main() {
    let mut parser = midi::MidiParser::new();
    for tv in 0xF8..=0xFF {
        println!("{:2X}: {:?}", tv, parser.parse_midi(tv));
    }
}
