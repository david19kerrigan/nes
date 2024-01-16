mod bus;
mod cpu;
mod ppu;
mod util;

use crate::util::*;
use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;

use csv::Reader;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::File;
use std::thread::sleep;
use std::time::{Duration, Instant};

const LINE_P: usize = 6;
const LINE_SP: usize = 5;
const LINE_A: usize = 2;
const LINE_X: usize = 3;
const LINE_Y: usize = 4;
const LINE_ADDR: usize = 7;
const LINE_CYC: usize = 1;

//const LINE_P: u8 = 9;
//const LINE_SP: u8 = 10;
//const LINE_A: u8 = 6;
//const LINE_X: u8 = 7;
//const LINE_Y: u8 = 8;
//const LINE_ADDR: u8 = 0;
//const LINE_CYC: usize = 13;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
    let mut ppu = Ppu::new();
	ppu.status.write(&mut bus);
    let mut cycles_left = 0;
    let mut cycles_total: u128 = 0;

    bus.load_cartridge("/home/david/Documents/nes/src/test/nestest.nes");
    cpu.Reset(&mut bus);
    cycles_total = 7; // JMP takes 7 cycles

    // --------------- Testing ------------------

    //cpu.pc = 0xC000;
    let file = File::open("/home/david/Documents/nes/src/test/reset6.log").unwrap();
    let mut rdr = Reader::from_reader(file);
    let mut rec = rdr.records();

    // --------------- SDL ------------------

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("game", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    // --------------- Inputs ------------------
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut start = Instant::now();
    let mut input: u8 = 0;

    loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::K => input |= 1 << 7,       // A
                    Keycode::J => input |= 1 << 6,       // B
                    Keycode::KpEnter => input |= 1 << 5, // Select
                    Keycode::V => input |= 1 << 4,       // Start
                    Keycode::W => input |= 1 << 3,       // Up
                    Keycode::S => input |= 1 << 2,       // Down
                    Keycode::A => input |= 1 << 1,       // Left
                    Keycode::D => input |= 1,            // Right
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::K => input &= 0 << 7,       // A
                    Keycode::J => input &= 0 << 6,       // B
                    Keycode::KpEnter => input &= 0 << 5, // Select
                    Keycode::V => input &= 0 << 4,       // Start
                    Keycode::W => input &= 0 << 3,       // Up
                    Keycode::S => input &= 0 << 2,       // Down
                    Keycode::A => input &= 0 << 1,       // Left
                    Keycode::D => input &= 0,            // Right
                    _ => (),
                },
                _ => (),
            }
        }

        bus.cpu_write_16(0x4016, input);

        // --------------- Instructions ------------------

        for n in 0..29780 {
            if cycles_left == 1 {
                cpu.execute_instruction(&mut bus, &mut ppu);
            } else if cycles_left == 0 {
                let (temp, p, sp, a, x, y, addr) = cpu.load_instruction(&mut bus);
                cycles_left = temp;

                // --------------- Testing ------------------
				println!("ppu status {:0b}", bus.cpu_memory[STATUS as usize]);

                let line = rec.next().unwrap().unwrap();
                let true_p = parse_processor_flags(&line[LINE_P]);
                check_attribute_128(&line[LINE_CYC], cycles_total, "cyc");
                check_attribute_8(&line[LINE_A], a, "a");
                check_attribute_8_str(true_p, p, "p");
                check_attribute_8(&line[LINE_SP], sp, "sp");
                check_attribute_8(&line[LINE_X], x, "x");
                check_attribute_8(&line[LINE_Y], y, "y");
                check_attribute_16(&line[LINE_ADDR], addr, "addr");


                // ------------------------------------------

                println!("cycles {}", cycles_total);
                println!("------------------------");
                cycles_total += cycles_left as u128;
            }

            for m in 0..cycles_left * 3 {
                ppu.tick(&mut bus, &mut canvas, &mut cpu);
            }

            cycles_left -= 1;

        }

        canvas.present();

        // --------------- Timing ------------------

        let duration = start.elapsed();
        let greatest_sleep = Duration::from_millis(17);
        if duration < greatest_sleep {
            let sleep_time = greatest_sleep - duration;
            std::thread::sleep(sleep_time);
        }
        start = Instant::now();
    }
}

fn parse_processor_flags(flags: &str) -> u8 {
    println!("flags {}", flags);
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

fn check_attribute_16(true_val: &str, my_val: u16, name: &str) {
    let parsed_val = u16::from_str_radix(true_val, 16).unwrap();
    println!(
        "true {}, my {} = {:04x}, {:04x}",
        name, name, parsed_val, my_val
    );
    if parsed_val != my_val {
        panic!("mismatched {}", name);
    }
}

fn check_attribute_8_str(true_val: u8, my_val: u8, name: &str) {
    let parsed_val = true_val;
    println!(
        "true {}, my {} = {:0b}, {:0b}",
        name, name, parsed_val, my_val
    );
    if parsed_val != my_val {
        panic!("mismatched {}", name);
    }
}

fn check_attribute_8(true_val: &str, my_val: u8, name: &str) {
    let parsed_val = u8::from_str_radix(true_val, 16).unwrap();
    println!(
        "true {}, my {} = {:02x}, {:02x}",
        name, name, parsed_val, my_val
    );
    if parsed_val != my_val {
        panic!("mismatched {}", name);
    }
}

fn check_attribute_128(true_val: &str, my_val: u128, name: &str) {
    let parsed_val = u128::from_str_radix(true_val, 10).unwrap();
    println!("true {}, my {} = {}, {}", name, name, parsed_val, my_val);
    if parsed_val != my_val {
        panic!("mismatched {}", name);
    }
}
