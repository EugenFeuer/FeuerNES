/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUADDR
    Address ($2006) >> write x2
    Common name: PPUADDR
    Description: PPU address register
    Access: write twice
*/
pub struct PPUADDR {
    vram_addr: u16,
    write_hi: bool,
}

impl PPUADDR {
    pub fn new() -> Self {
        PPUADDR {
            vram_addr: 0,
            write_hi: true,
        }
    }

    pub fn get_address(&self) -> u16 {
        self.vram_addr
    }

    pub fn write_address(&mut self, addr: u8) {
        if self.write_hi {
            self.vram_addr = (addr as u16) << 8;
        } else {
            self.vram_addr |= addr as u16;
        }
        self.write_hi = !self.write_hi;

        self.mirror_down();
    }

    pub fn increment_address(&mut self, inc: u8) {
        self.vram_addr.wrapping_add(inc as u16);

        self.mirror_down();
    }

    fn mirror_down(&mut self) {
        if self.vram_addr > 0x3FFF {
            self.vram_addr &= 0x3FFF;
        }
    }

    pub fn reset_latch(&mut self) {
        self.write_hi = true;
    }
}
