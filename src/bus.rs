use crate::mem;
use crate::cartridge;

const RAM_BEGIN: u16 = 0x0000;
const RAM_END:   u16 = 0x1FFF;

const CHR_BEGIN: u16 = 0x2000;
const CHR_END: u16 = 0x3FFF;

const PRG_BEGIN: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

pub struct Bus {
    ram: [u8; 0x2000],
    cartridge: cartridge::Cartridge,
}

impl Bus {
    pub fn new(cartridge: cartridge::Cartridge) -> Self {
        Bus {
            ram: [0; 0x2000],
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

impl  mem::Memory for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_BEGIN ..= RAM_END => {
                return self.ram[addr as usize];
            }
            CHR_BEGIN ..= CHR_END => {
                // reading ppu
            }
            PRG_BEGIN ..= PRG_END => {
                // reading prg rom
                return self.read_prg_rom(addr);
            }
            _ => {
                println!("ignore reading memory from: {:#02X}, return 0", addr);
                return 0;
            }
        }
        // TODO
        0
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_BEGIN ..= RAM_END => {
                // writing cpu ram
                self.ram[addr as usize] = data;
            }
            CHR_BEGIN ..= CHR_END => {
                // writing ppu
            }
            PRG_BEGIN ..= PRG_END => {
                panic!("cannot write to PRG ROM!");
            }
            _ => {
                println!("ignore writing memory to: {:#02X}", addr);
            }
        }
    }
}