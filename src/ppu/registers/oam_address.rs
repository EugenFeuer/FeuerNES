/*
https://wiki.nesdev.com/w/index.php/PPU_registers#OAMADDR
    OAM address ($2003) > write
    Common name: OAMADDR
    Description: OAM address port
    Access: write
*/

pub struct OAMADDR {
    oam_address: u8
}

impl OAMADDR {
    pub fn new() -> Self {
        OAMADDR {
            oam_address: 0
        }
    }

    pub fn write_oam_address(&mut self, addr: u8) {
        self.oam_address = addr;
    } 
}