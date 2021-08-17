use super::BitwiseRegister;

/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUMASK
    Mask ($2001) > write
    Common name: PPUMASK
    Description: PPU mask register
    Access: write
    This register controls the rendering of sprites and backgrounds, as well as colour effects.

    7  bit  0
    ---- ----
    BGRs bMmG
    |||| ||||
    |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    |||| +---- 1: Show background
    |||+------ 1: Show sprites
    ||+------- Emphasize red (green on PAL/Dendy)
    |+-------- Emphasize green (red on PAL/Dendy)
    +--------- Emphasize blue
*/
bitflags::bitflags! {
    pub struct PPUMASK : u8 {
        const GREY_SCALE  = 0b0000_0001;
        const SHOW_BG_LM  = 0b0000_0010;
        const SHOW_SPR_LM = 0b0000_0100;
        const SHOW_BG     = 0b0000_1000;
        const SHOW_SPR    = 0b0001_0000;
        const EMPHA_RED   = 0b0010_0000;
        const EMPHA_GREEN = 0b0010_0000;
        const EMPHA_BLUE  = 0b0010_0000;
    }
}

impl PPUMASK {
    pub fn new() -> Self {
        PPUMASK::from_bits_truncate(0b0000_0000)
    }

    pub fn get_grey_scale(&self) -> bool {
        self.contains(PPUMASK::GREY_SCALE)
    }

    pub fn get_show_background_in_leftmost(&self) -> bool {
        self.contains(PPUMASK::SHOW_BG_LM)
    }

    pub fn get_show_sprites_in_leftmost(&self) -> bool {
        self.contains(PPUMASK::SHOW_SPR_LM)
    }

    pub fn get_show_background(&self) -> bool {
        self.contains(PPUMASK::SHOW_BG)
    }

    pub fn get_show_sprites(&self) -> bool {
        self.contains(PPUMASK::SHOW_SPR)
    }

    pub fn get_emphasize_red(&self) -> bool {
        self.contains(PPUMASK::EMPHA_RED)
    }

    pub fn get_emphasize_green(&self) -> bool {
        self.contains(PPUMASK::EMPHA_GREEN)
    }

    pub fn get_emphasize_blue(&self) -> bool {
        self.contains(PPUMASK::EMPHA_BLUE)
    }
}

impl BitwiseRegister for PPUMASK {
    fn update_bits(&mut self, bits: u8) {
        self.bits = bits;
    }
}