// Little endian conversion
pub fn combine_low_high(low: u8, high: u8) -> u16 {
    low as u16 | (high as u16) << 8
}

pub fn u8_or(a: u8, b: u8) -> u8 {
    a | b
}

pub fn u8_and(a: u8, b: u8) -> u8 {
    a & b
}

pub fn u8_xor(a: u8, b: u8) -> u8 {
    a ^ b
}

pub fn u8_shl(a: u8) -> u8 {
    a << 1
}

pub fn u8_shr(a: u8) -> u8 {
    a >> 1
}
