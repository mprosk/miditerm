//! Implementation of the MIDI parser

use crate::midi::*;

impl Default for MidiParser {
    fn default() -> Self {
        Self::new()
    }
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

    /// Returns current running status
    pub fn get_state(&mut self) -> Option<u8> {
        self.status
    }

    /// Returns the name of the current running status
    pub fn get_state_name(&mut self) -> String {
        if let Some(state) = self.status {
            return match state {
                MIDI_MSG_NOTE_OFF => "Note Off".to_string(),
                MIDI_MSG_NOTE_ON => "Note On".to_string(),
                MIDI_MSG_POLY_PRESSURE => "Poly Pressure".to_string(),
                MIDI_MSG_CONTROL_CHANGE => "Control Change".to_string(),
                MIDI_MSG_PROGRAM_CHANGE => "Program Change".to_string(),
                MIDI_MSG_CHANNEL_PRESSURE => "Channel Pressure".to_string(),
                MIDI_MSG_PITCH_BEND => "Pitch Bend".to_string(),
                MIDI_SYSEX_SOX => "System Exclusive".to_string(),
                MIDI_SYSCOM_MTC_FRAME => "MTC Frame".to_string(),
                MIDI_SYSCOM_SONG_POSITION => "Song Position".to_string(),
                MIDI_SYSCOM_SONG_SELECT => "Song Select".to_string(),
                _ => format!("UNKNOWN: {:02X}", state),
            };
        }
        "NONE".to_string()
    }

    /// Clears the internal data buffer
    fn clear_data(&mut self) {
        self.d0 = None;
    }

    /// Accepts the given byte and outputs `Some(MidiMessage)`
    /// if the preceding byte sequences parsed into a MIDI message
    ///
    /// Returns `None` if the byte did not complete a MIDI message
    pub fn parse_midi(&mut self, byte: u8) -> (Option<MidiMessage>, MidiAnalysis) {
        if (byte & MIDI_BYTE_TYPE_MASK) != 0 {
            if (byte & MIDI_STATUS_MASK) == 0xF0 {
                // System Message
                self.parse_system_message(byte)
            } else {
                // Channel Message
                self.parse_channel_message(byte)
            }
        } else {
            // Data byte
            self.parse_data_byte(byte)
        }
    }

    /// Parses the given channel message byte
    fn parse_channel_message(&mut self, byte: u8) -> (Option<MidiMessage>, MidiAnalysis) {
        self.channel = byte & MIDI_CHANNEL_MASK;
        let status = byte & MIDI_STATUS_MASK;
        self.set_state(status);
        match status {
            MIDI_MSG_NOTE_OFF => (
                None,
                MidiAnalysis::Comment(format!("Note Off (Channel {})", self.channel)),
            ),
            MIDI_MSG_NOTE_ON => (
                None,
                MidiAnalysis::Comment(format!("Note On (Channel {})", self.channel)),
            ),
            MIDI_MSG_POLY_PRESSURE => (
                None,
                MidiAnalysis::Comment(format!("Poly Pressure (Channel {})", self.channel)),
            ),
            MIDI_MSG_CONTROL_CHANGE => (
                None,
                MidiAnalysis::Comment(format!("Control Change (Channel {})", self.channel)),
            ),
            MIDI_MSG_PROGRAM_CHANGE => (
                None,
                MidiAnalysis::Comment(format!("Program Change (Channel {})", self.channel)),
            ),
            MIDI_MSG_CHANNEL_PRESSURE => (
                None,
                MidiAnalysis::Comment(format!("Channel Pressure (Channel {})", self.channel)),
            ),
            MIDI_MSG_PITCH_BEND => (
                None,
                MidiAnalysis::Comment(format!("Pitch Bend (Channel {})", self.channel)),
            ),
            // This should never happen
            _ => panic!("{:02X} is not a channel message byte", status),
        }
    }

