//! Low level MIDI parser

pub mod controls;
mod parser;
pub mod sysex;
mod unparser;

// PUBLIC CONSTANTS
pub const MIDI_BAUD_RATE: u32 = 31_250_u32;

// Bit masks
const MIDI_BYTE_TYPE_MASK: u8 = 0b_1000_0000_u8;
const MIDI_DATA_MASK: u8 = 0b_0111_1111_u8;
const MIDI_CHANNEL_MASK: u8 = 0b_0000_1111_u8;
const MIDI_STATUS_MASK: u8 = 0b_1111_0000_u8;

// Channel Voice Messages
const MIDI_MSG_NOTE_OFF: u8 = 0x80_u8;
const MIDI_MSG_NOTE_ON: u8 = 0x90_u8;
const MIDI_MSG_POLY_PRESSURE: u8 = 0xA0_u8;
const MIDI_MSG_CONTROL_CHANGE: u8 = 0xB0_u8;
const MIDI_MSG_PROGRAM_CHANGE: u8 = 0xC0_u8;
const MIDI_MSG_CHANNEL_PRESSURE: u8 = 0xD0_u8;
const MIDI_MSG_PITCH_BEND: u8 = 0xE0_u8;

// Channel Mode Messages
const MIDI_CMM_ALL_SOUNDS_OFF: u8 = 120_u8;
const MIDI_CMM_RESET_ALL_CONTROLLERS: u8 = 121_u8;
const MIDI_CMM_LOCAL_CONTROL: u8 = 122_u8;
const MIDI_CMM_ALL_NOTES_OFF: u8 = 123_u8;
const MIDI_CMM_OMNI_MODE_OFF: u8 = 124_u8;
const MIDI_CMM_OMNI_MODE_ON: u8 = 125_u8;
const MIDI_CMM_MONO_MODE_ON: u8 = 126_u8;
const MIDI_CMM_POLY_MODE_ON: u8 = 127_u8;

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

/// Enum representing MIDI Channel Mode messages
#[derive(Debug, PartialEq)]
pub enum MidiChannelMode {
    AllSoundOff,
    ResetAllControllers,
    LocalControl(bool),
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn(u8),
    PolyModeOn,
}

/// Enum representing all MIDI messages.
/// Can be used to construct an outgoing MIDI message
/// Return type of the `MidiParser`
#[derive(Debug, PartialEq)]
pub enum MidiMessage {
    // Channel Messages
    NoteOff { channel: u8, note: u8, velocity: u8 },
    NoteOn { channel: u8, note: u8, velocity: u8 },
    PolyPressure { channel: u8, note: u8, pressure: u8 },
    ControlChange { channel: u8, control: u8, value: u8 },
    ChannelMode { channel: u8, mode: MidiChannelMode },
    ProgramChange { channel: u8, program: u8 },
    ChannelPressure { channel: u8, pressure: u8 },
    PitchBend { channel: u8, value: u16 },

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
}

/// Responses from the protocol analyzer
#[derive(Debug, PartialEq)]
pub enum MidiAnalysis {
    /// Lowest level of
    Comment(String),
    /// Something noteworthy happened
    ///
    /// Examples:
    /// - Running Status
    /// - Note On with velocity 0
    Info(String),
    /// Something is wrong but it's not invalid MIDI
    ///
    /// Examples:
    /// - Undefined MIDI messages
    /// - Orphaned data bytes
    /// - Timing violations
    Warning(String),
    /// The MIDI specification was explicitly violated
    Violation(String),
}

/// State machine that decodes MIDI messages byte by byte.
///
/// Example:
///
/// ```rust
/// let mut parser = MidiParser::new();
/// assert_eq!(parser.parse_midi(0x90), None);
/// assert_eq!(parser.parse_midi(0x3C), None);
/// assert_eq!(
///     parser.parse_midi(0x7F),
///     Some(MidiMessage::NoteOn {
///         channel: 0,
///         note: 60,
///         velocity: 127,
///     })
/// );
/// ```
pub struct MidiParser {
    status: Option<u8>,
    d0: Option<u8>,
    channel: u8,
    sysex: Vec<u8>,
}
