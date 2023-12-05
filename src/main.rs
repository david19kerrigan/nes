mod cpu;

use cpu::Cpu;

fn main() {
    let mut memory = &mut vec![1 as u8];
    let mut cpu = Cpu::new();
    let mut delay = 0;

    loop {
        if delay == 1 {
            cpu.execute_instruction(&mut memory);
        } else if delay == 0 {
            delay = cpu.load_instruction(&mut memory);
        }
        delay -= 1;
    }
}
