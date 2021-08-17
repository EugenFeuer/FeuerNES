﻿/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUADDR
    Address ($2006) >> write x2
    Common name: PPUADDR
    Description: PPU address register
    Access: write twice
*/
pub struct PPUADDR {
    vram_addr: u16,
    write_hi: bool
}

impl PPUADDR {
    pub fn new() -> Self {
        PPUADDR {
            vram_addr: 0,
            write_hi: true
        }
    }

    pub fn write_address(&mut self, addr: u8) {
        if self.write_hi {
            self.vram_addr = (addr as u16) << 8;
        } else {
            self.vram_addr |= addr as u16;
        }

        // mirror down
        if self.vram_addr > 0x3FFF {
            self.vram_addr &= 0x3FFF;
        }

        self.write_hi = !self.write_hi;
    }

    pub fn reset_latch(&mut self) {
        self.write_hi = true;
    }
}