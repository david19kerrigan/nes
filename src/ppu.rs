use crate::util::*;
use crate::Bus;
use crate::Cpu;

use colors_transform::{Color as ColorT, Hsl, Rgb};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub const PALETTE_ADDRESS: u16 = 0x3F00;

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
    sprites: [u8; 8],
    sprite_index: usize,
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
            sprites: [0; 8],
            sprite_index: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
        }
    }

    pub fn write_data(&mut self, data: u8, bus: &mut Bus) {
        bus.ppu_write_16(self.addr, data);
        self.addr += self.control.vram_increment as u16;
        //println!("ppu write {:0x} {:0x}", self.addr, data);
    }

    pub fn write_addr(&mut self, addr: u8) {
        if !self.w {
            self.addr &= 0x00FF;
            self.addr |= (addr as u16) << 8;
        } else {
            self.addr &= 0xFF00;
            self.addr |= addr as u16;
        }
        self.w = !self.w;
    }

    pub fn write_oam(&mut self, addr: u8, bus: &mut Bus) {
        let mut new_addr = (addr as u16) << 8;
        for n in 0..0xFF {
            self.oam[n] = bus.cpu_read_16(new_addr);
            new_addr += 1;
        }
    }

    pub fn get_pattern_address(
        &mut self,
        bus: &mut Bus,
        tile: u8,
        line: u16,
        table_half: u8,
    ) -> (u8, u8) {
        let pattern_address_0 = line | (tile as u16) << 4 | (table_half as u16) << 12;
        let pattern_address_1 = pattern_address_0 | 1 << 3;

        let pattern_byte_0 = bus.ppu_read_16(pattern_address_0);
        let pattern_byte_1 = bus.ppu_read_16(pattern_address_1);
        (pattern_byte_0, pattern_byte_1)
    }

    pub fn draw_tile(
        &mut self,
        bus: &mut Bus,
        canvas: &mut Canvas<Window>,
        pattern_byte_0: u8,
        pattern_byte_1: u8,
        n: u8,
        palette_color: u16,
        use_offset: bool,
    ) -> bool {
        let bit_0 = get_u8_bit(pattern_byte_0, 7 - n);
        let bit_1 = get_u8_bit(pattern_byte_1, 7 - n);
        let sum = bit_1 << 1 | bit_0;
        if sum > 0 {
            let color = bus.ppu_read_16(palette_color + sum as u16);
            let hue = ((color & 0x0F) as f32 / 0x100 as f32) * 360.0;
            let brightness = ((color >> 4 & 0b00000011) as f32 / 0b100 as f32) * 100.0;
            let rgb_color = Hsl::from(hue, 100.0, brightness).to_rgb();
            canvas.set_draw_color(Color::RGB(
                rgb_color.get_red() as u8,
                rgb_color.get_green() as u8,
                rgb_color.get_blue() as u8,
            ));

            let point = Point::new(
                (self.cycle + if use_offset { n } else { 0 } as u16) as i32,
                self.line as i32,
            );
            if self.mask.background {
                //canvas.draw_point(point);
            }
            canvas.draw_point(point);
            return true;
        }
        false
    }

    pub fn tick(&mut self, bus: &mut Bus, canvas: &mut Canvas<Window>, cpu: &mut Cpu) -> u8 {
        let mut cycles = 0;
        let mut sprite_0_draw = false;
        let mut background_draw = false;
        if self.line < 240 {
            // --------------------- GET SPRITES FOR NEXT LINE -----------------------
            if self.cycle == 257 {
                while self.sprite_index > 0
                    && self.line >= 8
                    && self.oam[(self.sprites[self.sprite_index - 1] * 4) as usize]
                        < (self.line - 8) as u8
                {
                    self.sprite_index -= 1;
                }
                for n in 0..(self.oam.len() / 4) {
                    if self.line + 1 == self.oam[n * 4] as u16 {
                        if self.sprite_index == 8 {
                            self.status.overflow = true;
                            self.status.write(bus);
                        } else {
                            self.sprites[self.sprite_index] = n as u8;
                            self.sprite_index += 1;
                        }
                    }
                }
            }
            if self.cycle >= 1 && self.cycle <= 256 {
                let n = (self.cycle) % 8;
                // --------------------- NAMETABLE -----------------------
                let nametable_address = self.control.nametable_address
                    + (((self.line + 1) / 8) * 32 + (self.cycle / 8));
                let nametable_byte = bus.ppu_read_16(nametable_address);

                let (pattern_byte_0, pattern_byte_1) = self.get_pattern_address(
                    bus,
                    nametable_byte,
                    (self.line + 1) % 8,
                    self.control.background_address,
                );

                //println!("nametable_byte {:0x}", nametable_byte);
                //println!("pattern_address {:0x}", pattern_address_0);

                // --------------------- ATTRIBUTE TABLE -----------------------

                let attribute = bus.ppu_read_16(
                    (self.control.nametable_address + 0x3C0)
                        + ((240 - self.line) / 32) * 8
                        + (256 - self.cycle) / 32,
                );

                let top_left = attribute & 0b00000011;
                let top_right = attribute >> 2 & 0b00000011;
                let bottom_left = attribute >> 4 & 0b00000011;
                let bottom_right = attribute >> 6 & 0b00000011;

                let rel_x = self.cycle % 16;
                let rel_y = self.line % 16;
                let mut palette_offset = 0;

                if rel_x <= 7 {
                    if rel_y <= 7 {
                        palette_offset = top_left * 4;
                    } else if rel_y > 7 {
                        palette_offset = bottom_left * 4;
                    }
                } else if rel_x > 7 {
                    if rel_y <= 7 {
                        palette_offset = top_right * 4;
                    } else if rel_y > 7 {
                        palette_offset = bottom_right * 4;
                    }
                }

                let palette_color = PALETTE_ADDRESS + palette_offset as u16;

                // --------------------- DRAW BACKGROUND -----------------------

                let background_draw = self.draw_tile(
                    bus,
                    canvas,
                    pattern_byte_0,
                    pattern_byte_1,
                    n as u8,
                    palette_color,
                    false,
                );

                // --------------------- DRAW SPRITES ON CURRENT LINE -----------------------
                for j in 0..self.sprite_index {
                    let n = self.sprites[j] as usize;
                    let x = self.oam[n * 4 + 3];
                    let y = self.oam[n * 4];
                    if self.cycle as u8 >= x
                        && self.cycle as u8 <= x + 7
                        && self.line as u8 <= y.wrapping_add(7)
                        && self.line as u8 >= y
                    {
                        let tile = self.oam[n * 4 + 1];
                        let attr = self.oam[n * 4 + 2];
                        let line = self.line as u8 - y;

                        let flip_hor = get_u8_bit(attr, 6) == 1;
                        let flip_ver = get_u8_bit(attr, 7) == 1;

                        let palette_color = PALETTE_ADDRESS + (attr & 0b00000011) as u16;
                        let mut x_offset = self.cycle as u8 - x;
                        if flip_hor {
                            x_offset = 7 - x_offset;
                        }

                        if self.control.sprite_size == 1 {
                            for i in 0..2 {
                                let (pattern_byte_0, pattern_byte_1) = self.get_pattern_address(
                                    bus,
                                    (tile >> 1 << 1) + i,
                                    line as u16,
                                    get_u8_bit(tile, 0),
                                );
                                self.draw_tile(
                                    bus,
                                    canvas,
                                    pattern_byte_0,
                                    pattern_byte_1,
                                    x_offset,
                                    palette_color,
                                    false,
                                );
                                self.line += 7;
                            }
                            self.line -= 14;
                        } else {
                            let (pattern_byte_0, pattern_byte_1) = self.get_pattern_address(
                                bus,
                                tile,
                                line as u16,
                                self.control.sprite_address,
                            );
                            let drew = self.draw_tile(
                                bus,
                                canvas,
                                pattern_byte_0,
                                pattern_byte_1,
                                x_offset,
                                palette_color,
                                false,
                            );
                            {
                                if drew && n == 0 && background_draw {
                                    self.status.hit = true;
                                    self.status.write(bus);
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.line == 240 && self.cycle == 1 {
            self.status.vblank = true;
            self.status.write(bus);
            if self.control.nmi {
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
            self.sprite_index = 0;
        }
        cycles
    }
}
