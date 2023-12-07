pub fn combine_8(low: u8, high: u8) -> u16 {
    low as u16 | (high as u16) << 8
}
