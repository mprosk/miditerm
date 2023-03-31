//! The unparser is responsible for converting an instance of the MidiMessage enum back into valid MIDI bytes

use crate::midi::*;

impl MidiMessage {
    /// Converts the `MidiMessage` into its corresponding sequence of MIDI bytes
    /// Extraneous bits within data and channel values will be stripped
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            // CHANNEL MESSAGES
            MidiMessage::NoteOff {
                channel,
                note,
                velocity,
            } => {
                vec![
                    MIDI_MSG_NOTE_OFF | (channel & MIDI_CHANNEL_MASK),
                    note & MIDI_DATA_MASK,
                    velocity & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => {
                vec![
                    MIDI_MSG_NOTE_ON | (channel & MIDI_CHANNEL_MASK),
                    note & MIDI_DATA_MASK,
                    velocity & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::PolyPressure {
                channel,
                note,
                pressure,
            } => {
                vec![
                    MIDI_MSG_POLY_PRESSURE | (channel & MIDI_CHANNEL_MASK),
                    note & MIDI_DATA_MASK,
                    pressure & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::ControlChange {
                channel,
                control,
                value,
            } => {
                vec![
                    MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                    control & MIDI_DATA_MASK,
                    value & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::ChannelMode { channel, mode } => match mode {
                MidiChannelMode::AllSoundOff => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_ALL_SOUNDS_OFF,
                        0,
                    ]
                }
                MidiChannelMode::ResetAllControllers => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_RESET_ALL_CONTROLLERS,
                        0,
                    ]
                }
                MidiChannelMode::LocalControl(on) => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_LOCAL_CONTROL,
                        if on { 127 } else { 0 },
                    ]
                }
                MidiChannelMode::AllNotesOff => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_ALL_NOTES_OFF,
                        0,
                    ]
                }
                MidiChannelMode::OmniModeOff => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_OMNI_MODE_OFF,
                        0,
                    ]
                }
                MidiChannelMode::OmniModeOn => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_OMNI_MODE_ON,
                        0,
                    ]
                }
                MidiChannelMode::MonoModeOn(m) => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_MONO_MODE_ON,
                        m & MIDI_DATA_MASK,
                    ]
                }
                MidiChannelMode::PolyModeOn => {
                    vec![
                        MIDI_MSG_CONTROL_CHANGE | (channel & MIDI_CHANNEL_MASK),
                        MIDI_CH_MODE_POLY_MODE_ON,
                        0,
                    ]
                }
            },
            MidiMessage::ProgramChange { channel, program } => {
                vec![
                    MIDI_MSG_PROGRAM_CHANGE | (channel & MIDI_CHANNEL_MASK),
                    program & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::ChannelPressure { channel, pressure } => {
                vec![
                    MIDI_MSG_CHANNEL_PRESSURE | (channel & MIDI_CHANNEL_MASK),
                    pressure & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::PitchBend { channel, value } => {
                vec![
                    MIDI_MSG_PITCH_BEND | (channel & MIDI_CHANNEL_MASK),
                    (value as u8) & MIDI_DATA_MASK,
                    (value >> 7) as u8 & MIDI_DATA_MASK,
                ]
            }

            // SYSTEM COMMON
            MidiMessage::MtcQuarterFrame(n) => {
                vec![MIDI_SYSCOM_MTC_FRAME, n & MIDI_DATA_MASK]
            }
            MidiMessage::SongPosition(spp) => {
                vec![
                    MIDI_SYSCOM_SONG_POSITION,
                    (spp as u8) & MIDI_DATA_MASK,
                    (spp >> 7) as u8 & MIDI_DATA_MASK,
                ]
            }
            MidiMessage::SongSelect(song) => {
                vec![MIDI_SYSCOM_SONG_SELECT, song & MIDI_DATA_MASK]
            }
            MidiMessage::TuneRequest => {
                vec![MIDI_SYSCOM_TUNE_REQUEST]
            }

            // SYSTEM REAL TIME
            MidiMessage::TimingClock => {
                vec![MIDI_SYSRT_TIMING_CLOCK]
            }
            MidiMessage::Start => {
                vec![MIDI_SYSRT_START]
            }
            MidiMessage::Continue => {
                vec![MIDI_SYSRT_CONTINUE]
            }
            MidiMessage::Stop => {
                vec![MIDI_SYSRT_STOP]
            }
            MidiMessage::ActiveSensing => {
                vec![MIDI_SYSRT_ACTIVE_SENSE]
            }
            MidiMessage::SystemReset => {
                vec![MIDI_SYSRT_SYSTEM_RESET]
            }

            // SYSTEM EXCLUSIVE
            MidiMessage::SystemExclusive(data) => {
                [vec![MIDI_SYSEX_SOX], data, vec![MIDI_SYSEX_EOX]].concat()
            }

            _ => vec![],
        }
    }
}