    /// Parses the given System Message byte
    fn parse_system_message(&mut self, byte: u8) -> (Option<MidiMessage>, MidiAnalysis) {
        match byte {
            // System Common Message - clear running status
            MIDI_SYSCOM_MTC_FRAME => {
                self.set_state(MIDI_SYSCOM_MTC_FRAME);
                (None, MidiAnalysis::Comment("MTC Frame".to_string()))
            }
            MIDI_SYSCOM_SONG_POSITION => {
                self.set_state(MIDI_SYSCOM_SONG_POSITION);
                (None, MidiAnalysis::Comment("Song Position".to_string()))
            }
            MIDI_SYSCOM_SONG_SELECT => {
                self.set_state(MIDI_SYSCOM_SONG_SELECT);
                (None, MidiAnalysis::Comment("Song Select".to_string()))
            }
            MIDI_SYSCOM_TUNE_REQUEST => {
                self.clear_state();
                (
                    Some(MidiMessage::TuneRequest),
                    MidiAnalysis::Comment("Tune Request".to_string()),
                )
            }

            // System Real Time Message - no effect to running status
            MIDI_SYSRT_TIMING_CLOCK => (
                Some(MidiMessage::TimingClock),
                MidiAnalysis::Comment("Timing Clock".to_string()),
            ),
            MIDI_SYSRT_START => (
                Some(MidiMessage::Start),
                MidiAnalysis::Comment("Start".to_string()),
            ),
            MIDI_SYSRT_CONTINUE => (
                Some(MidiMessage::Continue),
                MidiAnalysis::Comment("Continue".to_string()),
            ),
            MIDI_SYSRT_STOP => (
                Some(MidiMessage::Stop),
                MidiAnalysis::Comment("Stop".to_string()),
            ),
            MIDI_SYSRT_ACTIVE_SENSE => (
                Some(MidiMessage::ActiveSensing),
                MidiAnalysis::Comment("Active Sense".to_string()),
            ),
            MIDI_SYSRT_SYSTEM_RESET => (
                Some(MidiMessage::SystemReset),
                MidiAnalysis::Comment("System Reset".to_string()),
            ),

            // System Exclusive Message
            MIDI_SYSEX_SOX => {
                self.set_state(MIDI_SYSEX_SOX);
                self.sysex = vec![];
                (
                    None,
                    MidiAnalysis::Comment("Start of Exclusive".to_string()),
                )
            }
            MIDI_SYSEX_EOX => {
                if self.status != Some(MIDI_SYSEX_SOX) {
                    (None, MidiAnalysis::Warning(
                        "Received End of Exclusive while not within a System Exclusive sequence"
                            .to_string(),
                    ))
                } else {
                    self.clear_state();
                    (
                        Some(MidiMessage::SystemExclusive(self.sysex.clone())),
                        MidiAnalysis::Comment("End of Exclusive".to_string()),
                    )
                }
            }

            // Undefined System Message - no effect to running status
            undef => (
                None,
                MidiAnalysis::Warning(format!("Undefined status byte: {}", undef)),
            ),
        }
    }

