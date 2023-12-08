use crate::util::*;

const memory_size: usize = 65535;

pub struct Bus {
    pub memory: [u8; memory_size + 1],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            memory: [0; memory_size + 1],
        }
    }

    pub fn check_addr_in_range(&mut self, addr: usize) {
        if addr < 0 || addr > memory_size {
            panic!("bus memory address out of range");
        }
    }

    pub fn write_usize(&mut self, addr: usize, val: u8) {
        self.check_addr_in_range(addr);
        self.memory[addr] = val;
    }

    pub fn read_usize(&mut self, addr: usize) -> u8 {
        self.check_addr_in_range(addr);
        self.memory[addr]
    }

    // Read address in memory from a low and a high byte in low endian
    pub fn read_low_high(&mut self, low: u8, high: u8) -> u8 {
        self.read_usize(combine_low_high(low, high) as usize)
    }

    // Read absolute address
    pub fn read_8(&mut self, addr: u8) -> u8 {
        self.read_usize(addr as usize)
    }

    // Read zero page address
    pub fn read_16(&mut self, addr: u16) -> u8 {
        self.read_usize(addr as usize)
    }

    // Read one byte in relation to the PC
    pub fn read_single(&mut self, addr: usize) -> u8 {
        self.read_usize(addr - 1)
    }

    // Read two bytes in relation to the PC
    pub fn read_double(&mut self, addr: usize) -> u16 {
        combine_low_high(self.read_usize(addr - 2), self.read_usize(addr - 1))
    }

    // Does addressing mode Rel cross the page?
    pub fn cross_rel(&mut self, pc: usize) -> u8 {
        let low = self.read_usize(pc + 1);
        (pc as u8).overflowing_add(low).1 as u8
    }

    // Does addressing mode Indexed Y cross the page?
    pub fn cross_idy(&mut self, pc: usize, offset: u8) -> u8 {
        let low = self.read_usize(pc + 1);
        let zp = self.read_8(low);
        combine_low_high(zp, zp + 1)
            .overflowing_add(offset as u16)
            .1 as u8
    }

    // Does addressing mode Indexed X cross the page?
    pub fn cross_abs(&mut self, pc: usize, offset: u8) -> u8 {
        let low = self.read_usize(pc + 1);
        let high = self.read_usize(pc + 2);
        self.read_low_high(low, high).overflowing_add(offset).1 as u8
    }

    pub fn DEC(&mut self, addr: usize) -> u8 {
        let val = self.read_usize(addr) - 1;
        self.write_usize(addr, val);
        val
    }

    pub fn INC(&mut self, addr: usize) -> u8 {
        let val = self.read_usize(addr) + 1;
        self.write_usize(addr, val);
        val
    }
}
