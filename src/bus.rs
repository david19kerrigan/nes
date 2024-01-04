use crate::util::*;
use std::fs::read;

const cpu_memory_size: usize = 65535;
const ppu_memory_size: usize = 16384;

pub struct Bus {
    pub cpu_memory: [u8; cpu_memory_size + 1],
    pub ppu_memory: [u8; ppu_memory_size + 1],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cpu_memory: [0; cpu_memory_size + 1],
            ppu_memory: [0; ppu_memory_size + 1],
        }
    }

    pub fn load_cartridge(&mut self, path: &str) {
        let rom = match read(path) {
            Ok(res) => res,
            Err(why) => panic!("{}", why),
        };

        let six = rom[6];
        let seven = rom[7];
        let mapper = combine_low_high(six & 0b10000000, seven & 0b10000000);

        match mapper {
            0 => {
                for i in 0x10..0x4000 {
                    self.cpu_memory[0x8000 + i - 0x10] = rom[i];
                    self.cpu_memory[0xC000 + i - 0x10] = rom[i];
                }
                for i in 0x4000..0x6000 {
                    self.ppu_memory[0 + i - 0x4000] = rom[i];
                }
            }
            _ => panic!("unrecognized mapper"),
        }
    }

    pub fn check_addr_in_range(&mut self, addr: usize, component: &Component) {
        if addr > cpu_memory_size {
            panic!("bus memory address out of range");
        }
    }

    pub fn write_16(&mut self, addr: u16, val: u8, component: Component) {
        let u_addr = addr as usize;
        self.check_addr_in_range(u_addr, &component);
		if component == Component::CPU {
			self.cpu_memory[u_addr] = val;
		} else if component == Component::PPU {
			self.ppu_memory[u_addr] = val;
		}
    }

    // Read address in memory from a low and a high byte in low endian
    pub fn read_low_high(&mut self, low: u8, high: u8) -> u8 {
        self.read_16(combine_low_high(low, high), Component::CPU)
    }

    // Read zero page address
    pub fn read_8(&mut self, addr: u8) -> u8 {
        self.read_16(addr as u16, Component::CPU)
    }

    // Read absolute address
    pub fn read_16(&mut self, addr: u16, component: Component) -> u8 {
        let u_addr = addr as usize;
        self.check_addr_in_range(u_addr, &component);
		if component == Component::CPU {
			self.cpu_memory[u_addr]
		} else if component == Component::PPU {
			self.ppu_memory[u_addr]
		} else{
			panic!("oh shit homie");
		}
    }

    // Read one byte in relation to the PC
    pub fn read_single(&mut self, addr: u16) -> u8 {
        self.read_16(addr - 1, Component::CPU)
    }

    // Read two bytes in relation to the PC
    pub fn read_double(&mut self, addr: u16) -> u16 {
        combine_low_high(self.read_16(addr - 2, Component::CPU), self.read_16(addr - 1, Component::CPU))
    }

    // Does addressing mode Rel cross the page?
    pub fn cross_rel(&mut self, pc: u16) -> u8 {
        let low = self.read_16(pc + 1, Component::CPU) as i16;
        (pc + 2).overflowing_add_signed(low).1 as u8
    }

    // Does addressing mode Indexed Y cross the page?
    pub fn cross_idy(&mut self, pc: u16, offset: u8) -> u8 {
        let addr = self.read_16(pc + 1, Component::CPU);
        let low = self.read_8(addr);
        let (high_addr, overflow) = addr.overflowing_add(1);
        if overflow {
            return 1;
        }
        let high = self.read_8(high_addr);
        combine_low_high(low, high).overflowing_add(offset as u16).1 as u8
    }

    // Does addressing mode Indexed X cross the page?
    pub fn cross_abs(&mut self, pc: u16, offset: u8) -> u8 {
        let low = self.read_16(pc + 1, Component::CPU);
        let high = self.read_16(pc + 2, Component::CPU);
        println!("crossing {:x} {:x}", combine_low_high(low, high), offset);
        low.overflowing_add(offset).1 as u8
    }

    pub fn DEC(&mut self, addr: u16) -> u8 {
        let val = self.read_16(addr, Component::CPU).wrapping_sub(1);
        self.write_16(addr, val, Component::CPU);
        val
    }

    pub fn INC(&mut self, addr: u16) -> u8 {
        let val = self.read_16(addr, Component::CPU).wrapping_add(1);
        self.write_16(addr, val, Component::CPU);
        val
    }
}
