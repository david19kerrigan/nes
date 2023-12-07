use crate::util::*;

pub struct Bus {
    pub memory: [u8; 65536],
}

impl Bus {
    pub fn new() -> Bus {
        Bus { memory: [0; 65536] }
    }

    // Read address in memory from a low and a high byte in low endian
    pub fn read_16(&mut self, low: u8, high: u8) -> u8 {
        self.memory[combine_low_high(low, high) as usize]
    }

    // Read zero page address
    pub fn read_8(&mut self, addr: u8) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_usize(&mut self, addr: usize) -> u8 {
        self.memory[addr]
    }

    // Read one byte in relation to the PC
    pub fn read_single(&mut self, addr: usize) -> u8 {
        self.memory[addr - 1]
    }

    // Read two bytes in relation to the PC
    pub fn read_double(&mut self, addr: usize) -> u16 {
        combine_low_high(self.memory[addr - 2], self.memory[addr - 1])
    }

    // Does addressing mode Indexed Y cross the page?
    pub fn cross_idy(&mut self, pc: usize, offset: u8) -> u8 {
        let zp = self.read_8(self.memory[pc]);
        combine_low_high(zp, zp + 1)
            .overflowing_add(offset as u16)
            .1 as u8
    }

    // Does addressing mode Indexed X cross the page?
    pub fn cross_abs(&mut self, pc_first: usize, pc_second: usize, offset: u8) -> u8 {
        self.read_16(self.memory[pc_first], self.memory[pc_second])
            .overflowing_add(offset)
            .1 as u8
    }
}
