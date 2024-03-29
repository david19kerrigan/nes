use crate::util::*;
use crate::Ppu;
use std::fs::read;

const cpu_memory_size: usize = 0x10000;
const ppu_memory_size: usize = 0x4000;

pub struct Bus {
    pub cpu_memory: [u8; cpu_memory_size + 1],
    pub ppu_memory: [u8; ppu_memory_size + 1],
    pub input: u8,
    pub polling: bool,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cpu_memory: [0; cpu_memory_size + 1],
            ppu_memory: [0; ppu_memory_size + 1],
            input: 0,
            polling: false,
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
                for i in 0x10..0x4010 {
                    self.cpu_memory[0x8000 + i - 0x10] = rom[i];
                    self.cpu_memory[0xC000 + i - 0x10] = rom[i];
                }
                for i in 0x4010..0x6010 {
                    self.ppu_memory[0 + i - 0x4010] = rom[i];
                }
            }
            _ => panic!("unrecognized mapper"),
        }
    }

    pub fn ppu_check_addr_in_range(&mut self, addr: usize) {
        if addr > ppu_memory_size {
            panic!("bus memory address out of range");
        }
    }

    pub fn cpu_check_addr_in_range(&mut self, addr: usize) {
        if addr > cpu_memory_size {
            panic!("bus memory address out of range");
        }
    }

    pub fn ppu_write_16(&mut self, addr: u16, val: u8) {
        let u_addr = addr as usize;
        self.ppu_check_addr_in_range(u_addr);
        self.ppu_memory[u_addr] = val;
    }

    pub fn cpu_write_16(&mut self, addr: u16, val: u8) {
        let u_addr = addr as usize;
        self.cpu_check_addr_in_range(u_addr);
        if addr == INPUT {
            if val == 1 {
                self.polling = true;
                self.cpu_memory[INPUT as usize] = self.input;
            } else if val == 0 {
                self.polling = false;
            }
            return;
        }
        self.cpu_memory[u_addr] = val;
    }

    pub fn cpu_ppu_reg_addr_map(&mut self, addr: u16) -> u16 {
        let mut mut_addr = addr;
        if mut_addr < 0x4000 && mut_addr > 0x2007 {
            mut_addr = 0x2000 + ((mut_addr - 0x2007) % 8);
        }
        mut_addr
    }

    pub fn cpu_write_16_ppu_regs(&mut self, addr: u16, val: u8, ppu: &mut Ppu) {
        let mut_addr = self.cpu_ppu_reg_addr_map(addr);
        self.cpu_write_16(mut_addr, val);

        if mut_addr == DATA {
            ppu.write_data(val, self);
        } else if mut_addr == ADDR {
            ppu.write_addr(val);
        } else if mut_addr == OAM_DATA { // Only using OAM DMA for now
        } else if mut_addr == OAM_ADDR {
        } else if mut_addr == OAM_DMA {
            ppu.write_oam(val, self);
        } else if mut_addr == CONTROL {
            ppu.control.read(self);
        } else if mut_addr == MASK {
            ppu.mask.read(self);
        }
    }

    // Read address in memory from a low and a high byte in low endian
    pub fn cpu_read_low_high(&mut self, low: u8, high: u8) -> u8 {
        self.cpu_read_16(combine_low_high(low, high))
    }

    // Read zero page address
    pub fn cpu_read_8(&mut self, addr: u8) -> u8 {
        self.cpu_read_16(addr as u16)
    }

    // Read absolute address
    pub fn ppu_read_16(&mut self, addr: u16) -> u8 {
        let u_addr = addr as usize;
        self.ppu_check_addr_in_range(u_addr);
        self.ppu_memory[u_addr]
    }

    pub fn cpu_read_16_ppu_regs(&mut self, addr: u16, ppu: &mut Ppu) -> u8 {
        let u_addr = addr as usize;
        let mut_addr = self.cpu_ppu_reg_addr_map(addr) as usize;
        let mut temp = self.cpu_read_16(mut_addr as u16);

        if mut_addr == (STATUS as usize) {
            self.cpu_memory[STATUS as usize] = set_u8_bit(self.cpu_memory[STATUS as usize], 7, 0);
            if ppu.line == 240 && ppu.cycle >= 2 && ppu.cycle < 5 {
                temp = self.cpu_memory[mut_addr];
            }
        }

        temp
    }

    pub fn cpu_read_16(&mut self, addr: u16) -> u8 {
        let u_addr = addr as usize;
        self.cpu_check_addr_in_range(u_addr);
        let mut ret = self.cpu_memory[u_addr].clone();
        if addr == INPUT {
			ret &= 0x01;
			if !self.polling {
				self.cpu_memory[u_addr] >>= 1;
			}
        }
        ret
    }

    // Read one byte in relation to the PC
    pub fn read_single(&mut self, addr: u16) -> u8 {
        self.cpu_read_16(addr - 1)
    }

    // Read two bytes in relation to the PC
    pub fn read_double(&mut self, addr: u16) -> u16 {
        combine_low_high(self.cpu_read_16(addr - 2), self.cpu_read_16(addr - 1))
    }

    // Does addressing mode Rel cross the page?
    pub fn cross_rel(&mut self, pc: u16) -> u8 {
        let offset = self.cpu_read_16(pc + 1) as i8 as i16;
        let new = (pc + 2).wrapping_add_signed(offset);
        (get_u16_bit(pc + 2, 8) != get_u16_bit(new, 8)) as u8
    }

    // Does addressing mode Indexed Y cross the page?
    pub fn cross_idy(&mut self, pc: u16, offset: u8) -> u8 {
        let addr = self.cpu_read_16(pc + 1);
        let low = self.cpu_read_8(addr);
        let (high_addr, overflow) = addr.overflowing_add(1);
        if overflow {
            return 1;
        }
        let high = self.cpu_read_8(high_addr);
        (combine_low_high(low, high) as u8)
            .overflowing_add(offset as u8)
            .1 as u8
    }

    // Does addressing mode Indexed X cross the page?
    pub fn cross_abs(&mut self, pc: u16, offset: u8) -> u8 {
        let low = self.cpu_read_16(pc + 1);
        let high = self.cpu_read_16(pc + 2);
        low.overflowing_add(offset).1 as u8
    }

    pub fn DEC(&mut self, addr: u16) -> u8 {
        let val = self.cpu_read_16(addr).wrapping_sub(1);
        self.cpu_write_16(addr, val);
        val
    }

    pub fn INC(&mut self, addr: u16) -> u8 {
        let val = self.cpu_read_16(addr).wrapping_add(1);
        self.cpu_write_16(addr, val);
        val
    }
}
