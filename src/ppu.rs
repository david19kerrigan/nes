use crate::util::*;
use crate::Bus;

pub struct Control {
	nametable_address: u8,
	vram_increment: u8,
	sprite_address: u8,
	background_address: u8,
	sprite_size: u8,
	master_slave: bool,
	nmi: bool,
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
		let byte = bus.read_16(0x2000, Component::CPU);
		self.nmi = get_u8_bit(byte, 7) == 1;
		self.master_slave = get_u8_bit(byte, 6) == 1;
		self.sprite_size = get_u8_bit(byte, 5);
		self.background_address = get_u8_bit(byte, 4);
		self.sprite_address = get_u8_bit(byte, 3);
		self.vram_increment = get_u8_bit(byte, 2);
		self.nametable_address = get_u8_bits(byte, 1, 0);
	}
}

pub struct Status {
    vblank: bool,
    hit: bool,
    overflow: bool,
    bus: u8,
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

	pub fn to_u8(&mut self) -> u8 {
		(self.vblank as u8) << 7
		| (self.hit as u8) << 6
		| (self.overflow as u8) << 5
		| self.bus
	}

	pub fn write(&mut self, bus: &mut Bus) {
		bus.write_16(0x2002, self.to_u8(), Component::CPU);
	}
}

pub struct Ppu {
    oam: [u8; 256],
    cycle: u16,
    line: u16,
	status: Status,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 256],
            cycle: 0,
            line: 0,
			status: Status::new(),
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.line += 1;
        }
        if self.line > 261 {
            self.line = 0;
        }

        if self.line == 240 && self.cycle == 1 {
			self.status.vblank = true; self.status.write(bus);
        } else if self.line == 0 {
			self.status.vblank = false; self.status.write(bus);
        }
    }
}
