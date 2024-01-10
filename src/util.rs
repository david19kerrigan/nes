#[derive(PartialEq)]
pub enum Component {
    CPU,
    PPU,
    APU,
}

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

pub fn set_u8_bits(num: u8, val: u8, l: u8, r: u8) -> u8 {
    let mut new_num = num;
    for n in r..l {
        new_num = set_u8_bit(new_num, n, get_u8_bit(val, n - r));
    }
    new_num
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

pub fn get_u8_bit(num: u8, bit: u8) -> u8 {
    num >> bit & 0b00000001
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
