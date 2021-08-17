use super::BitwiseRegister;

/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUCTRL
    Controller ($2000) > write
    Common name: PPUCTRL
    Description: PPU control register
    Access: write
    Various flags controlling PPU operation

    7  bit  0
    ---- ----
    VPHB SINN
    |||| ||||
    |||| ||++- Base nametable address
    |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    |||| |     (0: add 1, going across; 1: add 32, going down)
    |||| +---- Sprite pattern table address for 8x8 sprites
    ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    |||+------ Background pattern table address (0: $0000; 1: $1000)
    ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    |+-------- PPU master/slave select
    |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    +--------- Generate an NMI at the start of the
            vertical blanking interval (0: off; 1: on)
*/
bitflags::bitflags! {
    pub struct PPUCTRL : u8 {
        const NAMETABLE_ADDR_L = 0b0000_0001;
        const NAMETABLE_ADDR_M = 0b0000_0010;
        const VRAM_ADDR_INC    = 0b0000_0100;
        const SPR_PTN_ADDR     = 0b0000_1000;
        const BG_PTN_ADDR      = 0b0001_0000;
        const SPR_SIZE         = 0b0010_0000;
        const MASTER_SLAVE_SEL = 0b0100_0000;
        const GEN_NMI_VBI      = 0b1000_0000;
    }
}

impl PPUCTRL {
    pub fn new() -> Self {
        PPUCTRL::from_bits_truncate(0b0000_0000)
    }

    pub fn get_nametable_address(&self) -> u16 {
        match self.bits & 0b0000_0011 {
            0b00 => 0x2000,
            0b01 => 0x2400,
            0b10 => 0x2800,
            0b11 => 0x2C00,
            _ => panic!("error")
        }
    }

    pub fn get_vram_address_increment(&self) -> u8 {
        if !self.contains(PPUCTRL::VRAM_ADDR_INC) {
            1
        } else {
            32
        }
    }

    pub fn get_sprite_pattern_table_address(&self) -> u16 {
        if !self.contains(PPUCTRL::SPR_PTN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn get_background_pattern_table_address(&self) -> u16 {
        if !self.contains(PPUCTRL::BG_PTN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn get_sprite_size(&self) -> u8 {
        if !self.contains(PPUCTRL::SPR_SIZE) {
            8
        } else {
            16
        }
    }

    pub fn get_master_slave_select(&self) -> bool {
        self.contains(PPUCTRL::MASTER_SLAVE_SEL) 
    }

    pub fn get_generate_nmi(&self) -> bool {
        self.contains(PPUCTRL::GEN_NMI_VBI) 
    }
}

impl BitwiseRegister for PPUCTRL {
    fn update_bits(&mut self, bits: u8) {
        self.bits = bits;
    }
}