mod instructions;

use instructions::*;
use instructions::common::*;
use instructions::bitwise::*;
use instructions::branch::*;
use instructions::compare::*;
use instructions::jump::*;
use instructions::memory::*;
use instructions::stack::*;
use instructions::status::*;
use instructions::transfer::*;

use crate::opcode;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::mem::Memory;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone)]
pub enum AddressMode {
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

/*
http://wiki.nesdev.com/w/index.php/Status_flags

    7  bit  0
    ---- ----
    NV_B DIZC
    || | ||||
    || | |||+- Carry
    || | ||+-- Zero
    || | |+--- Interrupt Disable
    || | +---- Decimal
    || +------ Break
    ||
    |+-------- Overflow
    +--------- Negative
*/

bitflags::bitflags! {
    pub struct CPUStatus: u8 {
        const NEGATIVE          = 0b1000_0000;
        const OVERFLOW          = 0b0100_0000;
        const UNUSED            = 0b0010_0000;
        const BREAK             = 0b0001_0000;
        const DECIMAL           = 0b0000_1000;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const ZERO              = 0b0000_0010;
        const CARRY             = 0b0000_0001;
    }
}

pub struct CPU {
    pub pc: u16,
    pub sp: u8,
    pub acc: u8,
    pub rx: u8,
    pub ry: u8,
    pub status: CPUStatus,
    pub bus: Bus,

    history: Vec<opcode::Opcode>,
    codes: HashSet<String>
}

impl Memory for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }    

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        self.bus.mem_read_u16(addr)
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        self.bus.mem_write_u16(addr, data);
    }
}

pub trait With<T> {
    fn with(value: T) -> Self;
}

