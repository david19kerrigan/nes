use crate::util::*;
use crate::Bus;
use crate::Cpu;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Status {
    vblank: bool,
    hit: bool,
    overflow: bool,
    bus: u8,
}

pub struct Control {
    nametable_address: u16,
    vram_increment: u8,
    sprite_address: u8,
    background_address: u8,
    sprite_size: u8,
    master_slave: bool,
    nmi: bool,
}

pub struct Mask {
    greyscale: bool,
    background_left_8: bool,
    sprite_left_8: bool,
    background: bool,
    sprite: bool,
    red: bool,
    green: bool,
    blue: bool,
}

pub fn parse_nametable(nametable_bits: u8) -> u16 {
    match nametable_bits {
        0 => 0x2000,
        1 => 0x2400,
        2 => 0x2800,
        3 => 0x2C00,
        _ => panic!(),
    }
}

impl Mask {
    pub fn new() -> Mask {
        Mask {
            greyscale: false,
            background_left_8: false,
            sprite_left_8: false,
            background: false,
            sprite: false,
            red: false,
            green: false,
            blue: false,
        }
    }

    pub fn read(&mut self, bus: &mut Bus) {
        let byte = bus.cpu_read_16(CONTROL);
        self.greyscale = get_u8_bit(byte, 0) == 1;
        self.background_left_8 = get_u8_bit(byte, 1) == 1;
        self.sprite_left_8 = get_u8_bit(byte, 2) == 1;
        self.background = get_u8_bit(byte, 3) == 1;
        self.sprite = get_u8_bit(byte, 4) == 1;
        self.red = get_u8_bit(byte, 5) == 1;
        self.green = get_u8_bit(byte, 6) == 1;
        self.blue = get_u8_bit(byte, 7) == 1;
    }
}

impl Control {
    pub fn new() -> Control {
        Control {
            nametable_address: parse_nametable(0),
            vram_increment: 1,
            sprite_address: 0,
            background_address: 0,
            sprite_size: 0,
            master_slave: false,
            nmi: false,
        }
    }

    pub fn read(&mut self, bus: &mut Bus) {
        let byte = bus.cpu_read_16(CONTROL);
        self.nmi = get_u8_bit(byte, 7) == 1;
        self.master_slave = get_u8_bit(byte, 6) == 1;
        self.sprite_size = get_u8_bit(byte, 5);
        self.background_address = get_u8_bit(byte, 4);
        self.sprite_address = get_u8_bit(byte, 3);
        let vram_increment = get_u8_bit(byte, 2);
        self.vram_increment = if vram_increment == 0 { 1 } else { 32 };
        self.nametable_address = parse_nametable(get_u8_bits(byte, 1, 0));
    }
}

impl Status {
    pub fn new() -> Status {
        Status {
            vblank: false,
            hit: false,
            overflow: false,
            bus: 0,
        }
    }

    pub fn write(&mut self, bus: &mut Bus) {
        let byte = (self.vblank as u8) << 7
            | (self.hit as u8) << 6
            | (self.overflow as u8) << 5
            | self.bus;
        bus.cpu_write_16(STATUS, byte);
    }
}

pub struct Ppu {
    oam: [u8; 256],
    pub cycle: u16,
    pub line: u16,
    nametable_addr: u16,
    pub status: Status,
    pub control: Control,
    pub mask: Mask,
    pub addr: u16,
    pub oam_addr: u16,
    v: u8,
    t: u8,
    x: u8,
    w: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 256],
            cycle: 0,
            line: 0,
            nametable_addr: 0,
            status: Status::new(),
            control: Control::new(),
            mask: Mask::new(),
            addr: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
        }
    }

    pub fn write_data(&mut self, data: u8, bus: &mut Bus) {
        bus.ppu_write_16(self.addr % 0x4000, data);
        println!("ppu write {:0x} {:0x}", self.addr, data);
        self.addr += self.control.vram_increment as u16;
    }

    pub fn write_addr(&mut self, addr: u8) {
        if !self.w {
            self.addr |= (addr as u16) << 8;
        } else {
            self.addr |= addr as u16;
        }
        self.w = !self.w
    }

    pub fn write_oam(&mut self, addr: u8, bus: &mut Bus) {
        let mut new_addr = (addr as u16) << 8;
        for n in 0..0xFF {
            self.oam[n] = bus.cpu_read_16(new_addr);
            new_addr += 1;
        }
    }

    pub fn tick(&mut self, bus: &mut Bus, canvas: &mut Canvas<Window>, cpu: &mut Cpu) -> u8 {
        let mut cycles = 0;
        //println!("line cycle {} {}", self.line, self.cycle);
        if self.line < 240 {
            if self.cycle >= 1 && self.cycle <= 256 && (self.cycle - 1) % 8 == 0 {
                let nametable_x = (256 - self.cycle) / 8;
                let nametable_y = (240 - self.line) / 8;
                let nametable_offset = nametable_y * 8 + nametable_x;
                let nametable_byte = self.control.nametable_address + nametable_offset;

                let tile_row = (240 - self.line) % 8;
                let mut pattern_address_0 = tile_row;
                pattern_address_0 |= (nametable_byte as u16) << 4;
                pattern_address_0 |= (self.control.background_address as u16) << 14;
                let pattern_address_1 = pattern_address_0 | 1 << 3;

                let pattern_byte_0 = bus.ppu_read_16(pattern_address_0);
                let pattern_byte_1 = bus.ppu_read_16(pattern_address_1);

                for n in (0..8).rev() {
                    let bit_0 = pattern_byte_0 >> n & 0x01;
                    let bit_1 = pattern_byte_1 >> n & 0x01;
                    let sum = bit_1 << 1 | bit_0;
                    // Render pixels in monochrome for now
                    if sum > 0 {
                        let point =
                            Point::new(256 - (self.cycle + n) as i32, 240 - self.line as i32);
                        if self.mask.background {
                            canvas.draw_point(point);
                        }
                        canvas.draw_point(point);
                    }
                }
            }
        } else if self.line == 261 {
        }

        if self.line == 240 && self.cycle == 1 {
            self.status.vblank = true;
            self.status.write(bus);
            if self.control.nmi {
                println!("NMI Occurred");
                cpu.NMI(bus);
                cycles = 7;
            }
        } else if self.line == 261 && self.cycle == 1 {
            self.status.vblank = false;
            self.status.write(bus);
        }

        self.cycle += 1;

        if self.cycle > 340 {
            self.cycle = 0;
            self.line += 1;
        }
        if self.line > 261 {
            self.line = 0;
        }
        cycles
    }
}