    /// Parses the given data byte
    fn parse_data_byte(&mut self, byte: u8) -> (Option<MidiMessage>, MidiAnalysis) {
        if self.status.is_none() {
            return (
                None,
                MidiAnalysis::Warning("Orphaned data byte".to_string()),
            );
        }

        let state = self.status.unwrap();
        match state {
            // Channel Messages
            MIDI_MSG_NOTE_OFF => {
                if let Some(note) = self.d0 {
                    self.clear_data();
                    (
                        Some(MidiMessage::NoteOff {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        }),
                        MidiAnalysis::Comment(format!(
                            "Note Off (Channel {}): Velocity: {}",
                            self.channel, byte
                        )),
                    )
                } else {
                    self.d0 = Some(byte);
                    (
                        None,
                        MidiAnalysis::Comment(format!(
                            "Note Off (Channel {}): Note {}",
                            self.channel,
                            byte
                        )),
                    )
                }
            }

            MIDI_MSG_NOTE_ON => {
                if let Some(note) = self.d0 {
                    self.clear_data();
                    (
                        Some(MidiMessage::NoteOn {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        }),
                        if byte == 0 {
                            MidiAnalysis::Info(format!(
                                "Note On* (Channel {}): Velocity: {} = NOTE OFF",
                                self.channel, byte
                            ))
                        } else {
                            MidiAnalysis::Comment(format!(
                                "Note On (Channel {}): Velocity: {}",
                                self.channel, byte
                            ))
                        },
                    )
                } else {
                    self.d0 = Some(byte);
                    (
                        None,
                        MidiAnalysis::Comment(format!(
                            "Note On (Channel {}): Note {}",
                            self.channel,
                            byte
                        )),
                    )
                }
            }

            MIDI_MSG_POLY_PRESSURE => {
                if let Some(note) = self.d0 {
                    self.clear_data();
                    (
                        Some(MidiMessage::PolyPressure {
                            channel: self.channel,
                            note,
                            pressure: byte,
                        }),
                        MidiAnalysis::Comment(format!(
                            "Poly Pressure (Channel {}): Pressure {}",
                            self.channel, byte
                        )),
                    )
                } else {
                    self.d0 = Some(byte);
                    (
                        None,
                        MidiAnalysis::Comment(format!(
                            "Poly Pressure (Channel {}): Note {}",
                            self.channel, byte
                        )),
                    )
                }
            }

            MIDI_MSG_CONTROL_CHANGE => self.parse_control_change(byte),

            MIDI_MSG_PROGRAM_CHANGE => (
                Some(MidiMessage::ProgramChange {
                    channel: self.channel,
                    program: byte,
                }),
                MidiAnalysis::Comment(format!(
                    "Program Change (Channel {}): Program {}",
                    self.channel, byte
                )),
            ),

            MIDI_MSG_CHANNEL_PRESSURE => (
                Some(MidiMessage::ChannelPressure {
                    channel: self.channel,
                    pressure: byte,
                }),
                MidiAnalysis::Comment(format!(
                    "Channel Pressure (Channel {}): Pressure {}",
                    self.channel, byte
                )),
            ),

            MIDI_MSG_PITCH_BEND => {
                if let Some(lsb) = self.d0 {
                    self.clear_data();
                    let bend = ((byte as u16) << 7) | (lsb as u16);
                    (
                        Some(MidiMessage::PitchBend {
                            channel: self.channel,
                            value: bend,
                        }),
                        MidiAnalysis::Comment(format!(
                            "Pitch Bend MSB (Channel {}): Bend: {}",
                            self.channel, bend
                        )),
                    )
                } else {
                    self.d0 = Some(byte);
                    (
                        None,
                        MidiAnalysis::Comment(format!("Pitch Bend LSB (Channel {})", self.channel)),
                    )
                }
            }

            // System Common
            MIDI_SYSCOM_MTC_FRAME => {
                self.clear_state();
                (
                    Some(MidiMessage::MtcQuarterFrame(byte)),
                    MidiAnalysis::Comment(format!("MTC Frame: 0x{:20X}", byte)),
                )
            }

            MIDI_SYSCOM_SONG_POSITION => {
                if let Some(lsb) = self.d0 {
                    self.clear_state();
                    let spp = ((byte as u16) << 7) | (lsb as u16);
                    (
                        Some(MidiMessage::SongPosition(spp)),
                        MidiAnalysis::Comment(format!(
                            "Song Position MSB (Song Position = {}",
                            spp
                        )),
                    )
                } else {
                    self.d0 = Some(byte);
                    (None, MidiAnalysis::Comment("Song Position LSB".to_string()))
                }
            }

            MIDI_SYSCOM_SONG_SELECT => {
                self.clear_state();
                (
                    Some(MidiMessage::SongSelect(byte)),
                    MidiAnalysis::Comment(format!("Song Select: {}", byte)),
                )
            }

            // System Exclusive
            MIDI_SYSEX_SOX => {
                self.sysex.push(byte);
                (None, MidiAnalysis::Comment("SysEx data byte".to_string()))
            }

            // Base case - this shouldn't happen
            _ => {
                panic!("Got data byte {:2X} while in state 0x{:2x}", byte, state);
            }
        }
    }

    fn parse_control_change(&mut self, byte: u8) -> (Option<MidiMessage>, MidiAnalysis) {
        if self.d0.is_none() {
            self.d0 = Some(byte);
            return (
                None,
                MidiAnalysis::Comment(format!(
                    "Control Change (Channel {}): Controller {} ({})",
                    self.channel,
                    byte,
                    controls::get_controller_name(byte)
                )),
            );
        }

        let control = self.d0.unwrap();
        self.clear_data();
        match control {
            MIDI_CH_MODE_ALL_SOUNDS_OFF => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::AllSoundOff,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 120 All Sounds Off. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Comment(format!("All Sounds Off (Channel {})", self.channel))
                },
            ),

