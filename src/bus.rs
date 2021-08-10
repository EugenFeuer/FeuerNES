use crate::mem;
use crate::cartridge;

const RAM_BEGIN: u16 = 0x0000;
const RAM_END:   u16 = 0x1FFF;

const PPU_BEGIN: u16 = 0x2000;
const PPU_END:   u16 = 0x3FFF;

pub struct Bus {
    ram: [u8; 2000],
    cartridge: cartridge::Cartridge,
}

impl Bus {
    pub fn new(cartridge: cartridge::Cartridge) -> Self {
        Bus {
            ram: [0; 2000],
            cartridge: cartridge
        }
    }

    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        // mirror
        if self.cartridge.prg.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.cartridge.prg[addr as usize]
    }
}

impl mem::Memory for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM_BEGIN ..= RAM_END => {
                // reading cpu ram
            }
            PPU_BEGIN ..= PPU_END => {
                // reading ppu
            }
            _ => {
                println!("not impl");
                return 0;
            }
        }
        // TODO
        0
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        // TODO
        // match addr {

        // }
    }
}