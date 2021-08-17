﻿use crate::cartridge::MirroringType;

mod registers;
use self::registers::controller::*;
use self::registers::mask::*;
use self::registers::status::*;
use self::registers::oam_address::*;
use self::registers::oam_data::*;
use self::registers::scroll::*;
use self::registers::address::*;
use self::registers::data::*;

pub struct PPU {
    pub chr: Vec<u8>,
    pub palette: [u8; 32],
    pub vram: [u8; 2048],
    pub oam: [u8; 256],
    pub mirroring_type: MirroringType,

    // registers from $2000 to $2007
    pub ctrl_register: PPUCTRL,
    pub mask_register: PPUMASK,
    pub status_register: PPUSTATUS,
    pub oam_address_register: OAMADDR,
    pub oam_data_register: OAMDATA,
    pub scroll_register: PPUSCROLL,
    pub address_register: PPUADDR,
    pub data_register: PPUDATA,
}

impl PPU {
    pub fn new(chr: Vec<u8>, mirroring_type: MirroringType) -> Self {
        PPU {
            chr: chr,
            palette: [0; 32],
            vram: [0; 2048],
            oam: [0; 256],
            mirroring_type: mirroring_type,

            ctrl_register: PPUCTRL::new(),
            mask_register: PPUMASK::new(),
            status_register: PPUSTATUS::new(),
            oam_address_register: OAMADDR::new(),
            oam_data_register: OAMDATA::new(),
            scroll_register: PPUSCROLL::new(),
            address_register: PPUADDR::new(),
            data_register: PPUDATA::new(),
        }
    }
}