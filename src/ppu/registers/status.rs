use super::BitwiseRegister;

/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUSTATUS
    Status ($2002) < read
    Common name: PPUSTATUS
    Description: PPU status register
    Access: read
    This register reflects the state of various functions inside the PPU. It is often used for determining timing. To determine when the PPU has reached a given pixel of the screen, put an opaque (non-transparent) pixel of sprite 0 there.

    7  bit  0
    ---- ----
    VSO. ....
    |||| ||||
    |||+-++++- Least significant bits previously written into a PPU register
    |||        (due to register not being updated for this address)
    ||+------- Sprite overflow. The intent was for this flag to be set
    ||         whenever more than eight sprites appear on a scanline, but a
    ||         hardware bug causes the actual behavior to be more complicated
    ||         and generate false positives as well as false negatives; see
    ||         PPU sprite evaluation. This flag is set during sprite
    ||         evaluation and cleared at dot 1 (the second dot) of the
    ||         pre-render line.
    |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    |          a nonzero background pixel; cleared at dot 1 of the pre-render
    |          line.  Used for raster timing.
    +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
            Set at dot 1 of line 241 (the line *after* the post-render
            line); cleared after reading $2002 and at dot 1 of the
            pre-render line.
*/
bitflags::bitflags! {
    pub struct PPUSTATUS : u8 {
        const NOT_USED_0   = 0b0000_0001;
        const NOT_USED_1   = 0b0000_0010;
        const NOT_USED_2   = 0b0000_0100;
        const NOT_USED_3   = 0b0000_1000;
        const NOT_USED_4   = 0b0001_0000;

        const SPR_OVERFLOW = 0b0010_0000;
        const SPR_ZERO_HIT = 0b0100_0000;
        const VBLANK       = 0b1000_0000;
    }
}

impl PPUSTATUS {
    pub fn new() -> Self {
        PPUSTATUS::from_bits_truncate(0b0000_0000)
    }
    
    pub fn set_sprite_overflow(&mut self, flag: bool) {
        self.set(PPUSTATUS::SPR_OVERFLOW, flag);
    }

    pub fn set_sprite_zero_hit(&mut self, flag: bool) {
        self.set(PPUSTATUS::SPR_ZERO_HIT, flag);
    }

    pub fn get_vertical_blank(&self) -> bool {
        self.contains(PPUSTATUS::VBLANK)
    }

    pub fn set_vertical_blank(&mut self, flag: bool) {
        self.set(PPUSTATUS::VBLANK, flag);
    }

    pub fn reset_vertical_blank(&mut self) {
        self.remove(PPUSTATUS::VBLANK);
    }
}

impl BitwiseRegister for PPUSTATUS {
    fn get_bits(&self) -> u8{
        self.bits
    }
}