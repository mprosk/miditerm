pub mod controls;
pub mod sysex;

// Bit masks
const MIDI_BYTE_TYPE_MASK: u8 = 0b_1000_0000_u8;
const MIDI_CHANNEL_MASK: u8 = 0b_0000_1111_u8;
const MIDI_STATUS_MASK: u8 = 0b_1111_0000_u8;

// Channel Messages
const MIDI_MSG_NOTE_OFF: u8 = 0x80_u8;
const MIDI_MSG_NOTE_ON: u8 = 0x90_u8;
const MIDI_MSG_POLY_PRESSURE: u8 = 0xA0_u8;
const MIDI_MSG_CONTROL_CHANGE: u8 = 0xB0_u8;
const MIDI_MSG_PROGRAM_CHANGE: u8 = 0xC0_u8;
const MIDI_MSG_CHANNEL_PRESSURE: u8 = 0xD0_u8;
const MIDI_MSG_PITCH_BEND: u8 = 0xE0_u8;

// System Exclusive Messages
const MIDI_SYSEX_SOX: u8 = 0xF0_u8;
const MIDI_SYSEX_EOX: u8 = 0xF7_u8;

// System Common Messages
const MIDI_SYSCOM_MTC_FRAME: u8 = 0xF1_u8;
const MIDI_SYSCOM_SONG_POSITION: u8 = 0xF2_u8;
const MIDI_SYSCOM_SONG_SELECT: u8 = 0xF3_u8;
const MIDI_SYSCOM_TUNE_REQUEST: u8 = 0xF6_u8;

// System Real Time Messages
const MIDI_SYSRT_TIMING_CLOCK: u8 = 0xF8_u8;
const MIDI_SYSRT_START: u8 = 0xFA_u8;
const MIDI_SYSRT_CONTINUE: u8 = 0xFB_u8;
const MIDI_SYSRT_STOP: u8 = 0xFC_u8;
const MIDI_SYSRT_ACTIVE_SENSE: u8 = 0xFE_u8;
const MIDI_SYSRT_SYSTEM_RESET: u8 = 0xFF_u8;

#[derive(Debug)]
pub enum MidiChannelMode {
    AllSoundOff,
    ResetAllControllers,
    LocalControl(bool),
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn,
    PolyModeOn,
}

#[derive(Debug)]
pub enum MidiMessage {
    // Channel Messages
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    PolyPressure {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        control: u8,
        value: u8,
    },
    ChannelMode {
        channel: u8,
        mode: MidiChannelMode,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },

    // System Common
    MtcQuarterFrame(u8),
    SongPosition(u16),
    SongSelect(u8),
    TuneRequest,

    // System Real Time
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,

    // System Exclusive
    SystemExclusive(Vec<u8>),

    /// Undefined status message
    Undefined(u8),
    /// Data byte that is not associated with a status message
    OrphanedData(u8),
    ///
    ProtocolError(String),
}

pub struct MidiParser {
    status: Option<u8>,
    d0: Option<u8>,
    channel: u8,
    sysex: Vec<u8>,
}

impl MidiParser {
    /// Creates a new instance of `MidiParser`
    pub fn new() -> MidiParser {
        MidiParser {
            status: None,
            d0: None,
            channel: 0xFF,
            sysex: vec![],
        }
    }

    /// Accepts the given byte and outputs `Some(MidiMessage)` if the preceding byte sequences parsed into a MIDI message
    ///
    /// Returns `None` if the byte did not complete a MIDI message
    pub fn parse_midi(&mut self, byte: u8) -> Option<MidiMessage> {
        if (byte & MIDI_BYTE_TYPE_MASK) != 0 {
            if (byte & MIDI_STATUS_MASK) == 0xF0 {
                // System Message
                return self.parse_system_message(byte);
            } else {
                // Channel Message
                self.channel = byte & MIDI_CHANNEL_MASK;
                self.set_state(byte & MIDI_STATUS_MASK);
            }
        } else {
            // Data byte
            return self.parse_data_byte(byte);
        }
        None
    }

