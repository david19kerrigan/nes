use crate::util::*;
use std::fs::read;

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

    pub fn load_cartridge(&mut self, path: &str) {
        let rom = match read(path) {
            Ok(res) => res,
            Err(why) => panic!("{}", why),
        };
        for i in 0..rom.len() {
            self.memory[0x4020 + i] = rom[i];
        }
    }

    pub fn check_addr_in_range(&mut self, addr: usize) {
        if addr > memory_size {
            panic!("bus memory address out of range");
        }
    }

    pub fn write_16(&mut self, addr: u16, val: u8) {
        let u_addr = addr as usize;
        self.check_addr_in_range(u_addr);
        self.memory[u_addr] = val;
    }

    // Read address in memory from a low and a high byte in low endian
    pub fn read_low_high(&mut self, low: u8, high: u8) -> u8 {
        self.read_16(combine_low_high(low, high))
    }

    // Read absolute address
    pub fn read_8(&mut self, addr: u8) -> u8 {
        self.read_16(addr as u16)
    }

    // Read zero page address
    pub fn read_16(&mut self, addr: u16) -> u8 {
        let u_addr = addr as usize;
        self.check_addr_in_range(u_addr);
        self.memory[u_addr]
    }

    // Read one byte in relation to the PC
    pub fn read_single(&mut self, addr: u16) -> u8 {
        self.read_16(addr - 1)
    }

    // Read two bytes in relation to the PC
    pub fn read_double(&mut self, addr: u16) -> u16 {
        combine_low_high(self.read_16(addr - 2), self.read_16(addr - 1))
    }

    // Does addressing mode Rel cross the page?
    pub fn cross_rel(&mut self, pc: u16) -> u8 {
        let low = self.read_16(pc + 1);
        (pc as u8).overflowing_add(low).1 as u8
    }

    // Does addressing mode Indexed Y cross the page?
    pub fn cross_idy(&mut self, pc: u16, offset: u8) -> u8 {
        let low = self.read_16(pc + 1);
        let zp = self.read_8(low);
        combine_low_high(zp, zp + 1)
            .overflowing_add(offset as u16)
            .1 as u8
    }

    // Does addressing mode Indexed X cross the page?
    pub fn cross_abs(&mut self, pc: u16, offset: u8) -> u8 {
        let low = self.read_16(pc + 1);
        let high = self.read_16(pc + 2);
        self.read_low_high(low, high).overflowing_add(offset).1 as u8
    }

    pub fn DEC(&mut self, addr: u16) -> u8 {
        let val = self.read_16(addr) - 1;
        self.write_16(addr, val);
        val
    }

    pub fn INC(&mut self, addr: u16) -> u8 {
        let val = self.read_16(addr) + 1;
        self.write_16(addr, val);
        val
    }
}
