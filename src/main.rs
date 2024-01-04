mod bus;
mod cpu;
mod util;

use crate::util::*;
use bus::Bus;
use cpu::Cpu;

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

struct Keys {
    a: u8,
    b: u8,
    start: u8,
    select: u8,
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

impl Keys {
    fn new() -> Keys {
        Keys {
            a: 0,
            b: 0,
            start: 0,
            select: 0,
            up: 0,
            down: 0,
            left: 0,
            right: 0,
        }
    }
}

fn main() {
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
    let mut cycles_left = 0;
    let mut cycles_total: u64 = 7;

    let file = File::open("/home/david/Documents/nes/src/test/nestest2.log").unwrap();
    let mut rdr = Reader::from_reader(file);
    let mut rec = rdr.records();

    bus.load_cartridge("/home/david/Documents/nes/src/test/nestest.nes");
    cpu.pc = 0xFFFC;
    println!("cycle: {}", cycles_total);

    // --------------- SDL ------------------

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("game", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // --------------- Inputs ------------------
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut start = Instant::now();

    loop {
        let mut input: u8 = 0;
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
                    Keycode::K => input |= 1 << 7, // A
                    Keycode::J => input |= 1 << 6, // B
                    Keycode::KpEnter => input |= 1 << 5, // Select
                    Keycode::V => input |= 1 << 4, // Start
                    Keycode::W => input |= 1 << 3, // Up
                    Keycode::S => input |= 1 << 2, // Down
                    Keycode::A => input |= 1 << 1, // Left
                    Keycode::D => input |= 1, // Right
                    _ => (),
                },
				_ => (),
            }
        }

        bus.write_16(0x4016, input, Component::CPU);
        println!("input {}", input);

        // --------------- Instructions ------------------

        for n in 0..29780 {
            if cycles_left == 1 {
                cpu.execute_instruction(&mut bus);
            } else if cycles_left == 0 {
                //let line = rec.next().unwrap().unwrap();
                cycles_left = cpu.load_instruction(&mut bus, cycles_total);
                cycles_total += cycles_left as u64;
                //println!("------------------------");
            }
            cycles_left -= 1;
        }

        // --------------- Timing ------------------

        let duration = start.elapsed();
        let greatest_sleep = Duration::from_millis(17);
        if duration < greatest_sleep {
			println!("LESS");
            let sleep_time = greatest_sleep - duration;
            std::thread::sleep(sleep_time);
        } else{
			println!("MORE");
		}
        start = Instant::now();
    }
}
