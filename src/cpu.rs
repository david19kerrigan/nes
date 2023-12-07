use crate::util::*;
use crate::Bus;

const ERR_INSTR: String = String::from("Invalid Instruction");
const ERR_OP: String = String::from("Invalid Instruction");
const ERR_ADDR: String = String::from("Invalid Addressing Mode");

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
    stack: Vec<u16>,
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

    //pub fn EOR(&mut self, val: u8) {
    //    self.a ^= val;
    //    self.flag_zero(self.a);
    //    self.flag_negative(self.a);
    //}

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

    //pub fn BIT(&mut self, val: u8) {
    //    let res = val & self.a;
    //    self.flag_zero(res);
    //    self.flag_negative(res);
    //    self.o = res & 0x40 == 1;
    //}

    //pub fn BXX(&mut self, val: isize) {
    //    self.pc = (self.pc).wrapping_add_signed(val) as usize;
    //}

    //pub fn ASLA(&mut self) {
    //    self.flag_carry(self.a & 0x80 == 1);
    //    self.a <<= 1;
    //    self.flag_zero(self.a);
    //    self.flag_negative(self.a);
    //    self.pc += 1;
    //}

    pub fn ADC_SBC(&mut self, bus: &mut Bus, target_addr: usize) {
        let val = bus.read_usize(target_addr);
        let (val_with_carry, overflow1) = val.overflowing_add(self.c as u8);
        let (result, overflow2) = self.a.overflowing_add(val_with_carry);
        self.a = result;
        self.flag_zero(self.a);
        self.flag_negative(self.a);
        self.flag_carry(overflow1 || overflow2);
        self.flag_overflow(self.a, val_with_carry);
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

    #[rustfmt::skip]
    pub fn execute_instruction(&mut self, bus: &mut Bus) {
        let target_addr = match self.addr {
            Addressing::ACC => 0,
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
                let zp = bus.read_single(self.pc);
                combine_low_high(zp, zp + 1).wrapping_add(self.y as u16) as usize
            }
            _ => panic!("{}", ERR_ADDR),
        };

        let mut target_val = bus.read_usize(target_addr);
        match self.instr {
            Instructions::ADC | Instructions::SBC => {
                let op: &dyn Fn(u8, u8) -> (u8, bool) = match self.instr { Instructions::ADC => &u8::overflowing_add, Instructions::SBC => &u8::overflowing_sub, _ => panic!("{}", ERR_INSTR) };
                let (val_with_carry, overflow1) = op(target_val, self.c as u8);
                let (result, overflow2) =  op(self.a, val_with_carry);
                self.a = result;
                self.flag_zero(self.a);
                self.flag_negative(self.a);
                self.flag_carry(overflow1 || overflow2);
                self.flag_overflow(self.a, val_with_carry);
            }
            Instructions::AND | Instructions::EOR | Instructions::ORA => {
                let op: &dyn Fn(u8, u8) -> (u8) = match self.instr { Instructions::AND => &u8_and, Instructions::ORA => &u8_or, _ => panic!("{}", ERR_INSTR) };
                self.a = op(self.a, target_val);
                self.flag_zero(self.a);
                self.flag_negative(self.a);
            }
            Instructions::ASL | Instructions::LSR | Instructions::ROL | Instructions::ROR => {
                let val: u8;
                let op: &dyn Fn(u8) -> (u8) = match self.instr { Instructions::ASL => &u8_shl, Instructions::LSR => &u8_shr, Instructions::ROR  => &u8_shr, Instructions::ROL => &u8_shl, _ => panic!("{}", ERR_INSTR) };
                if self.addr == Addressing::ACC {
                    self.flag_carry(self.a & 0x80 == 1);
                    self.a = op(self.a);
                    match self.instr { Instructions::ROL | Instructions::ROR => self.a |= self.c as u8, _ => () };
                    val = self.a;
                } else {
                    self.flag_carry(target_val & 0x80 == 1);
                    match self.instr { Instructions::ROL | Instructions::ROR => target_val |= self.c as u8, _ => () };
                    bus.write_usize(target_addr, op(target_val));
                    val = bus.read_usize(target_addr);
                }
                self.flag_zero(val);
                self.flag_negative(val);
            }
            _ => panic!("{}", ERR_OP),
        }
    }

    #[rustfmt::skip]
    pub fn load_instruction(&mut self, bus: &mut Bus) -> u8 {
        let cycles: u8;
        match bus.read_usize(self.pc) {
            0x69 => {self.instr = Instructions::ADC; self.addr = Addressing::IMM; cycles = 2; self.pc += 2},
            0x65 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPG; cycles = 3; self.pc += 2},
            0x75 => {self.instr = Instructions::ADC; self.addr = Addressing::ZPX; cycles = 4; self.pc += 2},
            0x6D => {self.instr = Instructions::ADC; self.addr = Addressing::ABS; cycles = 4; self.pc += 2},
            0x7D => {self.instr = Instructions::ADC; self.addr = Addressing::ABX; cycles = 4 + bus.cross_abs(self.pc + 1, self.pc + 2, self.x); self.pc += 3},
            0x79 => {self.instr = Instructions::ADC; self.addr = Addressing::ABY; cycles = 4 + bus.cross_abs(self.pc + 1, self.pc + 2, self.y); self.pc += 3},
            0x61 => {self.instr = Instructions::ADC; self.addr = Addressing::IDX; cycles = 6; self.pc += 2},
            0x71 => {self.instr = Instructions::ADC; self.addr = Addressing::IDY; cycles = 5 + bus.cross_idy(self.pc + 1, self.y); self.pc += 2},
            _ => panic!("{}", ERR_OP),
        }
        cycles
    }
}
