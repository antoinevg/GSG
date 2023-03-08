// - BoardInfo ----------------------------------------------------------------

/// BoardInformation
pub struct BoardInformation<'a> {
    pub board_id: [u8; 4],
    pub version_string: &'a str,
    pub part_id: [u8; 8],
    pub serial_number: [u8; 16],
}
