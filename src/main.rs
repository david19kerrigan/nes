mod cpu;

use cpu::Cpu;

fn main() {
    let mut memory = &mut vec![1 as u8];
    let mut cpu = Cpu::new();
    let mut delay = 1;

    loop {
        if delay == 1 { // Reading the instruction takes one cycle
            cpu.execute_instruction(&mut memory);
            delay = cpu.execute_cycle(&mut memory);
        } else {
            delay -= 1;
        }
    }
}
