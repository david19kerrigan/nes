use crate::util::*;
use crate::Bus;

pub struct Ppu {
	oam: [u8; 256],
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
			oam: [0; 256],
		}
    }

	pub fn get_ctrl(bus: &mut Bus) -> u8 {
		bus.read_16(0x2000, Component::CPU)
	}

	pub fn get_mask(bus: &mut Bus) -> u8 {
		bus.read_16(0x2001, Component::CPU)
	}

	pub fn get_status(bus: &mut Bus) -> u8 {
		bus.read_16(0x2002, Component::CPU)
	}

	pub fn get_oam_addr(bus: &mut Bus) -> u8 {
		bus.read_16(0x2003, Component::CPU)
	}

	pub fn get_oam_data(bus: &mut Bus) -> u8 {
		bus.read_16(0x2004, Component::CPU)
	}

	pub fn get_scroll(bus: &mut Bus) -> u8 {
		bus.read_16(0x2005, Component::CPU)
	}

	pub fn get_addr(bus: &mut Bus) -> u8 {
		bus.read_16(0x2006, Component::CPU)
	}

	pub fn get_data(bus: &mut Bus) -> u8 {
		bus.read_16(0x2007, Component::CPU)
	}

	pub fn get_oam_dma(bus: &mut Bus) -> u8 {
		bus.read_16(0x4014, Component::CPU)
	}
}
