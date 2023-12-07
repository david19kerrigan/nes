use crate::util::*;
use crate::Bus;

#[rustfmt::skip]
enum Instructions {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

#[rustfmt::skip]
enum Addressing {
    IMP, ACC, IMM, ZPG, ZPX, ZPY, REL, ABS, ABX, ABY, IND, IDX, IDY,
}

struct Instruction {
    opcode: Instructions,
    addressing: Addressing,
    cycles: u8,
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
    cycles: u8,
    ram: usize,
    st: Vec<u16>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0,
            c: false,
            z: false,
            i: false,
            d: false,
            b: false,
            o: false,
            n: false,
            instr: Instructions::ADC,
            addr: Addressing::IMM,
            cycles: 0,
            ram: 0,
            st: vec![],
        }
    }

    // --------------- FLAGS --------------------

    pub fn flag_carry(&mut self, carry: bool) {
        self.c = carry;
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

    pub fn flag_overflow(&mut self, a: u8, b: u8) {
        if (self.a > 0 && b > 0 && (self.a as i8) < 0)
            || (self.a < 0 && b < 0 && (self.a as i8) > 0)
        {
            self.o = true;
        }
    }

    // --------------- OPERATIONS --------------------

    pub fn add_and_set_flags(&mut self, a: u8, b: u8) -> u8 {
        let (result, overflow) = a.overflowing_add(b);
        self.flag_zero(result);
        self.flag_negative(result);
        self.flag_carry(overflow);
        self.flag_overflow(a, b);
        result
    }

    pub fn EOR(&mut self, val: u8) {
        self.a ^= val;
        self.flag_zero(self.a);
        self.flag_negative(self.a);
    }

    pub fn INC(&mut self, memory: &mut Vec<u8>) {
        memory[self.ram] -= 1;
        let res = memory[self.ram];
        self.flag_zero(res);
        self.flag_negative(res);
    }

    pub fn DEC(&mut self, memory: &mut Vec<u8>) {
        memory[self.ram] -= 1;
        let res = memory[self.ram];
        self.flag_zero(res);
        self.flag_negative(res);
    }

    pub fn CMP(&mut self, a: u8, val: u8) {
        let temp = a - val;
        self.flag_negative(temp);
        self.flag_carry(temp >= 0);
        self.flag_zero(temp);
    }

    pub fn BIT(&mut self, val: u8) {
        let res = val & self.a;
        self.flag_zero(res);
        self.flag_negative(res);
        self.o = res & 0x40 == 1;
    }

    pub fn BXX(&mut self, val: isize) {
        self.pc = (self.pc).wrapping_add_signed(val) as usize;
    }

    pub fn ASLA(&mut self) {
        self.flag_carry(self.a & 0x80 == 1);
        self.a <<= 1;
        self.flag_zero(self.a);
        self.flag_negative(self.a);
        self.pc += 1;
    }

    pub fn ASLM(&mut self, memory: &mut Vec<u8>) {
        self.flag_carry(memory[self.ram as usize] & 0x80 == 1);
        memory[self.ram as usize] <<= 1;
        self.flag_zero(memory[self.ram as usize]);
        self.flag_negative(memory[self.ram as usize]);
    }

    pub fn AND(&mut self, val: u8) {
        self.a |= val;
        self.flag_zero(self.a);
        self.flag_negative(self.a);
    }

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

    pub fn execute_instruction(&mut self, bus: &mut Bus) {
        let target_addr = match self.addr {
            Addressing::IMM => self.pc - 1 as usize,
            Addressing::ZPG => bus.read_single(self.pc) as usize,
            Addressing::ZPX => bus.read_single(self.pc).wrapping_add(self.x) as usize,
            Addressing::ABS => bus.read_double(self.pc) as usize,
            Addressing::ABX => bus.read_double(self.pc).wrapping_add(self.x as u16) as usize,
            Addressing::ABY => bus.read_double(self.pc).wrapping_add(self.y as u16) as usize,
            Addressing::IDX => {
                let zp = bus.read_single(self.pc).wrapping_add(self.x);
                combine_low_high(zp, zp + 1) as usize
            }
            Addressing::IDY => {
                let zp = bus.read_8(bus.memory[self.pc]);
                combine_low_high(zp, zp + 1).wrapping_add(self.y as u16) as usize
            }
            _ => panic!("invalid address mode"),
        };

        match self.instr {
            Instructions::ADC => {
                let val = bus.memory[target_addr];
                let (val_with_carry, overflow) = val.overflowing_add(self.c as u8);
                self.a = self.add_and_set_flags(self.a, val_with_carry);
                self.flag_carry(self.c | overflow);
            }
            _ => (),
        }
        //match &self.instr {
        //    Instructions::ADC => self.ADC(memory[self.ram]),
        //    Instructions::BIT => self.BIT(memory[self.ram]),
        //    Instructions::CMP => self.CMP(self.a, memory[self.ram]),
        //    Instructions::CPX => self.CMP(self.x, memory[self.ram]),
        //    Instructions::CPY => self.CMP(self.y, memory[self.ram]),
        //    Instructions::AND => self.AND(memory[self.ram]),
        //    Instructions::ASLM => self.ASLM(memory),
        //    Instructions::DEC => self.DEC(memory),
        //    Instructions::INC => self.INC(memory),
        //    Instructions::EOR => self.EOR(memory[self.ram]),
        //    Instructions::NONE => (),
        //    default => panic!("invalid instruction"),
        //}
        //self.instr = Instructions::NONE;
    }

    #[rustfmt::skip]
    pub fn load_instruction(&mut self, bus: &mut Bus) -> u8 {
        match bus.memory[self.pc] {
            0x69 => {self.instr = Instructions::ADC; self.addr = Addressing::IMM; self.cycles = 2; self.pc += 2},
            0x65 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPG; self.cycles = 3; self.pc += 2},
            0x75 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPX; self.cycles = 4; self.pc += 2},
            0x6D => {self.instr = Instructions::ADC; self.addr = Addressing::ABS; self.cycles = 4; self.pc += 2},
            0x7D => {self.instr = Instructions::ADC; self.addr = Addressing::ABX; self.cycles = 4 + bus.cross_abs(self.pc + 1, self.pc + 2, self.x); self.pc += 3},
            0x79 => {self.instr = Instructions::ADC; self.addr = Addressing::ABY; self.cycles = 4 + bus.cross_abs(self.pc + 1, self.pc + 2, self.y); self.pc += 3},
            0x61 => {self.instr = Instructions::ADC; self.addr = Addressing::IDX; self.cycles = 6; self.pc += 2},
            0x71 => {self.instr = Instructions::ADC; self.addr = Addressing::IDY; self.cycles = 5 + bus.cross_idy(self.pc + 1, self.y); self.pc += 2},
            _ => (),
        }
        self.cycles

        //match opcode {
        //    // --------------- JMP --------------------
        //    0x6C => {
        //        self.pc = self.ID(memory);
        //        5
        //    }
        //    0x4C => {
        //        self.pc = self.ABS(memory);
        //        3
        //    }
        //    // --------------- INC --------------------
        //    0xE6 => {
        //        self.instr = Instructions::INC;
        //        self.ram = self.ZP(memory);
        //        5
        //    }
        //    0xF6 => {
        //        self.instr = Instructions::INC;
        //        self.ram = self.ZPX(memory);
        //        6
        //    }
        //    0xEE => {
        //        self.instr = Instructions::INC;
        //        self.ram = self.ABS(memory);
        //        6
        //    }
        //    0xFE => {
        //        self.instr = Instructions::INC;
        //        self.ram = self.ABX(memory).0;
        //        7
        //    }
        //    // --------------- EOR --------------------
        //    0x51 => {
        //        self.instr = Instructions::EOR;
        //        let (val, overflow) = self.IDY(memory);
        //        self.ram = val;
        //        5 + overflow as u8
        //    }
        //    0x41 => {
        //        self.instr = Instructions::EOR;
        //        self.ram = self.IDX(memory);
        //        6
        //    }
        //    0x59 => {
        //        self.instr = Instructions::EOR;
        //        let (val, overflow) = self.ABY(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x5D => {
        //        self.instr = Instructions::EOR;
        //        let (val, overflow) = self.ABX(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x4D => {
        //        self.instr = Instructions::EOR;
        //        self.ram = self.ABS(memory);
        //        4
        //    }
        //    0x55 => {
        //        self.instr = Instructions::EOR;
        //        self.ram = self.ZPX(memory);
        //        4
        //    }
        //    0x45 => {
        //        self.instr = Instructions::EOR;
        //        self.ram = self.ZP(memory);
        //        3
        //    }
        //    0x49 => {
        //        let val = self.IMM(memory);
        //        self.EOR(val);
        //        1 // 2
        //    }
        //    // --------------- INX --------------------
        //    0xE8 => {
        //        self.x += 1;
        //        self.pc += 1;
        //        self.flag_zero(self.x);
        //        self.flag_negative(self.x);
        //        1 // 2
        //    }
        //    // --------------- INY --------------------
        //    0xC8 => {
        //        self.y += 1;
        //        self.pc += 1;
        //        self.flag_zero(self.y);
        //        self.flag_negative(self.y);
        //        1 // 2
        //    }
        //    // --------------- DEY --------------------
        //    0x88 => {
        //        self.y -= 1;
        //        self.pc += 1;
        //        self.flag_zero(self.y);
        //        self.flag_negative(self.y);
        //        1 // 2
        //    }
        //    // --------------- DEX --------------------
        //    0xCA => {
        //        self.x -= 1;
        //        self.pc += 1;
        //        self.flag_zero(self.x);
        //        self.flag_negative(self.x);
        //        1 // 2
        //    }
        //    // --------------- DEC --------------------
        //    0xC6 => {
        //        self.instr = Instructions::DEC;
        //        self.ram = self.ZP(memory);
        //        5
        //    }
        //    0xD6 => {
        //        self.instr = Instructions::DEC;
        //        self.ram = self.ZPX(memory);
        //        6
        //    }
        //    0xCE => {
        //        self.instr = Instructions::DEC;
        //        self.ram = self.ABS(memory);
        //        6
        //    }
        //    0xDE => {
        //        self.instr = Instructions::DEC;
        //        self.ram = self.ABX(memory).0;
        //        7
        //    }
        //    // --------------- CPY --------------------
        //    0xC0 => {
        //        self.instr = Instructions::CPY;
        //        self.ram = self.ABS(memory);
        //        4
        //    }
        //    0xC4 => {
        //        self.instr = Instructions::CPY;
        //        self.ram = self.ZP(memory);
        //        3
        //    }
        //    0xCC => {
        //        let val = self.IMM(memory);
        //        self.CMP(self.y, val);
        //        1 // 2
        //    }
        //    // --------------- CPX --------------------
        //    0xEC => {
        //        self.instr = Instructions::CPX;
        //        let val = self.ABS(memory);
        //        self.ram = val;
        //        4
        //    }
        //    0xE4 => {
        //        self.instr = Instructions::CPX;
        //        let val = self.ZP(memory);
        //        self.ram = val;
        //        3
        //    }
        //    0xE0 => {
        //        let val = self.IMM(memory);
        //        self.CMP(self.x, val);
        //        1 // 2
        //    }
        //    // --------------- CMP --------------------
        //    0xD1 => {
        //        self.instr = Instructions::CMP;
        //        let (val, overflow) = self.IDY(memory);
        //        self.ram = val;
        //        5 + overflow as u8
        //    }
        //    0xC1 => {
        //        self.instr = Instructions::CMP;
        //        self.ram = self.IDX(memory);
        //        6
        //    }
        //    0xD9 => {
        //        self.instr = Instructions::CMP;
        //        let (val, overflow) = self.ABY(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0xDD => {
        //        self.instr = Instructions::CMP;
        //        let (val, overflow) = self.ABX(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0xCD => {
        //        self.instr = Instructions::CMP;
        //        self.ram = self.ABS(memory);
        //        4
        //    }
        //    0xD5 => {
        //        self.instr = Instructions::CMP;
        //        self.ram = self.ZPX(memory);
        //        4
        //    }
        //    0xC5 => {
        //        self.instr = Instructions::CMP;
        //        self.ram = self.ZP(memory);
        //        3
        //    }
        //    0xC9 => {
        //        let val = self.IMM(memory);
        //        self.CMP(self.a, val);
        //        1 // 2
        //    }
        //    // --------------- CLEAR --------------------
        //    0xB8 => {
        //        self.o = false;
        //        1 // 2
        //    }
        //    0x58 => {
        //        self.i = false;
        //        1 // 2
        //    }
        //    0xD8 => {
        //        self.d = false;
        //        1 // 2
        //    }
        //    0x18 => {
        //        self.c = false;
        //        1 // 2
        //    }
        //    // --------------- BRK --------------------
        //    0x00 => {
        //        self.st.push(self.pc as u16);
        //        self.pc = (memory[0xFFFE as usize] | memory[0xFFFF as usize] << 8) as usize;
        //        self.b = true;
        //        6 // 7
        //    }
        //    // --------------- BIT --------------------
        //    0x24 => {
        //        self.instr = Instructions::BIT;
        //        self.ram = self.ZP(memory);
        //        3
        //    }
        //    // --------------- BRANCH --------------------
        //    0x70 => {
        //        let (val, cycles) = self.REL(memory, self.o);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0x50 => {
        //        let (val, cycles) = self.REL(memory, !self.o);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0x10 => {
        //        let (val, cycles) = self.REL(memory, !self.n);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0xD0 => {
        //        let (val, cycles) = self.REL(memory, !self.z);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0x30 => {
        //        let (val, cycles) = self.REL(memory, self.n);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0xF0 => {
        //        let (val, cycles) = self.REL(memory, self.z);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0xB0 => {
        //        let (val, cycles) = self.REL(memory, self.c);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    0x90 => {
        //        let (val, cycles) = self.REL(memory, !self.c);
        //        self.BXX(val);
        //        cycles - 1
        //    }
        //    // --------------- ASL --------------------
        //    0x0E => {
        //        self.instr = Instructions::ASLM;
        //        self.ram = self.ABS(memory);
        //        6
        //    }
        //    0x16 => {
        //        self.instr = Instructions::ASLM;
        //        self.ram = self.ZPX(memory);
        //        6
        //    }
        //    0x06 => {
        //        self.instr = Instructions::ASLM;
        //        self.ram = self.ZP(memory);
        //        5
        //    }
        //    0x0A => {
        //        self.ASLA();
        //        1 // 2
        //    }
        //    // --------------- AND --------------------
        //    0x31 => {
        //        self.instr = Instructions::AND;
        //        let (val, overflow) = self.IDY(memory);
        //        self.ram = val;
        //        5 + overflow as u8
        //    }
        //    0x21 => {
        //        self.instr = Instructions::AND;
        //        self.ram = self.IDX(memory);
        //        6
        //    }
        //    0x39 => {
        //        self.instr = Instructions::AND;
        //        let (val, overflow) = self.ABY(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x3D => {
        //        self.instr = Instructions::AND;
        //        let (val, overflow) = self.ABX(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x2D => {
        //        self.instr = Instructions::AND;
        //        let val = self.ABS(memory);
        //        self.ram = val;
        //        4
        //    }
        //    0x35 => {
        //        self.instr = Instructions::AND;
        //        let val = self.ZPX(memory);
        //        self.ram = val;
        //        4
        //    }
        //    0x25 => {
        //        self.instr = Instructions::AND;
        //        let val = self.ZP(memory);
        //        self.ram = val;
        //        3
        //    }
        //    0x29 => {
        //        let val = self.IMM(memory);
        //        self.AND(val);
        //        1 // 2
        //    }
        //    // --------------- ADC --------------------
        //    0x71 => {
        //        self.instr = Instructions::ADC;
        //        let (val, overflow) = self.IDY(memory);
        //        self.ram = val;
        //        5 + overflow as u8
        //    }
        //    0x61 => {
        //        self.instr = Instructions::ADC;
        //        self.ram = self.IDX(memory);
        //        6
        //    }
        //    0x79 => {
        //        self.instr = Instructions::ADC;
        //        let (val, overflow) = self.ABY(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x7D => {
        //        self.instr = Instructions::ADC;
        //        let (val, overflow) = self.ABX(memory);
        //        self.ram = val;
        //        4 + overflow as u8
        //    }
        //    0x6D => {
        //        self.instr = Instructions::ADC;
        //        self.ram = self.ABS(memory);
        //        4
        //    }
        //    0x75 => {
        //        self.instr = Instructions::ADC;
        //        self.ram = self.ZPX(memory);
        //        4
        //    }
        //    0x65 => {
        //        self.instr = Instructions::ADC;
        //        self.ram = self.ZP(memory);
        //        3
        //    }
        //    0x69 => {
        //        let val = self.IMM(memory);
        //        self.ADC(val);
        //        1 // 2
        //    }
        //    default => panic!("unknown opcode"),
        //}
    }
}
