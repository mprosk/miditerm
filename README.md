# MidiTerm
MidiTerm is a Rust-based MIDI debugger and protocol analyzer. It is intended to assist those developing their own MIDI devices by providing direct decoding of the raw MIDI data stream. Tools like MIDI-OX and others are useful, but rely on class-compliant USB MIDI adapters and operating system drivers which may be obscuring low-level MIDI protocol features like running status, System Real Time messages in the middle of other messages, and Note On with velocity 0 equals Note Off. These nuances of the MIDI protocol are important to account for when implementing a MIDI interface in your own device. MidiTerm bypasses these software layers by decoding the raw bytes of the MIDI stream directly, providing you with an unobstructed view of what's really happening.

Check out [miditerm-adapter](https://github.com/mprosk/miditerm-adapter) for a way to connect DIN MIDI devices as a USB serial device

## Current Features
- Fully MIDI 1.0 compliant
- Display of all bytes in the order they are received
- Decoding of MIDI messages
- Use of a serial port as a MIDI device

## Future Features
- MIDI transmission
  - Keyboard piano
  - Optional echo/pass through
  - Automated test suite
- Nicer TUI
  - Filter out message types (e.g. active sense, timing clock)
  - Display TX and RX simultaneously
- Extended SysEx features
  - Universal SysEx messages
  - Automatic manufacturer identification
  - SysEx message builder
- SysEx terminal

### Future Future Features
- MIDI Time Code
- MIDI Show Control
- MIDI Machine Control
- MIDI 2.0???
