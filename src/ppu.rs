use crate::util::*;
use crate::Bus;

pub struct Status {
    vblank: bool,
    hit: bool,
    overflow: bool,
    bus: u8,
}

pub struct Control {
    nametable_address: u8,
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
        let byte = bus.cpu_read_16(0x2000);
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
            nametable_address: 0,
            vram_increment: 0,
            sprite_address: 0,
            background_address: 0,
            sprite_size: 0,
            master_slave: false,
            nmi: false,
        }
    }

    pub fn read(&mut self, bus: &mut Bus) {
        let byte = bus.cpu_read_16(0x2000);
        self.nmi = get_u8_bit(byte, 7) == 1;
        self.master_slave = get_u8_bit(byte, 6) == 1;
        self.sprite_size = get_u8_bit(byte, 5);
        self.background_address = get_u8_bit(byte, 4);
        self.sprite_address = get_u8_bit(byte, 3);
        self.vram_increment = get_u8_bit(byte, 2);
        self.nametable_address = get_u8_bits(byte, 1, 0);
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
        bus.cpu_write_16(0x2002, byte);
    }
}

pub struct Ppu {
    oam: [u8; 256],
    cycle: u16,
    line: u16,
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
        bus.ppu_write_16(self.addr, data);
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

    pub fn tick(&mut self, bus: &mut Bus) {
        if self.cycle > 340 {
            self.cycle = 0;
            self.line += 1;
        }
        if self.line > 261 {
            self.line = 0;
        }

        if self.line < 240 {
            if self.cycle >= 1 && self.cycle <= 256 {
                let order = (self.cycle - 1) % 8;
                match order {
                    0 => (),
                    2 => (),
                    4 => (),
                    6 => (),
                    _ => panic!(),
                }
            }
        } else if self.line == 261 {
        }

        if self.line == 240 && self.cycle == 1 {
            self.status.vblank = true;
            self.status.write(bus);
        } else if self.line == 0 {
            self.status.vblank = false;
            self.status.write(bus);
        }

        self.cycle += 1;
    }
}
