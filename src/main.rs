mod cpu;
mod bus;
mod util;

use cpu::Cpu;
use bus::Bus;

use std::time::Duration;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
    let mut cycles_left = 0;
    let mut cycles_total = 0;

    println!("cycle: {}", cycles_total);
    bus.load_cartridge("/home/david/Documents/nes/src/test/nestest.nes");
    cpu.pc = 0xC000;
    println!("cycle: {}", cycles_total);

    loop {
        if cycles_left == 1 {
            cpu.execute_instruction(&mut bus);
        } else if cycles_left == 0 {
            cycles_left = cpu.load_instruction(&mut bus);
            cycles_total += cycles_left;
            println!("cycle: {}", cycles_total);
        }
        cycles_left -= 1;

        std::thread::sleep(Duration::from_millis(1));
    }
}
