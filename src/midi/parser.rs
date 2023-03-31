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

    /// Accepts the given byte and outputs `Some(MidiMessage)`
    /// if the preceding byte sequences parsed into a MIDI message
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
                    return Some(MidiMessage::ProtocolError(
                        "Received `End of Exclusive` while not within a System Exclusive sequence"
                            .to_string(),
                    ));
                }
                self.clear_state();
                return Some(MidiMessage::SystemExclusive(self.sysex.clone()));
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
                        self.clear_data();
                        return Some(MidiMessage::NoteOff {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        });
                    }
                }
                MIDI_MSG_NOTE_ON => {
                    if let Some(note) = self.d0 {
                        self.clear_data();
                        return Some(MidiMessage::NoteOn {
                            channel: self.channel,
                            note,
                            velocity: byte,
                        });
                    }
                }
                MIDI_MSG_POLY_PRESSURE => {
                    if let Some(note) = self.d0 {
                        self.clear_data();
                        return Some(MidiMessage::PolyPressure {
                            channel: self.channel,
                            note,
                            pressure: byte,
                        });
                    }
                }
                MIDI_MSG_CONTROL_CHANGE => {
                    if let Some(control) = self.d0 {
                        self.clear_data();
                        return match control {
                            MIDI_CH_MODE_ALL_SOUNDS_OFF => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::AllSoundOff,
                            }),
                            MIDI_CH_MODE_RESET_ALL_CONTROLLERS => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::ResetAllControllers,
                            }),
                            MIDI_CH_MODE_LOCAL_CONTROL => {
                                if byte != 0 || byte != 127 {
                                    Some(MidiMessage::ProtocolError(format!("Invalid data value for Channel Mode 122 Local Control. Got {}, expected to be 0 (local control off) or 127 (local control on)", byte)))
                                } else {
                                    Some(MidiMessage::ChannelMode {
                                        channel: self.channel,
                                        mode: MidiChannelMode::LocalControl(byte == 127),
                                    })
                                }
                            }
                            MIDI_CH_MODE_ALL_NOTES_OFF => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::AllNotesOff,
                            }),
                            MIDI_CH_MODE_OMNI_MODE_OFF => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::OmniModeOff,
                            }),
                            MIDI_CH_MODE_OMNI_MODE_ON => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::OmniModeOn,
                            }),
                            MIDI_CH_MODE_MONO_MODE_ON => Some(MidiMessage::ChannelMode {
                                channel: self.channel,
                                mode: MidiChannelMode::MonoModeOn(byte),
                            }),
                            MIDI_CH_MODE_POLY_MODE_ON => Some(MidiMessage::ChannelMode {
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
                        self.clear_data();
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

    /// Clears the internal data buffer
    fn clear_data(&mut self) {
        self.d0 = None;
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