            MIDI_CH_MODE_RESET_ALL_CONTROLLERS => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::ResetAllControllers,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 121 Reset All Controllers. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Comment(format!(
                        "Reset All Controllers (Channel {})",
                        self.channel
                    ))
                },
            ),

            MIDI_CH_MODE_LOCAL_CONTROL => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::LocalControl(byte >= 64),
                }),
                if byte != 0 || byte != 127 {
                    MidiAnalysis::Warning("Invalid data value for Channel Mode 122 Local Control. Expected 0 (local control off) or 0x7F (local control on)".to_string())
                } else {
                    MidiAnalysis::Comment(format!(
                        "Local Control (Channel {}): {}",
                        self.channel,
                        if byte == 0 { "Off" } else { "On" }
                    ))
                },
            ),

            MIDI_CH_MODE_ALL_NOTES_OFF => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::AllNotesOff,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 123 All Notes Off. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Comment(format!("All Notes Off (Channel {})", self.channel))
                },
            ),

            MIDI_CH_MODE_OMNI_MODE_OFF => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::OmniModeOff,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 124 Omni Mode Off. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Info(format!(
                        "Omni Mode Off (Channel {}) (All Notes Off)",
                        self.channel
                    ))
                },
            ),

            MIDI_CH_MODE_OMNI_MODE_ON => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::OmniModeOn,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 125 Omni Mode On. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Info(format!(
                        "Omni Mode On (Channel {}) (All Notes Off)",
                        self.channel
                    ))
                },
            ),

            MIDI_CH_MODE_MONO_MODE_ON => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::MonoModeOn(byte),
                }),
                MidiAnalysis::Comment(format!(
                    "Mono Mode On (Channel {}) (Poly Mode Off): Channels {}",
                    self.channel, byte
                )),
            ),

            MIDI_CH_MODE_POLY_MODE_ON => (
                Some(MidiMessage::ChannelMode {
                    channel: self.channel,
                    mode: MidiChannelMode::PolyModeOn,
                }),
                if byte != 0 {
                    MidiAnalysis::Warning(
                        "Invalid data byte for Channel Mode 127 Poly Mode On. 0x00 expected"
                            .to_string(),
                    )
                } else {
                    MidiAnalysis::Info(format!(
                        "Poly Mode On (Channel {}) (Mono Mode Off) (All Notes Off)",
                        self.channel
                    ))
                },
            ),

            _ => (
                Some(MidiMessage::ControlChange {
                    channel: self.channel,
                    control,
                    value: byte,
                }),
                MidiAnalysis::Comment(format!(
                    "Control Change (Channel {}): Controller {} ({}): Value {}",
                    self.channel, control, controls::get_controller_name(control), byte
                )),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::midi::{MidiMessage, MidiParser};

    #[test]
    fn note_on() {
        let mut parser = MidiParser::new();
        assert_eq!(parser.parse_midi(0x95), None);
        assert_eq!(parser.parse_midi(60), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOn {
                channel: 5,
                note: 60,
                velocity: 127,
            })
        );
    }
    #[test]
    fn note_off() {
        let mut parser = MidiParser::new();
        assert_eq!(parser.parse_midi(0x83), None);
        assert_eq!(parser.parse_midi(59), None);
        assert_eq!(
            parser.parse_midi(66),
            Some(MidiMessage::NoteOff {
                channel: 3,
                note: 59,
                velocity: 66,
            })
        );
    }
    #[test]
    fn running_status_note_on() {
        let mut parser = MidiParser::new();
        assert_eq!(parser.parse_midi(0x90), None);
        assert_eq!(parser.parse_midi(60), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOn {
                channel: 0,
                note: 60,
                velocity: 127,
            })
        );
        assert_eq!(parser.parse_midi(61), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOn {
                channel: 0,
                note: 61,
                velocity: 127,
            })
        );
        assert_eq!(parser.parse_midi(62), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOn {
                channel: 0,
                note: 62,
                velocity: 127,
            })
        );
    }
    #[test]
    fn running_status_note_off() {
        let mut parser = MidiParser::new();
        assert_eq!(parser.parse_midi(0x80), None);
        assert_eq!(parser.parse_midi(60), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 60,
                velocity: 127,
            })
        );
        assert_eq!(parser.parse_midi(61), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 61,
                velocity: 127,
            })
        );
        assert_eq!(parser.parse_midi(62), None);
        assert_eq!(
            parser.parse_midi(127),
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 62,
                velocity: 127,
            })
        );
    }
    #[test]
    fn pitch_bend() {
        let mut parser = MidiParser::new();
        assert_eq!(parser.parse_midi(0xE5), None);
        for n in 0x02_F0_u16..0x03_0F_u16 {
            assert_eq!(parser.parse_midi((n as u8) & 0x7F), None);
            assert_eq!(
                parser.parse_midi((n >> 7) as u8),
                Some(MidiMessage::PitchBend {
                    channel: 5,
                    value: n,
                })
            );
        }
    }
}
