use crate::cartridge;
use crate::mem;
use crate::ppu::registers::BitwiseRegister;
use crate::ppu::*;

const RAM_BEGIN: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_REG_MIRROR_BEGIN: u16 = 0x2008; // 0x2000-0x2007 is ppu registers, mirror to it
const PPU_REG_MIRROR_END: u16 = 0x3FFF;

const PRG_BEGIN: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

pub struct Bus {
    vram: [u8; 0x800],
    prg_rom: Vec<u8>,
    // cartridge: cartridge::Cartridge,
    ppu: PPU,
}

impl Bus {
    pub fn new(cartridge: cartridge::Cartridge) -> Self {
        Bus {
            vram: [0; 0x800],
            prg_rom: cartridge.prg,
            // cartridge: cartridge,
            ppu: PPU::new(cartridge.chr, cartridge.mirroring_type),
        }
    }

    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        // mirror
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }
}

impl mem::Memory for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_BEGIN..=RAM_END => {
                // mirror down 0x0000-0x1FFF -> 0x0000-0x7FF
                self.vram[(addr & 0x7FF) as usize]
            }
            PPU_REG_CTRL | PPU_REG_MASK | PPU_REG_OAMADDR | PPU_REG_SCROLL | PPU_REG_ADDR
            | PPU_REG_OAMDMA => {
                panic!("accessing write only ppu register {:x} !", addr);
            }
            PPU_REG_STATUS => {
                todo!();
            }
            PPU_REG_OAMDATA => self.ppu.oam_data_register.read_oam_data(),
            PPU_REG_DATA => self.ppu.read(),
            PPU_REG_MIRROR_BEGIN..=PPU_REG_MIRROR_END => {
                // mirror down to 0x2000-0x2007
                self.mem_read(addr & 0x2007)
            }
            PRG_BEGIN..=PRG_END => {
                // reading prg rom
                self.read_prg_rom(addr)
            }
            _ => {
                println!("ignore reading memory from: {:#02X}, return 0", addr);
                return 0;
            }
        }
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_BEGIN..=RAM_END => {
                // mirror down 0x0000-0x1FFF -> 0x0000-0x7FF
                self.vram[(addr & 0x7FF) as usize] = data;
            }
            PPU_REG_CTRL => {
                self.ppu.ctrl_register.update_bits(data);
            }
            PPU_REG_MASK => {
                self.ppu.mask_register.update_bits(data);
            }
            PPU_REG_STATUS => {
                panic!("writing to read only ppu register {:x} !", addr);
            }
            PPU_REG_OAMADDR => {
                self.ppu.oam_address_register.write_oam_address(data);
            }
            PPU_REG_OAMDATA => {
                self.ppu.oam_data_register.write_oam_data(data);
            }
            PPU_REG_SCROLL => {
                self.ppu.scroll_register.write(data);
            }
            PPU_REG_ADDR => {
                self.ppu.address_register.write_address(data);
            }
            PPU_REG_DATA => {
                self.ppu.write(data);
            }
            PPU_REG_MIRROR_BEGIN..=PPU_REG_MIRROR_END => {
                // writing ppu
            }
            PRG_BEGIN..=PRG_END => {
                panic!("cannot write to PRG ROM!");
            }
            _ => {
                println!("ignore writing memory to: {:#02X}", addr);
            }
        }
    }
}
