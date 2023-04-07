# miditerm
MidiTerm is a Rust-based MIDI debugger and protocol analyzer. It is intended to assist those developing their own MIDI devices by providing low-level decoding of the MIDI data stream. Tools like MIDI-OX and others are useful, but rely on class-compliant USB MIDI adapters which may be hiding MIDI features like running status, System Real Time messages in the middle of other messages. These nuances of the protocol are important to account for when developing drivers for your MIDI device.

Check out [miditerm-adapter](https://github.com/mprosk/miditer-adapter) for a way to connect DIN MIDI devices as a USB serial device

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
