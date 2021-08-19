use crate::cartridge::MirroringType;

pub mod registers;
use self::registers::address::*;
use self::registers::controller::*;
use self::registers::data::*;
use self::registers::mask::*;
use self::registers::oam_address::*;
use self::registers::oam_data::*;
use self::registers::scroll::*;
use self::registers::status::*;

pub const PPU_REG_CTRL: u16 = 0x2000;
pub const PPU_REG_MASK: u16 = 0x2001;
pub const PPU_REG_STATUS: u16 = 0x2002;
pub const PPU_REG_OAMADDR: u16 = 0x2003;
pub const PPU_REG_OAMDATA: u16 = 0x2004;
pub const PPU_REG_SCROLL: u16 = 0x2005;
pub const PPU_REG_ADDR: u16 = 0x2006;
pub const PPU_REG_DATA: u16 = 0x2007;
pub const PPU_REG_OAMDMA: u16 = 0x4014;

const SCANLINE_CYCLES_COST: u16 = 341;
const SCANLINE_TRIGGER_NMI: u16 = 241;
const SCANLINE_PER_FRAME: u16 = 262;

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

    cycles: u16,
    scanlines: u16,
    should_nmi_flag: bool,
    internal_last_read_byte: u8,
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

            cycles: 0,
            scanlines: 0,
            should_nmi_flag: false,
            internal_last_read_byte: 0,
        }
    }

    pub fn read(&mut self) -> u8 {
        let addr = self.address_register.get_address();
        self.address_register
            .increment_address(self.ctrl_register.get_vram_address_increment());

        match addr {
            0x0000..=0x1FFF => {
                self.internal_last_read_byte = self.chr[addr as usize];
                self.internal_last_read_byte
            }
            0x2000..=0x2FFF => {
                self.internal_last_read_byte = self.vram[(addr - 0x2000) as usize];
                self.internal_last_read_byte
            }
            0x3000..=0x3EFF => panic!("not used"),
            0x3F00..=0x3FFF => self.palette[(addr - 0x3F00) as usize],
            _ => panic!("unexpected address access: {:x}", addr),
        }
    }

    pub fn write(&mut self, data: u8) {
        let addr = self.address_register.get_address();
        self.address_register
            .increment_address(self.ctrl_register.get_vram_address_increment());

        match addr {
            0x0000..=0x1FFF => panic!("writing to chr rom {:x}", addr),
            0x2000..=0x2FFF => self.vram[(addr - 0x2000) as usize] = data,
            0x3000..=0x3EFF => panic!("not used"),
            // mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let add_mirror = addr - 0x10;
                self.palette[(addr - 0x10 - 0x3F00) as usize] = data;
            }
            0x3F00..=0x3FFF => self.palette[(addr - 0x3F00) as usize] = data,
            _ => panic!("unexpected address access: {:x}", addr),
        }
    }

    pub fn get_mirror_vram_addr(&self, mut addr: u16) -> u16 {
        addr &= 0x2FFF; // 0x3000-0x3FFF -> 0x2000-0x2FFF (0x3F00-0x3FFF should not pass in)
        addr -= 0x2000; // 0x2000-0x2FFF -> 0x0000-0x0FFF
        let index = addr / 0x400; // 0x0000-0x0FFF -> 0-3 screen index
        match (&self.mirroring_type, index) {
            (MirroringType::Vertical, 2) | (MirroringType::Vertical, 3) => addr - 0x800, // 0x400-0x800
            (MirroringType::Horizontal, 1) => addr - 0x400,                              // 0-0x400
            (MirroringType::Horizontal, 2) => addr - 0x400, // 0x400-0x800
            (MirroringType::Horizontal, 3) => addr - 0x800, // 0x400-0x800
            _ => addr,                                      // no need to map
        }
    }

    pub fn tick(&mut self, cycles: u16) {
        self.cycles += cycles;

        if self.cycles >= SCANLINE_CYCLES_COST {
            self.cycles -= SCANLINE_CYCLES_COST;
            self.scanlines += 1;

            if self.scanlines == SCANLINE_TRIGGER_NMI {
                self.status_register.set_vertical_blank(true);
                self.status_register.set_sprite_zero_hit(false);

                if self.ctrl_register.get_generate_nmi() {
                    self.should_nmi_flag = true;
                }
            }

            if self.scanlines >= SCANLINE_PER_FRAME {
                self.scanlines = 0;
                self.should_nmi_flag = false;
                self.status_register.set_sprite_zero_hit(false);
                self.status_register.set_vertical_blank(false);
            }
        }
    }

    pub fn should_nmi(&mut self) -> bool {
        if self.should_nmi_flag {
            self.should_nmi_flag = false;
            return true;
        }
        return false;
    }
}
