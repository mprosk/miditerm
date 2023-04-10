pub fn get_controller_name(control_number: u8) -> String {
    match control_number {
        0x00 => "Bank select",
        0x01 => "Mod wheel",
        0x02 => "Breath controller",
        0x04 => "Foot controller",
        0x05 => "Portamento time",
        0x06 => "Data entry MSB",
        0x07 => "Channel volume",
        0x08 => "Balance",
        0x0A => "Pan",
        0x0B => "Expression controller",
        0x0C => "Effect control 1",
        0x0D => "Effect control 2",
        0x10 => "General purpose controller 1",
        0x11 => "General purpose controller 2",
        0x12 => "General purpose controller 3",
        0x13 => "General purpose controller 4",
        _ => "Undefined",
    }
    .to_string()
}
