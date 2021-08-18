/*
https://wiki.nesdev.com/w/index.php/PPU_registers#OAMDATA
    OAM data ($2004) <> read/write
    Common name: OAMDATA
    Description: OAM data port
    Access: read, write
*/
pub struct OAMDATA {
    oam_data: u8,
}

impl OAMDATA {
    pub fn new() -> Self {
        OAMDATA { oam_data: 0 }
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data = data;
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data
    }
}
