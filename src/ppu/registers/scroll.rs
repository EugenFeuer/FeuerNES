/*
https://wiki.nesdev.com/w/index.php/PPU_registers#PPUSCROLL
    Scroll ($2005) >> write x2
    Common name: PPUSCROLL
    Description: PPU scrolling position register
    Access: write twice
*/

pub struct PPUSCROLL {
    cam_position_x: u8,
    cam_position_y: u8,
    latch: bool
}

impl PPUSCROLL {
    pub fn new() -> Self {
        PPUSCROLL {
            cam_position_x: 0,
            cam_position_y: 0,
            latch: true
        }
    }

    pub fn write(&mut self, cam_position: u8) {
        if self.latch {
            self.cam_position_x = cam_position;
        } else {
            self.cam_position_y = cam_position;
        }

        self.latch = !self.latch;
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }
}