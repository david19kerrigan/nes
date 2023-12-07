use crate::util::*;
use crate::Bus;

const ERR_INSTR: &str = "Invalid Instruction";
const ERR_OP: &str = "Invalid Opcode";
const ERR_ADDR: &str = "Invalid Addressing Mode";

#[rustfmt::skip]
#[derive(PartialEq)]
enum Instructions {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

#[rustfmt::skip]
#[derive(PartialEq)]
enum Addressing {
    IMP, ACC, IMM, ZPG, ZPX, ZPY, REL, ABS, ABX, ABY, IND, IDX, IDY,
}

pub struct Cpu {
    // registers
    a: u8,
    x: u8,
    y: u8,
    pc: usize,
    s: u8,
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
    stack: Vec<usize>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            // registers
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0,
            //flags
            c: false,
            z: false,
            i: false,
            d: false,
            b: false,
            o: false,
            n: false,
            // misc
            instr: Instructions::ADC,
            addr: Addressing::IMM,
            stack: vec![],
        }
    }

    // --------------- FLAGS --------------------

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

    pub fn flag_negative(&mut self, val: u8) {
        if (val as i8) < 0 {
            self.n = true;
        }
    }

    pub fn flag_zero(&mut self, val: u8) {
        if val == 0 {
            self.z = true;
        }
    }

    pub fn flag_overflow_adding(&mut self, a: u8, b: u8, c: u8) {
        if (self.a > 0 && b > 0 && (c as i8) < 0) || (self.a < 0 && b < 0 && (c as i8) > 0) {
            self.o = true;
        }
    }

    // --------------- OPERATIONS --------------------

    //pub fn INC(&mut self, memory: &mut Vec<u8>) {
    //    memory[self.ram] -= 1;
    //    let res = memory[self.ram];
    //    self.flag_zero(res);
    //    self.flag_negative(res);
    //}

    //pub fn DEC(&mut self, memory: &mut Vec<u8>) {
    //    memory[self.ram] -= 1;
    //    let res = memory[self.ram];
    //    self.flag_zero(res);
    //    self.flag_negative(res);
    //}

    //pub fn CMP(&mut self, a: u8, val: u8) {
    //    let temp = a - val;
    //    self.flag_negative(temp);
    //    self.flag_carry(temp >= 0);
    //    self.flag_zero(temp);
    //}

    // --------------- ADDRESSING --------------------

    pub fn ID(&mut self, memory: &mut Vec<u8>) -> (usize) {
        self.pc += 3;
        (memory[self.pc - 2] as u16 | (memory[self.pc - 1] as u16) << 8) as usize
    }

    pub fn REL(&mut self, memory: &mut Vec<u8>, can_branch: bool) -> (isize, u8) {
        self.pc += 2;
        let val = memory[self.pc - 1] as isize;
        if self.pc.overflowing_add_signed(val).1 && can_branch {
            (val, 4)
        } else if can_branch {
            (val, 3)
        } else {
            (0, 2)
        }
    }

    // --------------- END --------------------

    #[rustfmt::skip]
    pub fn execute_instruction(&mut self, bus: &mut Bus) {
        let target_addr = match self.addr {
            Addressing::ACC => { self.pc += 1; 0 },
            Addressing::IMP => { self.pc += 1; 0 },
            Addressing::IMM => { self.pc += 2; self.pc - 1 as usize },
            Addressing::ZPG => { self.pc += 2; bus.read_single(self.pc) as usize },
            Addressing::ZPX => { self.pc += 2; bus.read_single(self.pc).wrapping_add(self.x) as usize },
            Addressing::ABS => { self.pc += 3; bus.read_double(self.pc) as usize } ,
            Addressing::ABX => { self.pc += 3; bus.read_double(self.pc).wrapping_add(self.x as u16) as usize },
            Addressing::ABY => { self.pc += 3; bus.read_double(self.pc).wrapping_add(self.y as u16) as usize },
            Addressing::IDX => {
                self.pc += 2;
                let zp = bus.read_single(self.pc).wrapping_add(self.x);
                combine_low_high(zp, zp + 1) as usize
            }
            Addressing::IDY => {
                self.pc += 2;
                let zp = bus.read_single(self.pc);
                combine_low_high(zp, zp + 1).wrapping_add(self.y as u16) as usize
            }
            Addressing::REL => { self.pc += 2; self.pc - 1 as usize},
            _ => panic!("{}", ERR_ADDR),
        };

        let mut target_val = bus.read_usize(target_addr);

        match self.instr {
            Instructions::ADC | Instructions::SBC => {
                let op: &dyn Fn(u8, u8) -> (u8, bool) = match self.instr { Instructions::ADC => &u8::overflowing_add, Instructions::SBC => &u8::overflowing_sub, _ => panic!("{}", ERR_INSTR) };
                let (val_with_carry, overflow1) = op(target_val, self.c as u8);
                let (result, overflow2) =  op(self.a, val_with_carry);
                self.flag_zero(result); self.flag_negative(result); self.flag_carry(overflow1 || overflow2); self.flag_overflow_adding(self.a, val_with_carry, result);
                self.a = result;
            }
            Instructions::AND | Instructions::EOR | Instructions::ORA => {
                let op: &dyn Fn(u8, u8) -> (u8) = match self.instr { Instructions::AND => &u8_and, Instructions::ORA => &u8_or, _ => panic!("{}", ERR_INSTR) };
                self.a = op(self.a, target_val);
                self.flag_zero(self.a); self.flag_negative(self.a);
            }
            Instructions::ASL | Instructions::LSR | Instructions::ROL | Instructions::ROR => {
                let op: &dyn Fn(u8) -> (u8) = match self.instr { Instructions::ASL | Instructions::ROL => &u8_shl, Instructions::LSR | Instructions::ROR => &u8_shr, _ => panic!("{}", ERR_INSTR) };
                if self.addr == Addressing::ACC {
                    self.flag_carry(self.a & 0x80 == 1);
                    self.a = op(self.a);
                    match self.instr { Instructions::ROL | Instructions::ROR => self.a |= self.c as u8, _ => () };
                    self.flag_zero(self.a); self.flag_negative(self.a);
                } else {
                    self.flag_carry(target_val & 0x80 == 1);
                    let mut modified_val = op(target_val);
                    match self.instr { Instructions::ROL | Instructions::ROR => modified_val |= self.c as u8, _ => () };
                    bus.write_usize(target_addr, modified_val);
                    self.flag_zero(modified_val); self.flag_negative(modified_val);
                }
            }
            Instructions::BCC | Instructions::BCS | Instructions::BEQ | Instructions::BMI | Instructions::BMI | Instructions::BNE | Instructions::BPL | Instructions::BVC | Instructions::BVS => {
                let can_branch = match self.instr { Instructions::BCC => !self.c, Instructions::BCS => self.c, Instructions::BEQ => self.z, Instructions::BMI => self.n, Instructions::BNE => !self.z, Instructions::BPL => !self.n, Instructions::BVC => !self.o, Instructions::BVS => self.o, _ => panic!("{}", ERR_INSTR) }; 
                if can_branch {
                    self.pc = self.pc.wrapping_add(target_val as usize);
                }
            }
            Instructions::BIT => {
                let res = target_val & self.a;
                self.flag_zero(res); self.flag_negative(res); self.flag_overflow(res & 0x40 == 1);
            }
            Instructions::BRK => {
                self.stack.push(self.pc);
                self.pc = 0xFFFE;
                self.flag_break(true);
            }
            Instructions::CLC => self.flag_carry(false), Instructions::CLD => self.flag_decimal(false), Instructions::CLI => self.flag_interrupt(false), Instructions::CLV => self.flag_overflow(false),
            Instructions::SEC => self.flag_carry(true), Instructions::SED => self.flag_decimal(true), Instructions::SEI => self.flag_interrupt(true),
            _ => panic!("{}", ERR_OP),
        }
    }

    #[rustfmt::skip]
    pub fn load_instruction(&mut self, bus: &mut Bus) -> u8 {
        let cycles: u8;

        match bus.read_usize(self.pc) {
            0x69 => {self.instr = Instructions::ADC; self.addr = Addressing::IMM; cycles = 2},
            0x65 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPG; cycles = 3},
            0x75 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPX; cycles = 4},
            0x6D => {self.instr = Instructions::ADC; self.addr = Addressing::ABS; cycles = 4},
            0x7D => {self.instr = Instructions::ADC; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x79 => {self.instr = Instructions::ADC; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0x61 => {self.instr = Instructions::ADC; self.addr = Addressing::IDX; cycles = 6},
            0x71 => {self.instr = Instructions::ADC; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x29 => {self.instr = Instructions::AND; self.addr = Addressing::IMM; cycles = 2},
            0x25 => {self.instr = Instructions::AND; self.addr = Addressing::ZPG; cycles = 3},
            0x35 => {self.instr = Instructions::AND; self.addr = Addressing::ZPX; cycles = 4},
            0x2D => {self.instr = Instructions::AND; self.addr = Addressing::ABS; cycles = 4},
            0x3D => {self.instr = Instructions::AND; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc, self.x)},
            0x39 => {self.instr = Instructions::AND; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc, self.y)},
            0x21 => {self.instr = Instructions::AND; self.addr = Addressing::IDX; cycles = 6},
            0x31 => {self.instr = Instructions::AND; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc, self.y)},

            0x0A => {self.instr = Instructions::ASL; self.addr = Addressing::ACC; cycles = 2},
            0x06 => {self.instr = Instructions::ASL; self.addr = Addressing::ZPG; cycles = 5},
            0x16 => {self.instr = Instructions::ASL; self.addr = Addressing::ZPX; cycles = 6},
            0x0E => {self.instr = Instructions::ASL; self.addr = Addressing::ABS; cycles = 6},
            0x1E => {self.instr = Instructions::ASL; self.addr = Addressing::ABX; cycles = 7},

            0x90 => {self.instr = Instructions::BCC; self.addr = Addressing::REL; cycles = 2 + !self.c as u8 + bus.cross_rel(self.pc)},
            0xB0 => {self.instr = Instructions::BCS; self.addr = Addressing::REL; cycles = 2 + self.c as u8 + bus.cross_rel(self.pc)},
            0xF0 => {self.instr = Instructions::BEQ; self.addr = Addressing::REL; cycles = 2 + self.z as u8 + bus.cross_rel(self.pc)},
            0x30 => {self.instr = Instructions::BMI; self.addr = Addressing::REL; cycles = 2 + self.n as u8 + bus.cross_rel(self.pc)},
            0xD0 => {self.instr = Instructions::BNE; self.addr = Addressing::REL; cycles = 2 + !self.z as u8 + bus.cross_rel(self.pc)},
            0x10 => {self.instr = Instructions::BPL; self.addr = Addressing::REL; cycles = 2 + !self.n as u8 + bus.cross_rel(self.pc)},
            0x50 => {self.instr = Instructions::BVC; self.addr = Addressing::REL; cycles = 2 + !self.o as u8 + bus.cross_rel(self.pc)},
            0x70 => {self.instr = Instructions::BVS; self.addr = Addressing::REL; cycles = 2 + self.o as u8 + bus.cross_rel(self.pc)},

            0x24 => {self.instr = Instructions::BIT; self.addr = Addressing::ZPG; cycles = 3},
            0x2C => {self.instr = Instructions::BIT; self.addr = Addressing::ABS; cycles = 4},

            0x00 => {self.instr = Instructions::BRK; self.addr = Addressing::IMP; cycles = 7},

            0x50 => {self.instr = Instructions::CLC; self.addr = Addressing::IMP; cycles = 2},
            0xD8 => {self.instr = Instructions::CLD; self.addr = Addressing::IMP; cycles = 2},
            0x58 => {self.instr = Instructions::CLI; self.addr = Addressing::IMP; cycles = 2},
            0xB8 => {self.instr = Instructions::CLV; self.addr = Addressing::IMP; cycles = 2},
            0x38 => {self.instr = Instructions::SEC; self.addr = Addressing::IMP; cycles = 2},
            0xF8 => {self.instr = Instructions::SED; self.addr = Addressing::IMP; cycles = 2},
            0x78 => {self.instr = Instructions::SEI; self.addr = Addressing::IMP; cycles = 2},
            _ => panic!("{}", ERR_OP),
        }
        cycles
    }
}
