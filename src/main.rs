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

fn main() {
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
	let mut ppu = Ppu::new();
    let mut cycles_left = 0;
    let mut cycles_total: u128 = 7;

    bus.load_cartridge("/home/david/Documents/nes/src/test/nestest.nes");
    cpu.Reset(&mut bus);

    // --------------- Testing ------------------

    cpu.pc = 0xC000;
    let file = File::open("/home/david/Documents/nes/src/test/nestest2.log").unwrap();
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

        bus.write_16(0x4016, input, Component::CPU);
        //println!("input {}", input);

        // --------------- Instructions ------------------

        for n in 0..29780 {
            if cycles_left == 1 {
                cpu.execute_instruction(&mut bus);
            } else if cycles_left == 0 {
                let (temp, p, sp, a, x, y, addr) = cpu.load_instruction(&mut bus);
				cycles_left = temp;

                let line = rec.next().unwrap().unwrap();
				check_attribute_128(&line[13], cycles_total, "cyc");
				check_attribute_8(&line[9], p, "p");
				check_attribute_8(&line[10], sp, "sp");
				check_attribute_8(&line[6], a, "a");
				check_attribute_8(&line[7], x, "x");
				check_attribute_8(&line[8], y, "y");
				check_attribute_16(&line[0], addr, "addr");

                cycles_total += cycles_left as u128;
                println!("------------------------");
            }
            cycles_left -= 1;

			for m in 0..2 {
				ppu.tick(&mut bus);
			}
        }

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

fn check_attribute_16(true_val: &str, my_val: u16, name: &str) {
	let parsed_val = u16::from_str_radix(true_val, 16).unwrap();
	println!("true {}, my {} = {:04x}, {:04x}", name, name, parsed_val, my_val);
	if parsed_val != my_val {
		panic!("mismatched {}", name);
	}
}

fn check_attribute_8(true_val: &str, my_val: u8, name: &str) {
	let parsed_val = u8::from_str_radix(true_val, 16).unwrap();
	println!("true {}, my {} = {:02x}, {:02x}", name, name, parsed_val, my_val);
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
