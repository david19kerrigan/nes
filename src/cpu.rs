enum Instructions {
    NONE,
    ADC,
    ASLA,
    ASLM,
    AND,
    NOP,
    BXX,
    BIT,
    CMP,
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
            instr: Instructions::NONE,
            ram: 0,
            st: vec![],
        }
    }

    // --------------- FLAGS --------------------

    pub fn flag_carry(&mut self, overflow: bool) {
        if overflow {
            self.c = true;
        }
    }

    pub fn flag_negative(&mut self, result: u8) {
        if (result as i8) < 0 {
            self.n = true;
        }
    }

    pub fn flag_zero(&mut self, result: u8) {
        if result == 0 {
            self.z = true;
        }
    }

    pub fn flag_overflow(&mut self, result: u8, b: u8) {
        if (self.a > 0 && b > 0 && (result as i8) < 0)
            || (self.a < 0 && b < 0 && (result as i8) > 0)
        {
            self.o = true;
        }
    }

    // --------------- OPERATIONS --------------------

    pub fn CMPM(&mut self, memory: &mut Vec<u8>) {
        self.CMP(memory[self.ram]);
    }

    pub fn CMP(&mut self, val: u8) {
        let val = self.a - val;
        self.flag_negative(val);
        self.flag_carry(val >= 0);
        self.flag_zero(val);
    }

    pub fn BIT(&mut self, memory: &mut Vec<u8>) {
        let val = memory[self.ram] & self.a;
        self.flag_zero(val);
        self.flag_negative(val);
        self.o = val & 0x40 == 1;
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

    pub fn ANDM(&mut self, memory: &mut Vec<u8>) {
        self.AND(memory[self.ram]);
    }

    pub fn AND(&mut self, val: u8) {
        self.a |= val;
        self.flag_zero(self.a);
        self.flag_negative(self.a);
    }

    pub fn ADCM(&mut self, memory: &mut Vec<u8>) {
        self.ADC(memory[self.ram]);
    }

    pub fn ADC(&mut self, val: u8) {
        let (temp, overflow1) = val.overflowing_add(self.c as u8);
        let (result, overflow2) = self.a.overflowing_add(temp);
        self.a = result;
        self.flag_zero(result);
        self.flag_carry(overflow1 || overflow2);
        self.flag_negative(result);
        self.flag_overflow(result, val);
    }

    // --------------- ADDRESSING --------------------

    pub fn IMM(&mut self, memory: &mut Vec<u8>) -> u8 {
        self.pc += 2;
        memory[self.pc - 1]
    }

    pub fn ZP(&mut self, memory: &mut Vec<u8>) -> usize {
        self.pc += 2;
        memory[memory[self.pc - 1] as usize] as usize
    }

    pub fn ZPX(&mut self, memory: &mut Vec<u8>) -> usize {
        self.pc += 2;
        memory[self.pc - 1].wrapping_add(self.x as u8) as usize
    }

    pub fn ABS(&mut self, memory: &mut Vec<u8>) -> usize {
        self.pc += 3;
        (memory[self.pc - 2] as u16 | (memory[self.pc - 1] as u16) << 8) as usize
    }

    pub fn ABX(&mut self, memory: &mut Vec<u8>) -> (usize, bool) {
        self.pc += 3;
        let (result, overflow) = (memory[self.pc - 2] as u16 | (memory[self.pc - 1] as u16) << 8)
            .overflowing_add(self.x as u16);
        (result as usize, overflow)
    }

    pub fn ABY(&mut self, memory: &mut Vec<u8>) -> (usize, bool) {
        self.pc += 3;
        let (result, overflow) = (memory[self.pc - 2] as u16 | (memory[self.pc - 1] as u16) << 8)
            .overflowing_add(self.y as u16);
        (result as usize, overflow)
    }

    pub fn IDX(&mut self, memory: &mut Vec<u8>) -> (usize) {
        self.pc += 2;
        memory[self.pc - 1].wrapping_add(self.x) as usize
    }

    pub fn IDY(&mut self, memory: &mut Vec<u8>) -> (usize, bool) {
        self.pc += 2;
        let (val, overflow) = memory[self.pc - 1].overflowing_add(self.x);
        (memory[val as usize] as usize, overflow)
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

    pub fn execute_instruction(&mut self, memory: &mut Vec<u8>) {
        match &self.instr {
            Instructions::ADC => self.ADCM(memory),
            Instructions::BIT => self.BIT(memory),
            Instructions::CMP => self.CMPM(memory),
            Instructions::AND => self.ANDM(memory),
            Instructions::ASLM => self.ASLM(memory),
            Instructions::NONE => (),
            default => panic!("invalid instruction"),
        }
        self.instr = Instructions::NONE;
    }

    pub fn load_instruction(&mut self, memory: &mut Vec<u8>) -> u8 {
        let opcode = memory[self.pc];
        match opcode {
            // --------------- COMPARE --------------------
            0xD1 => {
                self.instr = Instructions::CMP;
                let (val, overflow) = self.IDY(memory);
                self.ram = val;
                5 + overflow as u8
            }
            0xC1 => {
                self.instr = Instructions::CMP;
                self.ram = self.IDX(memory);
                6
            }
            0xD9 => {
                self.instr = Instructions::CMP;
                let (val, overflow) = self.ABY(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0xDD => {
                self.instr = Instructions::CMP;
                let (val, overflow) = self.ABX(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0xCD => {
                self.instr = Instructions::CMP;
                self.ram = self.ABS(memory);
                4
            }
            0xD5 => {
                self.instr = Instructions::CMP;
                self.ram = self.ZPX(memory);
                4
            }
            0xC5 => {
                self.instr = Instructions::CMP;
                self.ram = self.ZP(memory);
                3
            }
            0xC9 => {
                let val = self.IMM(memory);
                1 // 2
            }
            // --------------- CLEAR --------------------
            0xB8 => {
                self.o = false;
                1 // 2
            }
            0x58 => {
                self.i = false;
                1 // 2
            }
            0xD8 => {
                self.d = false;
                1 // 2
            }
            0x18 => {
                self.c = false;
                1 // 2
            }
            // --------------- BRK --------------------
            0x00 => {
                self.st.push(self.pc as u16);
                self.pc = (memory[0xFFFE as usize] | memory[0xFFFF as usize] << 8) as usize;
                self.b = true;
                6 // 7
            }
            // --------------- BIT --------------------
            0x24 => {
                self.instr = Instructions::BIT;
                self.ram = self.ZP(memory);
                3
            }
            // --------------- BRANCH --------------------
            0x70 => {
                let (val, cycles) = self.REL(memory, self.o);
                self.BXX(val);
                cycles - 1
            }
            0x50 => {
                let (val, cycles) = self.REL(memory, !self.o);
                self.BXX(val);
                cycles - 1
            }
            0x10 => {
                let (val, cycles) = self.REL(memory, !self.n);
                self.BXX(val);
                cycles - 1
            }
            0xD0 => {
                let (val, cycles) = self.REL(memory, !self.z);
                self.BXX(val);
                cycles - 1
            }
            0x30 => {
                let (val, cycles) = self.REL(memory, self.n);
                self.BXX(val);
                cycles - 1
            }
            0xF0 => {
                let (val, cycles) = self.REL(memory, self.z);
                self.BXX(val);
                cycles - 1
            }
            0xB0 => {
                let (val, cycles) = self.REL(memory, self.c);
                self.BXX(val);
                cycles - 1
            }
            0x90 => {
                let (val, cycles) = self.REL(memory, !self.c);
                self.BXX(val);
                cycles - 1
            }
            // --------------- ASL --------------------
            0x0E => {
                self.instr = Instructions::ASLM;
                self.ram = self.ABS(memory);
                6
            }
            0x16 => {
                self.instr = Instructions::ASLM;
                self.ram = self.ZPX(memory);
                6
            }
            0x06 => {
                self.instr = Instructions::ASLM;
                self.ram = self.ZP(memory);
                5
            }
            0x0A => {
                self.ASLA();
                1 // 2
            }
            // --------------- AND --------------------
            0x31 => {
                self.instr = Instructions::AND;
                let (val, overflow) = self.IDY(memory);
                self.ram = val;
                5 + overflow as u8
            }
            0x21 => {
                self.instr = Instructions::AND;
                self.ram = self.IDX(memory);
                6
            }
            0x39 => {
                self.instr = Instructions::AND;
                let (val, overflow) = self.ABY(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0x3D => {
                self.instr = Instructions::AND;
                let (val, overflow) = self.ABX(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0x2D => {
                self.instr = Instructions::AND;
                let val = self.ABS(memory);
                self.ram = val;
                4
            }
            0x35 => {
                self.instr = Instructions::AND;
                let val = self.ZPX(memory);
                self.ram = val;
                4
            }
            0x25 => {
                self.instr = Instructions::AND;
                let val = self.ZP(memory);
                self.ram = val;
                3
            }
            0x29 => {
                let val = self.IMM(memory);
                self.AND(val);
                1 // 2
            }
            // --------------- ADC --------------------
            0x71 => {
                self.instr = Instructions::ADC;
                let (val, overflow) = self.IDY(memory);
                self.ram = val;
                5 + overflow as u8
            }
            0x61 => {
                self.instr = Instructions::ADC;
                self.ram = self.IDX(memory);
                6
            }
            0x79 => {
                self.instr = Instructions::ADC;
                let (val, overflow) = self.ABY(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0x7D => {
                self.instr = Instructions::ADC;
                let (val, overflow) = self.ABX(memory);
                self.ram = val;
                4 + overflow as u8
            }
            0x6D => {
                self.instr = Instructions::ADC;
                self.ram = self.ABS(memory);
                4
            }
            0x75 => {
                self.instr = Instructions::ADC;
                self.ram = self.ZPX(memory);
                4
            }
            0x65 => {
                self.instr = Instructions::ADC;
                self.ram = self.ZP(memory);
                3
            }
            0x69 => {
                let val = self.IMM(memory);
                self.ADC(val);
                1 // 2
            }
            default => panic!("unknown opcode"),
        }
    }
}
