use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// Current MIDI Association membership status of this manufacturer
pub enum ManufacturerStatus {
    /// Current MIDI Association Corporate member
    Current,
    /// SysEx Only
    SysExOnly,
    /// Membership lapsed
    Lapsed,
}

#[derive(Debug, Deserialize)]
/// Identifies the regional Group of the manufacturer.
/// Groups are delineated within specific ranges of ID numbers.
pub enum ManufacturerGroup {
    /// North America
    ///
    /// `01` to `1F` and `[00,00,00]` to `[00,1F,7F]`
    NorthAmerica,

    /// Europe
    ///
    /// `20` to `3F` and `[00,20,00]` to `[00,3F,7F]`
    ///
    /// Also includes China
    Europe,

    /// Japan
    ///
    /// `40` to `5F` and `[00,40,00]` to `[00,5F,7F]`
    Japan,

    /// Other
    ///
    /// `60 to 7C` and `[00,60,00]` to `[00,7F,7F]`
    Other,

    /// Special
    ///
    /// `7D` to `7F`
    Special,
}

#[derive(Debug, Deserialize)]
pub struct ManufacturerID {
    pub id: Vec<u8>,
    pub manufacturer: String,
    pub group: ManufacturerGroup,
    pub status: Option<ManufacturerStatus>,
    pub reserved: bool,
}
