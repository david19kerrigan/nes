use crate::util::*;
use crate::Bus;
use crate::Cpu;
use crate::Ppu;
use csv::{Reader, StringRecordsIntoIter};
use std::fs::File;

const LINE_P: usize = 6;
const LINE_SP: usize = 5;
const LINE_A: usize = 2;
const LINE_X: usize = 3;
const LINE_Y: usize = 4;
const LINE_PC: usize = 7;
const LINE_CYC: usize = 1;

const BEFORE_PASSED_DOUBLE_VBLANK: u128 = 89457;
const PASSED_DOUBLE_VBLANK: u128 = 89460;

// nogfx log
//const LINE_P: u8 = 9;
//const LINE_SP: u8 = 10;
//const LINE_A: u8 = 6;
//const LINE_X: u8 = 7;
//const LINE_Y: u8 = 8;
//const LINE_ADDR: u8 = 0;
//const LINE_CYC: usize = 13;

fn parse_processor_flags(flags: &str) -> u8 {
    let n = flags.chars().nth(0).unwrap().is_uppercase() as u8;
    let v = flags.chars().nth(1).unwrap().is_uppercase() as u8;
    let u = flags.chars().nth(2).unwrap().is_uppercase() as u8;
    let b = flags.chars().nth(3).unwrap().is_uppercase() as u8;
    let d = flags.chars().nth(4).unwrap().is_uppercase() as u8;
    let i = flags.chars().nth(5).unwrap().is_uppercase() as u8;
    let z = flags.chars().nth(6).unwrap().is_uppercase() as u8;
    let c = flags.chars().nth(7).unwrap().is_uppercase() as u8;
    n << 7 | v << 6 | 1 << 5 | b << 4 | d << 3 | i << 2 | z << 1 | c
}

fn check_attribute(is_equal: bool, name: &str) {
    if !is_equal {
        panic!("mismatched {}", name);
    }
}

pub struct Testing {
    my_vals: Vec<u128>,
    true_vals: Vec<u128>,
    records: StringRecordsIntoIter<File>,
    vblank_count: u8,
    has_passed_double_vblank: bool,
    pub cyc: u128,
}

impl Testing {
    pub fn new(path: &str) -> Testing {
        Testing {
            my_vals: vec![],
            true_vals: vec![],
            records: Reader::from_path(path).unwrap().into_records(),
            vblank_count: 0,
            has_passed_double_vblank: false,
            cyc: 0,
        }
    }
    pub fn check_vblank(&mut self, bus: &mut Bus, cpu: &mut Cpu) {
        let ppu_status = bus.cpu_memory[STATUS as usize];
        if !self.has_passed_double_vblank {
            if get_u8_bit(ppu_status, 7) == 1 {
                self.vblank_count += 1;
            }
            if self.vblank_count == 1 {
                self.has_passed_double_vblank = true;
                let mut line = self.records.next().unwrap().unwrap();
                while u128::from_str_radix(&line[LINE_CYC], 10).unwrap()
                    < BEFORE_PASSED_DOUBLE_VBLANK
                {
                    line = self.records.next().unwrap().unwrap();
                }
                self.cyc = PASSED_DOUBLE_VBLANK;
            }
        }
    }
    pub fn test_log(&mut self, cpu: &mut Cpu, ppu: &mut Ppu) {
        println!("x {:0x}", cpu.x);
        println!("y {:0x}", cpu.y);
        println!("a {:0x}", cpu.a);
        println!("p {:0b}", cpu.flags_to_byte());
        println!("pc {:0x}", cpu.pc);
        println!("cyc {}", self.cyc);
        println!("instr {:?}", cpu.instr);
        println!("addr {:?}", cpu.addr);
        println!("ppu line cycle {} {}", ppu.line, ppu.cycle);
        println!("--------");

        if self.has_passed_double_vblank {
            //let line = self.records.next().unwrap().unwrap();

            //let true_p = parse_processor_flags(&line[LINE_P]);
            //let true_cyc = u128::from_str_radix(&line[LINE_CYC], 10).unwrap();
            //let true_a = u8::from_str_radix(&line[LINE_A], 16).unwrap();
            //let true_x = u8::from_str_radix(&line[LINE_X], 16).unwrap();
            //let true_y = u8::from_str_radix(&line[LINE_Y], 16).unwrap();
            //let true_pc = u16::from_str_radix(&line[LINE_PC], 16).unwrap();

            //println!("true x {:0x}", true_x);
            //println!("true y {:0x}", true_y);
            //println!("true a {:0x}", true_a);
            //println!("true p {:0b}", true_p);
            //println!("true pc {:0x}", true_pc);
            //println!("true cyc {}", true_cyc);
            //println!("------------------------");

            //check_attribute(cpu.x == true_x, "x");
            //check_attribute(cpu.y == true_y, "y");
            //check_attribute(cpu.a == true_a, "a");
            //check_attribute(cpu.pc == true_pc, "pc");
            //check_attribute(self.cyc == true_cyc, "cyc");
            //check_attribute(cpu.flags_to_byte() == true_p, "p");
        }
    }
}
