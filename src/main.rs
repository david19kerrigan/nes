mod bus;
mod cpu;
mod util;

use bus::Bus;
use cpu::Cpu;

use std::time::Duration;
use std::fs::File;
use csv::Reader;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
    let mut cycles_left = 0;
    let mut cycles_total: u64 = 7;

	let file = File::open("/home/david/Documents/nes/src/test/nestest2.log").unwrap();
	let mut rdr = Reader::from_reader(file);
	let mut rec = rdr.records();

    bus.load_cartridge("/home/david/Documents/nes/src/test/nestest.nes");
    cpu.pc = 0xC000;
    println!("cycle: {}", cycles_total);

    loop {
        if cycles_left == 1 {
            cpu.execute_instruction(&mut bus);
        } else if cycles_left == 0 {
			let line = rec.next().unwrap().unwrap();
            cycles_left = cpu.load_instruction(&mut bus, &line);
            cycles_total += cycles_left as u64;
            println!("------------------------");
            println!("cycle: {}", cycles_total);
        }
        cycles_left -= 1;

        std::thread::sleep(Duration::from_millis(1));
    }
}
