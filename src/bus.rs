use crate::mem;

const RAM_BEGIN: u16 = 0x0000;
const RAM_END:   u16 = 0x1FFF;

const PPU_BEGIN: u16 = 0x2000;
const PPU_END:   u16 = 0x3FFF;

pub struct Bus {
    mem: mem::Memory,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            mem: mem::Memory::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
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
        self.mem.read(addr)
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        self.mem.read_u16(addr)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.mem.write(addr, data)
    }

    pub fn write_u16(&mut self, addr: u16, data: u16) {
        self.mem.write_u16(addr, data)
    }

    pub fn load(&mut self, bytes:[u8; mem::MEMORY_CAP]) {
        self.mem.load(bytes)
    }
}