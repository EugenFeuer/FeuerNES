﻿use crate::opcode;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::mem::Memory;

use std::collections::HashMap;
use std::collections::HashSet;

const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

const STACK_BOTTOM_LOC: u16 = 0x0100;
const STACK_RESET_LOC: u8 = 0xFD;

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

        self.pc = self.bus.mem_read_u16(RESET_INTERRUPT_MEM_LOC);
        self.sp = STACK_RESET_LOC;
    }

    pub fn get_absolute_address(&self, mode: &AddressMode, addr: u16) -> u16 {
        match mode {
            AddressMode::ZeroPage => {
                self.bus.mem_read(addr) as u16
            }
            AddressMode::ZeroPageX => {
                let pos = self.bus.mem_read(addr);
                pos.wrapping_add(self.rx) as u16
            }
            AddressMode::ZeroPageY => {
                let pos = self.bus.mem_read(addr);
                pos.wrapping_add(self.ry) as u16
            }
            AddressMode::Absolute => {
                self.bus.mem_read_u16(addr)
            }
            AddressMode::AbsoluteX => {
                let pos = self.bus.mem_read_u16(addr);
                pos.wrapping_add(self.rx as u16) as u16
            }
            AddressMode::AbsoluteY => {
                let pos = self.bus.mem_read_u16(addr);
                pos.wrapping_add(self.ry as u16) as u16
            }
            AddressMode::IndirectX => {
                let base = self.bus.mem_read(addr);
                let ptr = (base as u8).wrapping_add(self.rx) as u8;
                let lo = self.bus.mem_read(ptr as u16);
                let hi = self.bus.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressMode::IndirectY => {
                let base = self.bus.mem_read(addr);
                let lo = self.bus.mem_read(base as u16);
                let hi = self.bus.mem_read(base.wrapping_add(1) as u16);
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

    fn update_zero_flag(&mut self, flag: u8) {
        if flag == 0 {
            self.status.insert(CPUStatus::ZERO);
        } else {
            self.status.remove(CPUStatus::ZERO);
        }
    }

    fn update_neg_flag(&mut self, flag: u8) {
        if flag & 0b1000_0000 != 0 {
            self.status.insert(CPUStatus::NEGATIVE);
        } else {
            self.status.remove(CPUStatus::NEGATIVE);
        }
    }

    fn update_overflow_flag(&mut self, flag: bool) {
        if flag {
            self.status.insert(CPUStatus::OVERFLOW);
        } else {
            self.status.remove(CPUStatus::OVERFLOW);
        }
    }

    fn update_carry_flag(&mut self, flag: bool) {
        if flag {
            self.status.insert(CPUStatus::CARRY);
        } else {
            self.status.remove(CPUStatus::CARRY);
        }
    }

    fn add_to_acc(&mut self, data: u8) {
        let cur_carry: u16 = if self.status.contains(CPUStatus::CARRY) {
            1
        } else {
            0
        };

        // A = A + M + C
        let sum = self.acc as u16 +
                  data     as u16 +
                  cur_carry;

        // update flags
        self.update_carry_flag(sum > 0xFF);

        let res = sum as u8;
        // (M ^ result) & (N ^ result) & 0x80 != 0
        self.update_overflow_flag((data ^ res) & (self.acc ^ res) & 0x80 != 0);

        self.acc = res;
    }

    fn adc(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.add_to_acc(self.bus.mem_read(addr));
    }

    fn and(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let res = self.acc & self.bus.mem_read(addr);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.acc = res;
    }

    fn ora(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let res = self.acc | self.bus.mem_read(addr);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.acc = res;
    }

    fn eor(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let res = self.acc ^ self.bus.mem_read(addr);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.acc = res;
    }

    fn rol_acc(&mut self) {
        let res = (self.acc << 1) | (0x01 & self.status.bits());

        self.update_carry_flag(self.acc >> 7 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);

        self.acc = res;
    }

    fn rol(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        let res = (value << 1) | (0x01 & self.status.bits());
        
        self.update_carry_flag(value >> 7 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.bus.mem_write(addr, res);
    }

    fn ror_acc(&mut self) {
        let res = (self.acc >> 1) | (self.status.bits() << 7);

        self.update_carry_flag(self.acc & 0x01 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);

        self.acc = res;
    }

    fn ror(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        let res = (value >> 1) | (self.status.bits() << 7);

        self.update_carry_flag(value & 0x01 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);

        self.acc = res;
    }

    fn lsr_acc(&mut self) {
        let res = self.acc >> 1;

        self.update_carry_flag(self.acc & 0x1 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.acc = res;
    }

    fn lsr(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        let res = value >> 1;
        
        self.update_carry_flag(value & 0x01 == 1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.bus.mem_write(addr, res);
    }

    fn asl_acc(&mut self) {
        let mut res = self.acc;

        self.update_carry_flag(res >> 7 == 1);

        res <<= 1;
        self.update_neg_flag(res);
        self.update_zero_flag(res);
        self.acc = res;
    }

    fn asl(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.bus.mem_read(addr);

        self.update_carry_flag(value >> 7 == 1);

        value <<= 1;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
        
        self.bus.mem_write(addr, value);
    }

    fn jsr(&mut self, mode: &AddressMode) {
        self.stack_push_u16(self.pc + 1);   // PC + 2 - 1
        let addr = self.get_operand_address(mode);
        self.pc = addr;
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_u16() + 1;
    }

    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CPUStatus::BREAK);
        self.pc = self.stack_pop_u16();
    }

    fn branch(&mut self, flag: bool) {
        if flag {
            let offset = self.bus.mem_read(self.pc) as i8;  // offset can be negative
            let dst = self.pc.wrapping_add(1).wrapping_add(offset as u16);
            self.pc = dst;
        }
    }

    fn bcc(&mut self) {
        self.branch(!self.status.contains(CPUStatus::CARRY));
    }

    fn bcs(&mut self) {
        self.branch(self.status.contains(CPUStatus::CARRY));
    }

    fn beq(&mut self) {
        self.branch(self.status.contains(CPUStatus::ZERO));
    }

    fn bmi(&mut self) {
        self.branch(self.status.contains(CPUStatus::NEGATIVE));
    }

    fn bne(&mut self) {
        self.branch(!self.status.contains(CPUStatus::ZERO));
    }

    fn bpl(&mut self) {
        self.branch(!self.status.contains(CPUStatus::NEGATIVE));
    }

    fn bvc(&mut self) {
        self.branch(!self.status.contains(CPUStatus::OVERFLOW));
    }

    fn bvs(&mut self) {
        self.branch(self.status.contains(CPUStatus::OVERFLOW));
    }

    fn sbc(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr) as i8;
        // A = A - M - (1 - C)
        self.add_to_acc((value.wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn bit(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        self.update_neg_flag(value);
        self.update_overflow_flag(value & 0b0100_0000 == 1);
        self.update_zero_flag(self.acc & value);
    }

    fn clc(&mut self) {
        self.status.remove(CPUStatus::CARRY);
    }

    fn cld(&mut self) {
        self.status.remove(CPUStatus::DECIMAL);
    }

    fn cli(&mut self) {
        self.status.remove(CPUStatus::INTERRUPT_DISABLE);
    }

    fn clv(&mut self) {
        self.status.remove(CPUStatus::OVERFLOW);
    }

    fn compare(&mut self, v1: u8, v2: u8) {
        self.update_carry_flag(v1 >= v2);
        let res = v1.wrapping_sub(v2);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
    }

    fn cmp(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        self.compare(self.acc, value);
    }

    fn cpx(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        self.compare(self.rx, value);
    }

    fn cpy(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);
        self.compare(self.ry, value);
    }

    fn dec(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        let res = value.wrapping_sub(1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);

        self.bus.mem_write(addr, res);
    }

    fn dex(&mut self) {
        self.rx = self.rx.wrapping_sub(1);
        self.update_zero_flag(self.rx);
        self.update_neg_flag(self.rx);
    }

    fn dey(&mut self) {
        self.ry = self.ry.wrapping_sub(1);
        self.update_zero_flag(self.ry);
        self.update_neg_flag(self.ry);
    }

    fn inc(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        let res = value.wrapping_add(1);
        self.update_zero_flag(res);
        self.update_neg_flag(res);

        self.bus.mem_write(addr, res);
    }

    fn inx(&mut self) {
        self.rx = self.rx.wrapping_add(1);
        self.update_zero_flag(self.rx);
        self.update_neg_flag(self.rx);
    }

    fn iny(&mut self) {
        self.ry = self.ry.wrapping_add(1);
        self.update_zero_flag(self.ry);
        self.update_neg_flag(self.ry);
    }

    fn stack_push(&mut self, value: u8) {
        self.bus.mem_write(self.sp as u16 + STACK_BOTTOM_LOC, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.mem_read(self.sp as u16 + STACK_BOTTOM_LOC)
    }

    fn stack_push_u16(&mut self, value: u16) {
        self.stack_push((value >> 8) as u8);    // hi
        self.stack_push(value as u8);           // lo
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn php(&mut self) {
        let mut s = self.status.clone();
        // http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        s.insert(CPUStatus::BREAK);
        s.insert(CPUStatus::UNUSED);
        self.stack_push(s.bits());
    }

    fn plp(&mut self) {
        let s = self.stack_pop();
        self.status.bits = s;
        self.status.remove(CPUStatus::BREAK);
    }

    fn pha(&mut self) {
        self.stack_push(self.acc);
    }

    fn pla(&mut self) {
        self.acc = self.stack_pop();
        self.update_neg_flag(self.acc);
        self.update_zero_flag(self.acc);
    }

    fn lda(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        self.acc = value;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
    }

    fn ldx(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        self.rx = value;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
    }

    fn ldy(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.mem_read(addr);

        self.ry = value;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
    }

    fn sta(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.bus.mem_write(addr, self.acc);
    }

    fn stx(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.bus.mem_write(addr, self.rx);
    }

    fn sty(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.bus.mem_write(addr, self.ry);
    }

    fn tax(&mut self) {
        self.rx = self.acc;
        self.update_neg_flag(self.rx);
        self.update_zero_flag(self.rx);
    }

    fn tay(&mut self) {
        self.ry = self.acc;
        self.update_neg_flag(self.ry);
        self.update_zero_flag(self.ry);
    }

    fn txa(&mut self) {
        self.acc = self.rx;
        self.update_neg_flag(self.acc);
        self.update_zero_flag(self.acc);
    }

    fn tya(&mut self) {
        self.acc = self.ry;
        self.update_neg_flag(self.acc);
        self.update_zero_flag(self.acc);
    }

    fn tsx(&mut self) {
        self.rx = self.sp;
        self.update_neg_flag(self.rx);
        self.update_zero_flag(self.rx);
    }

    fn txs(&mut self) {
        self.sp = self.rx;
        self.update_neg_flag(self.sp);
        self.update_zero_flag(self.sp);
    }

    fn sec(&mut self) {
        self.status.insert(CPUStatus::CARRY);
    }

    fn sed(&mut self) {
        self.status.insert(CPUStatus::DECIMAL);   
    }

    fn sei(&mut self) {
        self.status.insert(CPUStatus::INTERRUPT_DISABLE);
    }

    fn brk(&mut self) {
        self.stack_push_u16(self.pc);
        self.stack_push(self.status.bits());
        self.pc = self.bus.mem_read_u16(RESET_INTERRUPT_MEM_LOC);
        self.status.insert(CPUStatus::BREAK);
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

            let op = self.bus.mem_read(self.pc);
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
                    self.tax();
                }
                0xA8 => {
                    self.tay();
                }
                0x8A => {
                    self.txa();
                }
                0x98 => {
                    self.tya();
                }
                0xBA => {
                    self.tsx();
                }
                0x9A => {
                    self.txs();
                }
                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&code.mode);
                }
                // LDX
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&code.mode);
                }
                // LDY
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&code.mode);
                }
                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&code.mode);
                }
                // STX
                0x86 | 0x96 | 0x8E => {
                    self.stx(&code.mode);
                }
                // STY
                0x84 | 0x94 | 0x8C => {
                    self.sty(&code.mode);
                }
                // ADC
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&code.mode);
                }
                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&code.mode);
                }
                // EOR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&code.mode);
                }
                // ORA
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&code.mode);
                }
                // ASL
                0x0A => {
                    self.asl_acc();
                }
                0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&code.mode);
                }
                // LSR
                0x4A => {
                    self.lsr_acc();
                }
                0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(&code.mode);
                }
                // ROL
                0x2A => {
                    self.rol_acc();
                }
                0x26 | 0x36 | 0x2E | 0x3E => {
                    self.rol(&code.mode);
                }
                // ROR
                0x6A => {
                    self.ror_acc();
                }
                0x66 | 0x76 | 0x6E | 0x7E => {
                    self.ror(&code.mode);
                }
                // BRANCH
                0x90 => {
                    self.bcc();
                }
                0xB0 => {
                    self.bcs();
                }
                0xF0 => {
                    self.beq();
                }
                0x30 => {
                    self.bmi();
                }
                0xD0 => {
                    self.bne();
                }
                0x10 => {
                    self.bpl();
                }
                0x50 => {
                    self.bvc();
                }
                0x70 => {
                    self.bvs();
                }
                // SBC
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 |0xF1 => {
                    self.sbc(&code.mode);
                }
                // BIT
                0x24 | 0x2C => {
                    self.bit(&code.mode);
                }
                // CLEAR
                0x18 => {
                    self.clc();
                }
                0xD8 => {
                    self.cld();
                }
                0x58 => {
                    self.cli();
                }
                0xB8 => {
                    self.clv();
                }
                // COMPARE
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&code.mode);
                }
                0xE0 | 0xE4 | 0xEC => {
                    self.cpx(&code.mode);
                }
                0xC0 | 0xC4 | 0xCC => {
                    self.cpy(&code.mode);
                }
                // DEC
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    self.dec(&code.mode);
                }
                // DEX
                0xCA => {
                    self.dex();
                }
                // DEY
                0x88 => {
                    self.dey();
                }
                // INC
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    self.inc(&code.mode);
                }
                // INX
                0xE8 => {
                    self.inx();
                }
                // INY
                0xC8 => {
                    self.iny();
                }
                // PHP
                0x08 => {
                    self.php();
                }
                // PHA
                0x48 => {
                    self.pha();
                }
                // PLP
                0x28 => {
                    self.plp();
                }
                // PLA
                0x68 => {
                    self.pla();
                }
                // JSR
                0x20 => {
                    self.jsr(&code.mode);
                }
                // RTS
                0x60 => {
                    self.rts();
                }
                // RTI
                0x40 => {
                    self.rti();
                }
                // SET
                0x38 => {
                    self.sec();
                }
                0xF8 => {
                    self.sed();
                }
                0x78 => {
                    self.sei();
                }
                // JMP
                0x4C => {
                    // absolute
                    let addr = self.bus.mem_read_u16(self.pc);
                    self.pc = addr;
                }
                0x6C => {
                    // indirect
                    // JMP is the only 6502 instruction to support indirection.
                    // The instruction contains a 16 bit address 
                    // which identifies the location of the least significant byte of another 16 bit memory address
                    // which is the real target of the instruction.
                    // http://www.obelisk.me.uk/6502/addressing.html#IND
                    let addr = self.bus.mem_read_u16(self.pc);
                    let indirect_ref = if addr & 0x00FF == 0x00FF {
                        let lo = self.bus.mem_read(addr);
                        let hi = self.bus.mem_read(addr & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.bus.mem_read_u16(addr)
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



#[cfg(test)]
mod test {
    use super::*;

    /* test for ADC */
    #[test]
    fn test_adc() {
        let program = vec!(
            0x69, 0x10, 0x69, 0x20, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.acc, 0x30);
    }

    #[test]
    fn test_adc_overflow() {
        
        let program = vec!(
            0x69, 0xD0, 0x69, 0x90, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();
        
        assert!(cpu.status.contains(CPUStatus::OVERFLOW));
    }

    /* test for SBC */
    #[test]
    fn test_sbc() {
        
        let program = vec!(
            0x69, 0x10, 0xE9, 0x01, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();
        
        assert_eq!(cpu.acc, 0x0E);
    }

    /* test for AND */
    #[test]
    fn test_and() {
        
        let program = vec!(
            0x69, 0x0F, 0x29, 0x11, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();
        
        assert_eq!(cpu.acc, 0x01);
    }

    /* test for EOR */
    #[test]
    fn test_eor() {
        
        let program = vec!(
            0x69, 0x09, 0x49, 0x06, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();
        
        assert_eq!(cpu.acc, 0x0F);
    }

    /* test for ASL */
    #[test]
    fn test_asl() {
        
        let program = vec!(
            0x06, 0xFF, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.bus.mem_write(0x00FF, 0x10);
        cpu.run();
        
        assert_eq!(cpu.bus.mem_read(0x00FF), 0x20);
    }

    #[test]
    fn test_asl_acc() {
        
        let program = vec!(
            0x69, 0x10, 0x0A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();
        
        assert_eq!(cpu.acc, 0x20);
    }

    /* test for BRANCH */
    #[test]
    fn test_bcc() {
        
        let program = vec!(
            0x90, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.remove(CPUStatus::CARRY);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bcs() {
        
        let program = vec!(
            0xB0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.insert(CPUStatus::CARRY);
        cpu.interprect();
        
        assert_eq!(cpu.acc, 0x21);  // because the CARRY bit has been set
    }

    #[test]
    fn test_beq() {
        
        let program = vec!(
            0xF0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.insert(CPUStatus::ZERO);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bmi() {
        
        let program = vec!(
            0x30, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.insert(CPUStatus::NEGATIVE);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bne() {
        
        let program = vec!(
            0xD0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.remove(CPUStatus::ZERO);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bpl() {
        
        let program = vec!(
            0x10, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.remove(CPUStatus::NEGATIVE);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bvc() {
        
        let program = vec!(
            0x50, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.remove(CPUStatus::OVERFLOW);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_bvs() {
        
        let program = vec!(
            0x70, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.status.insert(CPUStatus::OVERFLOW);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    /* test for COMPARE */
    #[test]
    fn test_cmp1() {
        
        let program = vec!(
            0x69, 0x10, 0xC9, 0x0F, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert!(cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn test_cmp2() {
        
        let program = vec!(
            0x69, 0x10, 0xC9, 0x10, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert!(cpu.status.contains(CPUStatus::CARRY));
        assert!(cpu.status.contains(CPUStatus::ZERO));
    }

    /* test for TRANSFER */
    #[test]
    fn test_tax() {
        
        let program = vec!(
            0x69, 0x10, 0xAA, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_tay() {
        
        let program = vec!(
            0x69, 0x10, 0xA8, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.ry, 0x10);
    }

    #[test]
    fn test_txa() {
        
        let program = vec!(
            0x8A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tya() {
        
        let program = vec!(
            0x98, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.ry = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tsx() {
        
        let program = vec!(
            0xBA, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.sp = 0x10;
        cpu.interprect();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_txs() {
        
        let program = vec!(
            0x9A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.sp, 0x10);
    }

    /* test for LSR */
    #[test]
    fn test_lsr() {
        
        let program = vec!(
            0x4A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.acc = 0x09;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x4);
        assert!(cpu.status.contains(CPUStatus::CARRY));
    }

    /* test for ROL */
    #[test]
    fn test_rol() {
        
        let program = vec!(
            0x2A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.acc = 0x40;
        cpu.status.insert(CPUStatus::CARRY);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x81);
        assert!(!cpu.status.contains(CPUStatus::CARRY));
    }

    /* test for ROR */
    #[test]
    fn test_ror() {
        
        let program = vec!(
            0x6A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.acc = 0x08;
        cpu.status.insert(CPUStatus::CARRY);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x84);
        assert!(!cpu.status.contains(CPUStatus::CARRY));
    }

    /* test for JMP */
    #[test]
    fn test_jmp_absolute() {
        
        let program = vec!(
            0x4C, 0x05, 0x80, 0x69, 0x10, 0x69, 0x20, 0x0
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_jmp_indirect() {
        
        let program = vec!(
            0x6C, 0x00, 0x10, 0x69, 0x10, 0x69, 0x20, 0x0
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.bus.mem_write_u16(0x1000, 0x8005);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }
}