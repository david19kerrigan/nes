use crate::util::*;
use crate::Bus;

use csv::StringRecord;

const ERR_OP: &str = "Invalid Opcode";
const ERR_ADDR: &str = "Invalid Addressing Mode";

#[rustfmt::skip]
#[derive(PartialEq)]
#[derive(Debug)]
enum Instructions {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

#[rustfmt::skip]
#[derive(PartialEq)]
#[derive(Debug)]
enum Addressing {
    IMP, ACC, IMM, ZPG, ZPX, ZPY, REL, ABS, ABX, ABY, IND, IDX, IDY,
}

pub struct Cpu {
    // registers
    a: u8,
    x: u8,
    y: u8,
    pub pc: u16,
    //flags
    c: bool,
    z: bool,
    i: bool,
    d: bool,
    b: bool,
    o: bool,
    n: bool,
    // misc
    instr: Instructions,
    addr: Addressing,
    stack: [u8; 256],
    stack_pointer: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            // registers
            a: 0,
            x: 0,
            y: 0,
            pc: 0x34,
            //flags
            c: false,
            z: false,
            i: true,
            d: false,
            b: false,
            o: false,
            n: false,
            // misc
            instr: Instructions::NOP,
            addr: Addressing::IMM,
            stack: [0; 256],
            stack_pointer: 0xFD,
        }
    }

    // --------------- INSTRUCTIONS --------------------

	#[rustfmt::skip]
    pub fn PHP(&mut self, bus: &mut Bus) {
        let all_flags = self.flags_to_byte() | 0b00010000;
        self.stack_push(all_flags, bus);
    }

    pub fn PLP(&mut self, bus: &mut Bus) {
        let all_flags = self.stack_pull(bus);
        self.n = all_flags & 0b10000000 == 0b10000000;
        self.o = all_flags & 0b01000000 == 0b01000000;
        //self.b = all_flags & 0b00100000 == 0b00100000;
        self.b = false;
        self.d = all_flags & 0b00001000 == 0b00001000;
        self.i = all_flags & 0b00000100 == 0b00000100;
        self.z = all_flags & 0b00000010 == 0b00000010;
        self.c = all_flags & 0b00000001 == 0b00000001;
    }

    // --------------- REGISTERS --------------------

    pub fn stack_push_pc(&mut self, bus: &mut Bus) {
        let low = self.pc as u8;
        let high = (self.pc >> 8) as u8;
        println!("low, high = {}, {}", low, high);
        self.stack_push(high, bus);
        self.stack_push(low, bus);
    }

    pub fn stack_pull_pc(&mut self, bus: &mut Bus) {
        let low = self.stack_pull(bus);
        let high = self.stack_pull(bus);
        println!("low, high = {}, {}", low, high);
        self.pc = combine_low_high(low, high);
    }

    pub fn stack_push(&mut self, val: u8, bus: &mut Bus) {
        let stack_addr = 1 << 8 | (self.stack_pointer as u16);
        bus.write_16(stack_addr, val);
        self.stack_pointer -= 1;
    }

    pub fn stack_pull(&mut self, bus: &mut Bus) -> u8 {
        self.stack_pointer += 1;
        let stack_addr = 1 << 8 | (self.stack_pointer as u16);
        bus.read_16(stack_addr)
    }

    pub fn INX(&mut self) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.x
    }

    pub fn INY(&mut self) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.y
    }

    pub fn DEX(&mut self) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.x
    }

    pub fn DEY(&mut self) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.y
    }

    // --------------- FLAGS --------------------

    pub fn flags_to_byte(&mut self) -> u8 {
        (self.n as u8) << 7
            | (self.o as u8) << 6
            | 1 << 5
            //| (self.b as u8) << 4
			| 0 << 4
            | (self.d as u8) << 3
            | (self.i as u8) << 2
            | (self.z as u8) << 1
            | (self.c as u8)
    }

    pub fn flag_interrupt(&mut self, interrupt: bool) {
        self.i = interrupt;
    }

    pub fn flag_decimal(&mut self, decimal: bool) {
        self.d = decimal;
    }

    pub fn flag_carry(&mut self, carry: bool) {
        self.c = carry;
    }

    pub fn flag_overflow(&mut self, overflow: bool) {
        self.o = overflow;
    }

    pub fn flag_break(&mut self, break_bool: bool) {
        self.b = break_bool;
    }

    pub fn flag_negative(&mut self, negative: bool) {
        self.n = negative;
    }

    pub fn flag_zero(&mut self, zero: bool) {
        self.z = zero;
    }

    pub fn flag_negative_from_val(&mut self, val: u8) {
        if (val as i8) < 0 {
            self.n = true;
        } else {
            self.n = false;
        }
    }

    pub fn flag_zero_from_val(&mut self, val: u8) {
        if val == 0 {
            self.z = true;
        } else {
            self.z = false;
        }
    }

    pub fn flag_overflow_from_vals_sbc(&mut self, b: u8, c: u8) {
        let a = self.a as i8;
        let b = b as i8;
        let c = c as i8;
        if (a >= 0 && b <= 0 && c <= 0) || (a <= 0 && b >= 0 && c >= 0) {
            self.o = true;
        } else {
            self.o = false;
        }
    }

    pub fn flag_overflow_from_vals_adc(&mut self, b: u8, c: u8) {
        let a = self.a as i8;
        let b = b as i8;
        let c = c as i8;
        if (a >= 0 && b >= 0 && c <= 0) || (a <= 0 && b <= 0 && c >= 0) {
            self.o = true;
        } else {
            self.o = false;
        }
    }

    // --------------- END --------------------

    #[rustfmt::skip]
    pub fn execute_instruction(&mut self, bus: &mut Bus) {
		// --------------- ADRESSING --------------------
        let target_addr = match self.addr {
            Addressing::IMP | Addressing::ACC => {self.pc += 1; 0},
            Addressing::IMM => { self.pc += 2; self.pc - 1 },
            Addressing::ZPG => { self.pc += 2; bus.read_single(self.pc) as u16 },
            Addressing::ZPX => { self.pc += 2; bus.read_single(self.pc).wrapping_add(self.x) as u16 },
            Addressing::ZPY => { self.pc += 2; bus.read_single(self.pc).wrapping_add(self.y) as u16 },
            Addressing::ABS => { self.pc += 3; bus.read_double(self.pc) as u16 } ,
            Addressing::ABX => { self.pc += 3; bus.read_double(self.pc).wrapping_add(self.x as u16) as u16 },
            Addressing::ABY => { self.pc += 3; bus.read_double(self.pc).wrapping_add(self.y as u16) as u16 },
            Addressing::IDX => {
                self.pc += 2;
                let inline_addr = bus.read_single(self.pc).wrapping_add(self.x);
                let low = bus.read_8(inline_addr);
                let high = bus.read_8(inline_addr.wrapping_add(1));
                combine_low_high(low, high)
            }
            Addressing::IDY => {
                self.pc += 2;
                let inline_addr = bus.read_single(self.pc);
                let low = bus.read_8(inline_addr);
                let high = bus.read_8(inline_addr.wrapping_add(1));
                combine_low_high(low, high).wrapping_add(self.y as u16)
            }
            Addressing::REL => { self.pc += 2; self.pc - 1 },
            Addressing::IND => {
                self.pc += 3;
                let inline_addr = bus.read_double(self.pc);
                let low = bus.read_16(inline_addr);
				let high_addr = ((inline_addr as u8).wrapping_add(1) as u16) | (inline_addr & 0xFF00);
                let high = bus.read_16(high_addr);
                combine_low_high(low, high)
            },
            _ => panic!("{}", ERR_ADDR),
        };

        let target_val = bus.read_16(target_addr);

		// --------------- INSTRUCTIONS --------------------
        match self.instr {
            Instructions::ADC => {
                let (result1, overflow1) = self.a.overflowing_add(target_val);
                let (result2, overflow2) = result1.overflowing_add(self.c as u8);
                self.flag_zero_from_val(result2); self.flag_negative_from_val(result2); self.flag_carry(overflow1 || overflow2); self.flag_overflow_from_vals_adc(target_val, result2);
                self.a = result2;
            }
            Instructions::SBC => {
                let (result1, overflow1) = self.a.overflowing_sub(target_val);
                let (result2, overflow2) = result1.overflowing_sub(!self.c as u8);
                self.flag_zero_from_val(result2); self.flag_negative_from_val(result2); self.flag_carry(!overflow1 && !overflow2); self.flag_overflow_from_vals_sbc(target_val, result2);
                self.a = result2;
            }
            Instructions::AND | Instructions::EOR | Instructions::ORA => {
                let op: &dyn Fn(u8, u8) -> u8 = match self.instr { Instructions::AND => &u8_and, Instructions::ORA => &u8_or, Instructions::EOR => &u8_xor, _ => panic!() };
                self.a = op(self.a, target_val);
                self.flag_zero_from_val(self.a); self.flag_negative_from_val(self.a);
            }
            Instructions::ASL | Instructions::LSR | Instructions::ROL | Instructions::ROR => {
                let op: &dyn Fn(u8) -> u8 = match self.instr { Instructions::ASL | Instructions::ROL => &u8_shl, Instructions::LSR | Instructions::ROR => &u8_shr, _ => panic!() };
                let test: u8 = match self.instr { Instructions::ASL | Instructions::ROL => 0x80, Instructions::LSR | Instructions::ROR => 0x01, _ => panic!() };
                if self.addr == Addressing::ACC {
                    let new_carry = self.a & test == test;
                    self.a = op(self.a);
					if self.instr == Instructions::ROL {
						self.a |= self.c as u8
					} else if self.instr == Instructions::ROR {
						self.a |= (self.c as u8) << 7
					}
                    self.flag_zero_from_val(self.a); self.flag_negative_from_val(self.a); self.flag_carry(new_carry)
                } else {
                    let new_carry = target_val & test == test;
                    let mut modified_val = op(target_val);
					if self.instr == Instructions::ROL {
						modified_val |= self.c as u8
					} else if self.instr == Instructions::ROR {
						modified_val |= (self.c as u8) << 7
					}
                    bus.write_16(target_addr, modified_val);
                    self.flag_zero_from_val(modified_val); self.flag_negative_from_val(modified_val); self.flag_carry(new_carry);
                }
            }
            Instructions::BCC | Instructions::BCS | Instructions::BEQ | Instructions::BMI | Instructions::BNE | Instructions::BPL | Instructions::BVC | Instructions::BVS => {
                let can_branch = match self.instr { Instructions::BCC => !self.c, Instructions::BCS => self.c, Instructions::BEQ => self.z, Instructions::BMI => self.n, Instructions::BNE => !self.z, Instructions::BPL => !self.n, Instructions::BVC => !self.o, Instructions::BVS => self.o, _ => panic!() };
                if can_branch {
                    self.pc = self.pc.wrapping_add(target_val as u16);
                }
            }
            Instructions::BIT => {
                let res = target_val & self.a;
                self.flag_zero_from_val(res); self.flag_negative_from_val(target_val); self.flag_overflow(target_val & 0b01000000 == 0b01000000);
            }
            Instructions::BRK => {
                self.stack_push_pc(bus);
                self.PHP(bus);
                self.pc = 0xFFFE;
                self.flag_break(true);
            }
            Instructions::CLC => self.flag_carry(false), Instructions::CLD => self.flag_decimal(false), Instructions::CLI => self.flag_interrupt(false), Instructions::CLV => self.flag_overflow(false),
            Instructions::SEC => self.flag_carry(true), Instructions::SED => self.flag_decimal(true), Instructions::SEI => self.flag_interrupt(true),
            Instructions::CMP | Instructions::CPX | Instructions::CPY => {
                let reg = match self.instr { Instructions::CMP => self.a, Instructions::CPX => self.x, Instructions::CPY => self.y, _ => panic!()};
                self.flag_negative_from_val(reg.wrapping_sub(target_val)); self.flag_zero(reg == target_val); self.flag_carry(reg >= target_val);
            }
            Instructions::DEC | Instructions::DEX | Instructions::DEY | Instructions::INC | Instructions::INX | Instructions::INY => {
                let res = match self.instr { Instructions::DEC => bus.DEC(target_addr), Instructions::INC => bus.INC(target_addr), Instructions::DEX => self.DEX(), Instructions::DEY => self.DEY(), Instructions::INX => self.INX(), Instructions::INY => self.INY(), _ => panic!() };
                self.flag_negative_from_val(res); self.flag_zero_from_val(res);
            }
            Instructions::JMP | Instructions::JSR => {
                if self.instr == Instructions::JSR { self.pc -= 1; self.stack_push_pc(bus); }
                self.pc = target_addr;
            }
            Instructions::RTS => {
                self.stack_pull_pc(bus);
				self.pc += 1;
            }
            Instructions::LDA | Instructions::LDX | Instructions::LDY => {
                match self.instr { Instructions::LDA => self.a = target_val, Instructions::LDX => self.x = target_val, Instructions::LDY => self.y = target_val, _ => panic!() };
                self.flag_zero_from_val(target_val); self.flag_negative_from_val(target_val);
            }
            Instructions::NOP => (),
            Instructions::PHA => {
                self.stack_push(self.a, bus);
            }
            Instructions::PLA => {
                self.a = self.stack_pull(bus);
				self.flag_zero_from_val(self.a); self.flag_negative_from_val(self.a)
            }
            Instructions::PHP => {
                self.PHP(bus);
            },
            Instructions::PLP => {
                self.PLP(bus);
            },
            Instructions::RTI => {
                self.PLP(bus);
                self.stack_pull_pc(bus);
            },
            Instructions::STA => {
				bus.write_16(target_addr, self.a);
            },
            Instructions::STX => {
				bus.write_16(target_addr, self.x);
            },
            Instructions::STY => {
				bus.write_16(target_addr, self.y);
            },
            Instructions::TAX | Instructions::TSX => {
                self.x = match self.instr { Instructions::TAX => self.a, Instructions::TSX => self.stack_pointer, _ => panic!() };
                self.flag_zero_from_val(self.x); self.flag_negative_from_val(self.x);
            },
            Instructions::TXA | Instructions::TYA => {
                self.a = match self.instr { Instructions::TXA => self.x, Instructions::TYA => self.y, _ => panic!() };
                self.flag_zero_from_val(self.a); self.flag_negative_from_val(self.a);
            },
            Instructions::TAY => {
                self.y = self.a;
                self.flag_zero_from_val(self.y); self.flag_negative_from_val(self.y);
            },
            Instructions::TXS => {
                self.stack_pointer = self.x;
            },
            _ => panic!("{}", ERR_OP),
        }
        println!("prev target val: {:02x}", target_val);
        println!("prev target addr: {:04x}", target_addr);
    }

    #[rustfmt::skip]
    pub fn load_instruction(&mut self, bus: &mut Bus, line: &StringRecord, cycles_total: u64) -> u8 {
        let cycles: u8;

        match bus.read_16(self.pc) {
            0x69 => {self.instr = Instructions::ADC; self.addr = Addressing::IMM; cycles = 2},
            0x65 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPG; cycles = 3},
            0x75 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPX; cycles = 4},
            0x6D => {self.instr = Instructions::ADC; self.addr = Addressing::ABS; cycles = 4},
            0x7D => {self.instr = Instructions::ADC; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x79 => {self.instr = Instructions::ADC; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0x61 => {self.instr = Instructions::ADC; self.addr = Addressing::IDX; cycles = 6},
            0x71 => {self.instr = Instructions::ADC; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0xE9 => {self.instr = Instructions::SBC; self.addr = Addressing::IMM; cycles = 2},
            0xE5 => {self.instr = Instructions::SBC; self.addr = Addressing::ZPG; cycles = 3},
            0xF5 => {self.instr = Instructions::SBC; self.addr = Addressing::ZPX; cycles = 4},
            0xED => {self.instr = Instructions::SBC; self.addr = Addressing::ABS; cycles = 4},
            0xFD => {self.instr = Instructions::SBC; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0xF9 => {self.instr = Instructions::SBC; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0xE1 => {self.instr = Instructions::SBC; self.addr = Addressing::IDX; cycles = 6},
            0xF1 => {self.instr = Instructions::SBC; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x29 => {self.instr = Instructions::AND; self.addr = Addressing::IMM; cycles = 2},
            0x25 => {self.instr = Instructions::AND; self.addr = Addressing::ZPG; cycles = 3},
            0x35 => {self.instr = Instructions::AND; self.addr = Addressing::ZPX; cycles = 4},
            0x2D => {self.instr = Instructions::AND; self.addr = Addressing::ABS; cycles = 4},
            0x3D => {self.instr = Instructions::AND; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x39 => {self.instr = Instructions::AND; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0x21 => {self.instr = Instructions::AND; self.addr = Addressing::IDX; cycles = 6},
            0x31 => {self.instr = Instructions::AND; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x09 => {self.instr = Instructions::ORA; self.addr = Addressing::IMM; cycles = 2},
            0x05 => {self.instr = Instructions::ORA; self.addr = Addressing::ZPG; cycles = 3},
            0x15 => {self.instr = Instructions::ORA; self.addr = Addressing::ZPX; cycles = 4},
            0x0D => {self.instr = Instructions::ORA; self.addr = Addressing::ABS; cycles = 4},
            0x1D => {self.instr = Instructions::ORA; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x19 => {self.instr = Instructions::ORA; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x01 => {self.instr = Instructions::ORA; self.addr = Addressing::IDX; cycles = 6},
            0x11 => {self.instr = Instructions::ORA; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x49 => {self.instr = Instructions::EOR; self.addr = Addressing::IMM; cycles = 2},
            0x45 => {self.instr = Instructions::EOR; self.addr = Addressing::ZPG; cycles = 3},
            0x55 => {self.instr = Instructions::EOR; self.addr = Addressing::ZPX; cycles = 4},
            0x4D => {self.instr = Instructions::EOR; self.addr = Addressing::ABS; cycles = 4},
            0x5D => {self.instr = Instructions::EOR; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x59 => {self.instr = Instructions::EOR; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x41 => {self.instr = Instructions::EOR; self.addr = Addressing::IDX; cycles = 6},
            0x51 => {self.instr = Instructions::EOR; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x0A => {self.instr = Instructions::ASL; self.addr = Addressing::ACC; cycles = 2},
            0x06 => {self.instr = Instructions::ASL; self.addr = Addressing::ZPG; cycles = 5},
            0x16 => {self.instr = Instructions::ASL; self.addr = Addressing::ZPX; cycles = 6},
            0x0E => {self.instr = Instructions::ASL; self.addr = Addressing::ABS; cycles = 6},
            0x1E => {self.instr = Instructions::ASL; self.addr = Addressing::ABX; cycles = 7},

            0x4A => {self.instr = Instructions::LSR; self.addr = Addressing::ACC; cycles = 2},
            0x46 => {self.instr = Instructions::LSR; self.addr = Addressing::ZPG; cycles = 5},
            0x56 => {self.instr = Instructions::LSR; self.addr = Addressing::ZPX; cycles = 6},
            0x4E => {self.instr = Instructions::LSR; self.addr = Addressing::ABS; cycles = 6},
            0x5E => {self.instr = Instructions::LSR; self.addr = Addressing::ABX; cycles = 7},

            0x2A => {self.instr = Instructions::ROL; self.addr = Addressing::ACC; cycles = 2},
            0x26 => {self.instr = Instructions::ROL; self.addr = Addressing::ZPG; cycles = 5},
            0x36 => {self.instr = Instructions::ROL; self.addr = Addressing::ZPX; cycles = 6},
            0x2E => {self.instr = Instructions::ROL; self.addr = Addressing::ABS; cycles = 6},
            0x3E => {self.instr = Instructions::ROL; self.addr = Addressing::ABX; cycles = 7},

            0x6A => {self.instr = Instructions::ROR; self.addr = Addressing::ACC; cycles = 2},
            0x66 => {self.instr = Instructions::ROR; self.addr = Addressing::ZPG; cycles = 5},
            0x76 => {self.instr = Instructions::ROR; self.addr = Addressing::ZPX; cycles = 6},
            0x6E => {self.instr = Instructions::ROR; self.addr = Addressing::ABS; cycles = 6},
            0x7E => {self.instr = Instructions::ROR; self.addr = Addressing::ABX; cycles = 7},

            0x90 => {self.instr = Instructions::BCC; self.addr = Addressing::REL; cycles = if self.c { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0xB0 => {self.instr = Instructions::BCS; self.addr = Addressing::REL; cycles = if !self.c { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0xF0 => {self.instr = Instructions::BEQ; self.addr = Addressing::REL; cycles = if !self.z { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0x30 => {self.instr = Instructions::BMI; self.addr = Addressing::REL; cycles = if !self.n { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0xD0 => {self.instr = Instructions::BNE; self.addr = Addressing::REL; cycles = if self.z { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0x10 => {self.instr = Instructions::BPL; self.addr = Addressing::REL; cycles = if self.n { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0x50 => {self.instr = Instructions::BVC; self.addr = Addressing::REL; cycles = if self.o { 2 } else { 3 + bus.cross_rel(self.pc) } },
            0x70 => {self.instr = Instructions::BVS; self.addr = Addressing::REL; cycles = if !self.o { 2 } else { 3 + bus.cross_rel(self.pc) } },

            0x24 => {self.instr = Instructions::BIT; self.addr = Addressing::ZPG; cycles = 3},
            0x2C => {self.instr = Instructions::BIT; self.addr = Addressing::ABS; cycles = 4},

            0x00 => {self.instr = Instructions::BRK; self.addr = Addressing::IMP; cycles = 7},
            0x40 => {self.instr = Instructions::RTI; self.addr = Addressing::IMP; cycles = 6},

            0x18 => {self.instr = Instructions::CLC; self.addr = Addressing::IMP; cycles = 2},
            0xD8 => {self.instr = Instructions::CLD; self.addr = Addressing::IMP; cycles = 2},
            0x58 => {self.instr = Instructions::CLI; self.addr = Addressing::IMP; cycles = 2},
            0xB8 => {self.instr = Instructions::CLV; self.addr = Addressing::IMP; cycles = 2},
            0x38 => {self.instr = Instructions::SEC; self.addr = Addressing::IMP; cycles = 2},
            0xF8 => {self.instr = Instructions::SED; self.addr = Addressing::IMP; cycles = 2},
            0x78 => {self.instr = Instructions::SEI; self.addr = Addressing::IMP; cycles = 2},

            0xC9 => {self.instr = Instructions::CMP; self.addr = Addressing::IMM; cycles = 2},
            0xC5 => {self.instr = Instructions::CMP; self.addr = Addressing::ZPG; cycles = 3},
            0xD5 => {self.instr = Instructions::CMP; self.addr = Addressing::ZPX; cycles = 4},
            0xCD => {self.instr = Instructions::CMP; self.addr = Addressing::ABS; cycles = 4},
            0xDD => {self.instr = Instructions::CMP; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0xD9 => {self.instr = Instructions::CMP; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0xC1 => {self.instr = Instructions::CMP; self.addr = Addressing::IDX; cycles = 6},
            0xD1 => {self.instr = Instructions::CMP; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0xE0 => {self.instr = Instructions::CPX; self.addr = Addressing::IMM; cycles = 2},
            0xE4 => {self.instr = Instructions::CPX; self.addr = Addressing::ZPG; cycles = 3},
            0xEC => {self.instr = Instructions::CPX; self.addr = Addressing::ABS; cycles = 4},

            0xC0 => {self.instr = Instructions::CPY; self.addr = Addressing::IMM; cycles = 2},
            0xC4 => {self.instr = Instructions::CPY; self.addr = Addressing::ZPG; cycles = 3},
            0xCC => {self.instr = Instructions::CPY; self.addr = Addressing::ABS; cycles = 4},

            0xC6 => {self.instr = Instructions::DEC; self.addr = Addressing::ZPG; cycles = 5},
            0xD6 => {self.instr = Instructions::DEC; self.addr = Addressing::ZPX; cycles = 6},
            0xCE => {self.instr = Instructions::DEC; self.addr = Addressing::ABS; cycles = 6},
            0xDE => {self.instr = Instructions::DEC; self.addr = Addressing::ABX; cycles = 7},

            0xE6 => {self.instr = Instructions::INC; self.addr = Addressing::ZPG; cycles = 5},
            0xF6 => {self.instr = Instructions::INC; self.addr = Addressing::ZPX; cycles = 6},
            0xEE => {self.instr = Instructions::INC; self.addr = Addressing::ABS; cycles = 6},
            0xFE => {self.instr = Instructions::INC; self.addr = Addressing::ABX; cycles = 7},

            0xE8 => {self.instr = Instructions::INX; self.addr = Addressing::IMP; cycles = 2},
            0xC8 => {self.instr = Instructions::INY; self.addr = Addressing::IMP; cycles = 2},

            0xCA => {self.instr = Instructions::DEX; self.addr = Addressing::IMP; cycles = 2},
            0x88 => {self.instr = Instructions::DEY; self.addr = Addressing::IMP; cycles = 2},

            0x4C => {self.instr = Instructions::JMP; self.addr = Addressing::ABS; cycles = 3},
            0x6C => {self.instr = Instructions::JMP; self.addr = Addressing::IND; cycles = 5},

            0x20 => {self.instr = Instructions::JSR; self.addr = Addressing::ABS; cycles = 6},

            0x60 => {self.instr = Instructions::RTS; self.addr = Addressing::IMP; cycles = 6},

            0xA9 => {self.instr = Instructions::LDA; self.addr = Addressing::IMM; cycles = 2},
            0xA5 => {self.instr = Instructions::LDA; self.addr = Addressing::ZPG; cycles = 3},
            0xB5 => {self.instr = Instructions::LDA; self.addr = Addressing::ZPX; cycles = 4},
            0xAD => {self.instr = Instructions::LDA; self.addr = Addressing::ABS; cycles = 4},
            0xBD => {self.instr = Instructions::LDA; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0xB9 => {self.instr = Instructions::LDA; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0xA1 => {self.instr = Instructions::LDA; self.addr = Addressing::IDX; cycles = 6},
            0xB1 => {self.instr = Instructions::LDA; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0xA0 => {self.instr = Instructions::LDY; self.addr = Addressing::IMM; cycles = 2},
            0xA4 => {self.instr = Instructions::LDY; self.addr = Addressing::ZPG; cycles = 3},
            0xB4 => {self.instr = Instructions::LDY; self.addr = Addressing::ZPX; cycles = 4},
            0xAC => {self.instr = Instructions::LDY; self.addr = Addressing::ABS; cycles = 4},
            0xBC => {self.instr = Instructions::LDY; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},

            0xA2 => {self.instr = Instructions::LDX; self.addr = Addressing::IMM; cycles = 2},
            0xA6 => {self.instr = Instructions::LDX; self.addr = Addressing::ZPG; cycles = 3},
            0xB6 => {self.instr = Instructions::LDX; self.addr = Addressing::ZPY; cycles = 4},
            0xAE => {self.instr = Instructions::LDX; self.addr = Addressing::ABS; cycles = 4},
            0xBE => {self.instr = Instructions::LDX; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},

            0xEA => {self.instr = Instructions::NOP; self.addr = Addressing::IMP; cycles = 2},

            0x48 => {self.instr = Instructions::PHA; self.addr = Addressing::IMP; cycles = 3},
            0x68 => {self.instr = Instructions::PLA; self.addr = Addressing::IMP; cycles = 4},
            0x08 => {self.instr = Instructions::PHP; self.addr = Addressing::IMP; cycles = 3},
            0x28 => {self.instr = Instructions::PLP; self.addr = Addressing::IMP; cycles = 4},

            0x85 => {self.instr = Instructions::STA; self.addr = Addressing::ZPG; cycles = 3},
            0x95 => {self.instr = Instructions::STA; self.addr = Addressing::ZPX; cycles = 4},
            0x8D => {self.instr = Instructions::STA; self.addr = Addressing::ABS; cycles = 4},
            0x9D => {self.instr = Instructions::STA; self.addr = Addressing::ABX; cycles = 5},
            0x99 => {self.instr = Instructions::STA; self.addr = Addressing::ABY; cycles = 5},
            0x81 => {self.instr = Instructions::STA; self.addr = Addressing::IDX; cycles = 6},
            0x91 => {self.instr = Instructions::STA; self.addr = Addressing::IDY; cycles = 6},

            0x86 => {self.instr = Instructions::STX; self.addr = Addressing::ZPG; cycles = 3},
            0x96 => {self.instr = Instructions::STX; self.addr = Addressing::ZPY; cycles = 4},
            0x8E => {self.instr = Instructions::STX; self.addr = Addressing::ABS; cycles = 4},

            0x84 => {self.instr = Instructions::STY; self.addr = Addressing::ZPG; cycles = 3},
            0x94 => {self.instr = Instructions::STY; self.addr = Addressing::ZPX; cycles = 4},
            0x8C => {self.instr = Instructions::STY; self.addr = Addressing::ABS; cycles = 4},

            0xAA => {self.instr = Instructions::TAX; self.addr = Addressing::IMP; cycles = 2},
            0xA8 => {self.instr = Instructions::TAY; self.addr = Addressing::IMP; cycles = 2},
            0x8A => {self.instr = Instructions::TXA; self.addr = Addressing::IMP; cycles = 2},
            0x98 => {self.instr = Instructions::TYA; self.addr = Addressing::IMP; cycles = 2},

            0xBA => {self.instr = Instructions::TSX; self.addr = Addressing::IMP; cycles = 2},
            0x9A => {self.instr = Instructions::TXS; self.addr = Addressing::IMP; cycles = 2},
            _ => panic!("{}", ERR_OP),
        }

        println!("instruction: {:?}", self.instr);
        println!("addressing mode: {:?}", self.addr);

		let p = self.flags_to_byte();
		let (true_cyc, my_cyc) = self.print_debug_int(&line[13], cycles_total, "cyc");
		let (true_p, my_p) = self.print_debug_8(&line[9], p, "p");
		let (true_sp, my_sp) = self.print_debug_8(&line[10], self.stack_pointer, "sp");
		let (true_a, my_a) = self.print_debug_8(&line[6], self.a, "a");
		let (true_x, my_x) = self.print_debug_8(&line[7], self.x, "x");
		let (true_y, my_y) = self.print_debug_8(&line[8], self.y, "y");
		let (true_addr, my_addr) = self.print_debug_16(&line[0], self.pc, "pc");

		let abs_pointer = 1 << 8 | self.stack_pointer as u16;
		for n in abs_pointer - 5 .. abs_pointer + 5 {
			print!(" {:02x} {:02x} ", n, bus.read_16(1 << 8 | n as u16));
			println!();
		}

		self.check_debug(true_p, my_p, "p");
		self.check_debug(true_addr, my_addr, "addr");
		self.check_debug(true_a, my_a, "a");
		self.check_debug(true_x, my_x, "x");
		self.check_debug(true_y, my_y, "y");
		self.check_debug(true_sp, my_sp, "sp");
		self.check_debug(true_cyc, my_cyc, "cyc");

        cycles
    }

    pub fn check_debug(&mut self, true_val: String, my_val: String, name: &str) {
        if true_val != my_val {
            panic!("mismatched {}", name);
        }
    }

    pub fn print_debug(&mut self, true_val: &str, my_val: String, name: &str) -> (String, String) {
        let true_val = true_val.to_uppercase();
        println!("true_{}, my_{}: {}, {}", name, name, true_val, my_val);
        (true_val, my_val)
    }

    pub fn print_debug_8(&mut self, true_val: &str, my_val: u8, name: &str) -> (String, String) {
        let my_val = format!("{:02x}", my_val).to_uppercase();
        self.print_debug(true_val, my_val, name)
    }

    pub fn print_debug_16(&mut self, true_val: &str, my_val: u16, name: &str) -> (String, String) {
        let my_val = format!("{:04x}", my_val).to_uppercase();
        self.print_debug(true_val, my_val, name)
    }

    pub fn print_debug_int(&mut self, true_val: &str, my_val: u64, name: &str) -> (String, String) {
        self.print_debug(true_val, my_val.to_string(), name)
    }
}
