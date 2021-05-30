const PROGRAM_BEGIN_LOC: u16 = 0x8000;
const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

const MEMORY_CAP: usize = 0xFFFF;

#[derive(Debug)]
enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing
}

pub struct Memory {
    raw_mem: [u8; MEMORY_CAP]
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            raw_mem: [0u8; MEMORY_CAP]
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.raw_mem[addr as usize]
    }

    // little-endian
    pub fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.raw_mem[addr as usize] = data;
    }

    pub fn write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.write(addr, lo);
        self.write(addr + 1, hi);
    }

    pub fn load(&mut self, bytes:[u8; MEMORY_CAP]) {
        self.raw_mem = bytes;
    }
}

pub struct CPU {
    pc: u16,
    sp: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    status: u8,
    mem: Memory
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            sp: 0,
            acc: 0,
            rx: 0,
            ry: 0,
            status: 0,
            mem: Memory::new()
        }
    }

    fn get_operand_address(&self, mode: &AddressMode) -> u16 {
        match mode {
            AddressMode::Immediate => {
                self.pc
            }
            AddressMode::ZeroPage => {
                self.mem.read(self.pc) as u16
            }
            AddressMode::ZeroPageX => {
                let pos = self.mem.read(self.pc);
                pos.wrapping_add(self.rx) as u16
            }
            AddressMode::ZeroPageY => {
                let pos = self.mem.read(self.pc);
                pos.wrapping_add(self.ry) as u16
            }
            AddressMode::Absolute => {
                self.mem.read_u16(self.pc)
            }
            AddressMode::AbsoluteX => {
                let pos = self.mem.read_u16(self.pc);
                pos.wrapping_add(self.rx as u16) as u16
            }
            AddressMode::AbsoluteY => {
                let pos = self.mem.read_u16(self.pc);
                pos.wrapping_add(self.ry as u16) as u16
            }
            AddressMode::IndirectX => {
                let base = self.mem.read(self.pc);
                let ptr = (base as u8).wrapping_add(self.rx) as u8;
                let lo = self.mem.read(ptr as u16);
                let hi = self.mem.read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressMode::IndirectY => {
                let base = self.mem.read(self.pc);
                let lo = self.mem.read(base as u16);
                let hi = self.mem.read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.ry as u16)
            }
            AddressMode::NoneAddressing => {
                panic!("not support for {:?}", mode)
            }
        }
    }

    fn update_zero_flag(&mut self, flag: u8) {
        if flag == 0 {
            self.status |= 0b0000_0010
        } else {
            self.status &= 0b1111_1101
        }
    }

    fn update_neg_flag(&mut self, flag: u8) {
        if flag & 0b1000_0000 != 0 {
            self.status |= 0b1000_0000
        } else {
            self.status &= 0b0111_1111
        }
    }

    fn lda(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);

        self.acc = value;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
    }

    fn sta(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.mem.write(addr, self.acc);
    }

    fn tax(&mut self) {
        self.rx = self.acc;
        self.update_neg_flag(self.rx);
        self.update_zero_flag(self.rx);
    }

    pub fn load(&mut self, bytes:[u8; 0xFFFF]) {
        self.mem.load(bytes);
        self.mem.write_u16(RESET_INTERRUPT_MEM_LOC, PROGRAM_BEGIN_LOC);
    }

    pub fn reset(&mut self) {
        self.acc = 0;
        self.rx = 0;
        self.status = 0;

        self.pc = self.mem.read_u16(RESET_INTERRUPT_MEM_LOC);
    }

    pub fn run(&mut self) {
        self.reset();
        self.interprect();
    }

    pub fn interprect(&mut self) -> () {
        loop {
            let opcode = self.mem.read(self.pc);
            self.pc += 1;

            match opcode {
                0x00 => {
                    return;
                }
                // begin of LDA
                0xA9 => {
                    self.lda(&AddressMode::Immediate);
                    self.pc += 1;
                }
                0xA5 => {
                    self.lda(&AddressMode::ZeroPage);
                    self.pc += 1;
                }
                0xB5 => {
                    self.lda(&AddressMode::ZeroPageX);
                    self.pc += 1;
                }
                0xAD => {
                    self.lda(&AddressMode::Absolute);
                    self.pc += 2;
                }
                0xBD => {
                    self.lda(&AddressMode::AbsoluteX);
                    self.pc += 2;
                }
                0xB9 => {
                    self.lda(&AddressMode::AbsoluteY);
                    self.pc += 2;
                }
                0xA1 => {
                    self.lda(&AddressMode::IndirectX);
                    self.pc += 1;
                }
                0xB1 => {
                    self.lda(&AddressMode::IndirectY);
                    self.pc += 1;
                }
                // end of LDA
                0xAA => {
                    self.pc += 1;
                    self.tax();
                }

                // begin of STA
                0x85 => {
                    self.sta(&AddressMode::ZeroPage);
                    self.pc += 1;
                }
                0x95 => {
                    self.sta(&AddressMode::ZeroPageX);
                    self.pc += 1;
                }
                0x8D => {
                    self.sta(&AddressMode::Absolute);
                    self.pc += 2;
                }
                0x9D => {
                    self.sta(&AddressMode::AbsoluteX);
                    self.pc += 2;
                }
                0x99 => {
                    self.sta(&AddressMode::AbsoluteY);
                    self.pc += 2;
                }
                0x81 => {
                    self.sta(&AddressMode::IndirectX);
                    self.pc += 1;
                }
                0x91 => {
                    self.sta(&AddressMode::IndirectY);
                    self.pc += 1;
                }
                // end of STA

                _ => {

                }
            }
        }
    }
}