impl With<Vec<u8>> for CPU {
    fn with(value: Vec<u8>) -> Self {
        CPU {
            pc: 0,
            sp: STACK_RESET_LOC,
            acc: 0,
            rx: 0,
            ry: 0,
            status: CPUStatus::from_bits_truncate(CPUStatus::UNUSED.bits()),
            bus: Bus::new(Cartridge::new(&value).unwrap()),

            history: Vec::new(),
            codes: HashSet::new()
        }
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            pc: 0,
            sp: STACK_RESET_LOC,
            acc: 0,
            rx: 0,
            ry: 0,
            status: CPUStatus::from_bits_truncate(CPUStatus::UNUSED.bits()),
            bus: bus,

            history: Vec::new(),
            codes: HashSet::new()
        }
    }

    pub fn reset(&mut self) {
        self.acc = 0;
        self.rx = 0;
        self.ry = 0;
        self.status = CPUStatus::from_bits_truncate(CPUStatus::UNUSED.bits());

        self.pc = self.mem_read_u16(RESET_INTERRUPT_MEM_LOC);
        self.sp = STACK_RESET_LOC;
    }

    pub fn get_absolute_address(&self, mode: &AddressMode, addr: u16) -> u16 {
        match mode {
            AddressMode::ZeroPage => {
                self.mem_read(addr) as u16
            }
            AddressMode::ZeroPageX => {
                let pos = self.mem_read(addr);
                pos.wrapping_add(self.rx) as u16
            }
            AddressMode::ZeroPageY => {
                let pos = self.mem_read(addr);
                pos.wrapping_add(self.ry) as u16
            }
            AddressMode::Absolute => {
                self.mem_read_u16(addr)
            }
            AddressMode::AbsoluteX => {
                let pos = self.mem_read_u16(addr);
                pos.wrapping_add(self.rx as u16) as u16
            }
            AddressMode::AbsoluteY => {
                let pos = self.mem_read_u16(addr);
                pos.wrapping_add(self.ry as u16) as u16
            }
            AddressMode::IndirectX => {
                let base = self.mem_read(addr);
                let ptr = (base as u8).wrapping_add(self.rx) as u8;
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressMode::IndirectY => {
                let base = self.mem_read(addr);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.ry as u16)
            }
            _ => {
                panic!("not support for {:?}", mode)
            }
        }
    }

    pub fn get_operand_address(&self, mode: &AddressMode) -> u16 {
        match mode {
            AddressMode::Immediate => {
                self.pc
            }
            _ => {
                self.get_absolute_address(mode, self.pc)
            }
        }
    }

    pub fn run(&mut self) {
        self.reset();
        self.interprect();
    }

    pub fn interprect(&mut self) {
        self.interprect_with_callback(|_|{});
    }

    pub fn interprect_with_callback<T>(&mut self, mut callback: T) where T: FnMut(&mut CPU) -> () {
        let ref opcodes: HashMap<u8, &'static opcode::Opcode> = *opcode::OPCODES_MAP;
        // loop {
            callback(self);

            let op = self.mem_read(self.pc);
            self.pc += 1;
            let pc_state = self.pc;

            let code = opcodes.get(&op).expect(&format!("op: {:x} not exists or not impl.", op));
            // self.history.push(**code);
            self.codes.insert(String::from(code.name));

            match op {
                0x00 => {
                    self.reset();
                    // println!("{:?}", self.codes);
                    // return;
                    // self.brk();
                }
                // NOP
                0xEA => {

                }
                // TRANSFER
                0xAA => {
                    tax(self);
                }
                0xA8 => {
                    tay(self);
                }
                0x8A => {
                    txa(self);
                }
                0x98 => {
                    tya(self);
                }
                0xBA => {
                    tsx(self);
                }
                0x9A => {
                    txs(self);
                }
                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    lda(self, &code.mode);
                }
                // LDX
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    ldx(self, &code.mode);
                }
                // LDY
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    ldy(self, &code.mode);
                }
                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    sta(self, &code.mode);
                }
                // STX
                0x86 | 0x96 | 0x8E => {
                    stx(self, &code.mode);
                }
                // STY
                0x84 | 0x94 | 0x8C => {
                    sty(self, &code.mode);
                }
                // ADC
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    adc(self, &code.mode);
                }
                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    and(self, &code.mode);
                }
                // EOR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    eor(self, &code.mode);
                }
                // ORA
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    ora(self, &code.mode);
                }
                // ASL
                0x0A => {
                    asl_acc(self);
                }
                0x06 | 0x16 | 0x0E | 0x1E => {
                    asl(self, &code.mode);
                }
                // LSR
                0x4A => {
                    lsr_acc(self);
                }
                0x46 | 0x56 | 0x4E | 0x5E => {
                    lsr(self, &code.mode);
                }
                // ROL
                0x2A => {
                    rol_acc(self);
                }
                0x26 | 0x36 | 0x2E | 0x3E => {
                    rol(self, &code.mode);
                }
                // ROR
                0x6A => {
                    ror_acc(self);
                }
                0x66 | 0x76 | 0x6E | 0x7E => {
                    ror(self, &code.mode);
                }
                // BRANCH
                0x90 => {
                    bcc(self);
                }
                0xB0 => {
                    bcs(self);
                }
                0xF0 => {
                    beq(self);
                }
                0x30 => {
                    bmi(self);
                }
                0xD0 => {
                    bne(self);
                }
                0x10 => {
                    bpl(self);
                }
                0x50 => {
                    bvc(self);
                }
                0x70 => {
                    bvs(self);
                }
                // SBC
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 |0xF1 => {
                    sbc(self, &code.mode);
                }
                // BIT
                0x24 | 0x2C => {
                    bit(self, &code.mode);
                }
                // CLEAR
                0x18 => {
                    clc(self);
                }
                0xD8 => {
                    cld(self);
                }
                0x58 => {
                    cli(self);
                }
                0xB8 => {
                    clv(self);
                }
                // COMPARE
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    cmp(self, &code.mode);
                }
                0xE0 | 0xE4 | 0xEC => {
                    cpx(self, &code.mode);
                }
                0xC0 | 0xC4 | 0xCC => {
                    cpy(self, &code.mode);
                }
                // DEC
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    dec(self, &code.mode);
                }
                // DEX
                0xCA => {
                    dex(self);
                }
                // DEY
                0x88 => {
                    dey(self);
                }
                // INC
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    inc(self, &code.mode);
                }
                // INX
                0xE8 => {
                    inx(self);
                }
                // INY
                0xC8 => {
                    iny(self);
                }
                // PHP
                0x08 => {
                    php(self);
                }
                // PHA
                0x48 => {
                    pha(self);
                }
                // PLP
                0x28 => {
                    plp(self);
                }
                // PLA
                0x68 => {
                    pla(self);
                }
                // JSR
                0x20 => {
                    jsr(self, &code.mode);
                }
                // RTS
                0x60 => {
                    rts(self);
                }
                // RTI
                0x40 => {
                    rti(self);
                }
                // SET
                0x38 => {
                    sec(self);
                }
                0xF8 => {
                    sed(self);
                }
                0x78 => {
                    sei(self);
                }
                // JMP
                0x4C => {
                    // absolute
                    let addr = self.mem_read_u16(self.pc);
                    self.pc = addr;
                }
                0x6C => {
                    // indirect
                    // JMP is the only 6502 instruction to support indirection.
                    // The instruction contains a 16 bit address 
                    // which identifies the location of the least significant byte of another 16 bit memory address
                    // which is the real target of the instruction.
                    // http://www.obelisk.me.uk/6502/addressing.html#IND
                    let addr = self.mem_read_u16(self.pc);
                    let indirect_ref = if addr & 0x00FF == 0x00FF {
                        let lo = self.mem_read(addr);
                        let hi = self.mem_read(addr & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(addr)
                    };

                    self.pc = indirect_ref;
                }
                _ => {

                }
            }

            if pc_state == self.pc {
                self.pc += (code.bytes - 1) as u16
            }
        // }
    }
}