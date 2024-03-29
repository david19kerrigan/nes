pub const CONTROL: u16 = 0x2000;
pub const MASK: u16 = 0x2001;
pub const STATUS: u16 = 0x2002;
pub const OAM_ADDR: u16 = 0x2003;
pub const OAM_DATA: u16 = 0x2004;
pub const SCROLL: u16 = 0x2005;
pub const ADDR: u16 = 0x2006;
pub const DATA: u16 = 0x2007;
pub const OAM_DMA: u16 = 0x4014;
pub const INPUT: u16 = 0x4016;

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

pub fn get_u8_bit(num: u8, bit: u8) -> u8 {
    num >> bit & 0x01
}

pub fn get_u16_bit(num: u16, bit: u8) -> u8 {
    (num >> bit & 0x0001) as u8
}

pub fn set_u8_bit(num: u8, bit: u8, val: u8) -> u8 {
    if val == 1 {
        num | (1 << bit)
    } else if val == 0 {
        num & ((1 << bit) ^ 0b11111111)
    } else {
		panic!("non-binary bit");
	}
}

pub fn get_u8_bits(num: u8, l: u8, r: u8) -> u8 {
	let new_num = num >> r;
    let mut mask = 1;
    for n in 0..l - r {
        mask <<= 1;
        mask |= 1;
    }
    new_num & mask
}