    /// Parses the given System Message byte
    fn parse_system_message(&mut self, byte: u8) -> Option<MidiMessage> {
        match byte {
            // System Common Message - clear running status
            MIDI_SYSCOM_MTC_FRAME => {
                self.set_state(MIDI_SYSCOM_MTC_FRAME);
            }
            MIDI_SYSCOM_SONG_POSITION => {
                self.set_state(MIDI_SYSCOM_SONG_POSITION);
            }
            MIDI_SYSCOM_SONG_SELECT => {
                self.set_state(MIDI_SYSCOM_SONG_SELECT);
            }
            MIDI_SYSCOM_TUNE_REQUEST => {
                self.clear_state();
                return Some(MidiMessage::TuneRequest);
            }

            // System Real Time Message - no effect to running status
            MIDI_SYSRT_TIMING_CLOCK => return Some(MidiMessage::TimingClock),
            MIDI_SYSRT_START => return Some(MidiMessage::Start),
            MIDI_SYSRT_CONTINUE => return Some(MidiMessage::Continue),
            MIDI_SYSRT_STOP => return Some(MidiMessage::Stop),
            MIDI_SYSRT_ACTIVE_SENSE => return Some(MidiMessage::ActiveSensing),
            MIDI_SYSRT_SYSTEM_RESET => return Some(MidiMessage::SystemReset),

            // System Exclusive Message
            MIDI_SYSEX_SOX => {
                self.set_state(MIDI_SYSEX_SOX);
                self.sysex = vec![];
            }
            MIDI_SYSEX_EOX => {
                if self.status != Some(MIDI_SYSEX_SOX) {
                    return Some(MidiMessage::ProtocolError("Received `End of Exclusive` while not within a System Exclusive sequence".to_string()))
                }
                self.clear_state();
                return Some(MidiMessage::SystemExclusive(self.sysex.clone()))
            }

            // Undefined System Message - no effect to running status
            _ => return Some(MidiMessage::Undefined(byte)),
        }
        None
    }

    /// Parses the given data byte
    fn parse_data_byte(&mut self, byte: u8) -> Option<MidiMessage> {
        if let Some(state) = self.status {
            match state {
                // Channel Messages
                MIDI_MSG_NOTE_OFF => {
                    if let Some(note) = self.d0 {
                        return Some(MidiMessage::NoteOff {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        });
                    }
                }
                MIDI_MSG_NOTE_ON => {
                    if let Some(note) = self.d0 {
                        return Some(MidiMessage::NoteOn {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        });
                    }
                }
                MIDI_MSG_POLY_PRESSURE => {
                    if let Some(note) = self.d0 {
                        return Some(MidiMessage::PolyPressure {
                            channel: self.channel,
                            note,
                            pressure: byte,
                        });
                    }
                }
                MIDI_MSG_CONTROL_CHANGE => {
                    if let Some(control) = self.d0 {
                        return match control {
                            120 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::AllSoundOff,
                            }),
                            121 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::ResetAllControllers,
                            }),
                            122 => {
                                if byte != 0 || byte != 127 {
                                    Some(MidiMessage::ProtocolError(format!("Invalid data value for Channel Mode 122 Local Control. Got {}, expected to be 0 (local control off) or 127 (local control on)", byte)))
                                } else {
                                    Some(MidiMessage::ChannelMode {
                                        channel: self.channel,
                                        mode: MidiChannelMode::LocalControl(byte == 127),
                                    })
                                }
                            }
                            123 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::AllNotesOff,
                            }),
                            124 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::OmniModeOff,
                            }),
                            125 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::OmniModeOn,
                            }),
                            126 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::MonoModeOn,
                            }),
                            127 => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::PolyModeOn,
                            }),
                            _ => Some(MidiMessage::ControlChange {
                                channel: self.channel,
                                control,
                                value: byte,
                            }),
                        };
                    }
                }
                MIDI_MSG_PROGRAM_CHANGE => {
                    return Some(MidiMessage::ProgramChange {
                        channel: self.channel,
                        program: byte,
                    });
                }
                MIDI_MSG_CHANNEL_PRESSURE => {
                    return Some(MidiMessage::ChannelPressure {
                        channel: self.channel,
                        pressure: byte,
                    });
                }
                MIDI_MSG_PITCH_BEND => {
                    if let Some(lsb) = self.d0 {
                        self.clear_state();
                        let bend = ((byte as u16) << 7) | (lsb as u16);
                        return Some(MidiMessage::PitchBend {
                            channel: self.channel,
                            value: bend,
                        });
                    }
                }

                // System Common
                MIDI_SYSCOM_MTC_FRAME => {
                    self.clear_state();
                    return Some(MidiMessage::MtcQuarterFrame(byte));
                }
                MIDI_SYSCOM_SONG_POSITION => {
                    if let Some(lsb) = self.d0 {
                        self.clear_state();
                        let spp = ((byte as u16) << 7) | (lsb as u16);
                        return Some(MidiMessage::SongPosition(spp));
                    }
                }
                MIDI_SYSCOM_SONG_SELECT => {
                    self.clear_state();
                    return Some(MidiMessage::SongSelect(byte));
                }

                // System Exclusive
                MIDI_SYSEX_SOX => {
                    self.sysex.push(byte);
                }

                // Base case - this shouldn't happen
                _ => {
                    panic!("Got data byte 0x{:2X} while in state 0x{:2x}", byte, state);
                }
            }
        } else {
            return Some(MidiMessage::OrphanedData(byte));
        }
        self.d0 = Some(byte);
        None
    }

    /// Set the internal state to a given status message type and clear the data buffer
    fn set_state(&mut self, state: u8) {
        self.status = Some(state);
        self.d0 = None;
    }

    /// Clear the internal state status message type and clear the data buffer
    fn clear_state(&mut self) {
        self.status = None;
        self.d0 = None;
    }
}
