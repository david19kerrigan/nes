mod bus;
mod cpu;
mod ppu;
mod testing;
mod util;

use crate::util::*;
use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;
use testing::Testing;

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
    let mut cycles_frame: u128 = 0;

    bus.load_cartridge("/home/david/Downloads/dk.nes");
    cpu.Reset(&mut bus);
    let mut testing = Testing::new("/home/david/Documents/nes/src/testing/second_break/reset6.log");

    // --------------- SDL ------------------

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("game", 256 * 2, 240 * 2)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
	canvas.set_scale(2.0, 2.0);
    canvas.clear();
    canvas.present();

    // --------------- Inputs ------------------
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut input: u8 = 0;
    let mut start = Instant::now();

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
                    Keycode::K => input |= 1 << 0,       // A
                    Keycode::J => input |= 1 << 1,       // B
                    Keycode::KpEnter => input |= 1 << 2, // Select
                    Keycode::V => input |= 1 << 3,       // Start
                    Keycode::W => input |= 1 << 4,       // Up
                    Keycode::S => input |= 1 << 5,       // Down
                    Keycode::A => input |= 1 << 6,       // Left
                    Keycode::D => input |= 1 << 7,            // Right
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::K => input &= 0 << 0,       // A
                    Keycode::J => input &= 0 << 1,       // B
                    Keycode::KpEnter => input &= 0 << 2, // Select
                    Keycode::V => input &= 0 << 3,       // Start
                    Keycode::W => input &= 0 << 4,       // Up
                    Keycode::S => input &= 0 << 5,       // Down
                    Keycode::A => input &= 0 << 6,       // Left
                    Keycode::D => input &= 0 << 7,            // Right
                    _ => (),
                },
                _ => (),
            }
        }

		bus.input = input;

        // --------------- Instructions ------------------

        while cycles_frame < 29780 {
            if cycles_left == 1 { // on the final cycle -> execute the previous instruction
                cpu.execute_instruction(&mut bus, &mut ppu);
                testing.check_vblank(&mut bus, &mut cpu);
            } else if cycles_left == 0 { // get a new instruction and wait for cycles_left
                let (temp, p, sp, a, x, y, addr) = cpu.load_instruction(&mut bus);
                cycles_left = temp;
                testing.test_log(&mut cpu, &mut ppu);
				testing.cyc += cycles_left as u128;
                cycles_frame += cycles_left as u128;
            }
            for m in 0..3 {
                let temp = ppu.tick(&mut bus, &mut canvas, &mut cpu);
				cycles_left += temp;
				testing.cyc += temp as u128;
            }
            cycles_left -= 1;
        }

        cycles_frame = 0;
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
