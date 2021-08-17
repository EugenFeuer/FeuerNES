/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUDATA
    Data ($2007) <> read/write
    Common name: PPUDATA
    Description: PPU data port
    Access: read, write
*/
pub struct PPUDATA {
    data: u8
}

impl PPUDATA {
    pub fn new() -> Self {
        PPUDATA {
            data: 0
        }
    }

    pub fn write_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read_data(&self) -> u8 {
        self.data
    }
}