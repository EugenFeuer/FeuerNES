mod opcode;

use std::collections::HashMap;

const PROGRAM_BEGIN_LOC: u16 = 0x8000;
const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

const MEMORY_CAP: usize = 0xFFFF;

const STACK_BOTTOM_LOC: u16 = 0x0100;
const STACK_RESET_LOC: u8 = 0xFD;

#[derive(Debug)]
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
    pc: u16,
    sp: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    status: CPUStatus,
    mem: Memory
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            sp: STACK_RESET_LOC,
            acc: 0,
            rx: 0,
            ry: 0,
            status: CPUStatus::from_bits_truncate(CPUStatus::UNUSED.bits()),
            mem: Memory::new()
        }
    }

    pub fn reset(&mut self) {
        self.acc = 0;
        self.rx = 0;
        self.ry = 0;
        self.status = CPUStatus::from_bits_truncate(CPUStatus::UNUSED.bits());

        self.pc = self.mem.read_u16(RESET_INTERRUPT_MEM_LOC);
        self.sp = STACK_RESET_LOC;
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
        self.add_to_acc(self.mem.read(addr));
    }

    fn and(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let res = self.acc & self.mem.read(addr);
        self.update_zero_flag(res);
        self.update_neg_flag(res);
        self.acc = res;
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
        let mut value = self.mem.read(addr);

        self.update_carry_flag(value >> 7 == 1);

        value <<= 1;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
        
        self.mem.write(addr, value);
    }

    fn branch(&mut self, flag: bool) {
        if flag {
            let offset = self.mem.read(self.pc) as u16;
            let dst = self.pc.wrapping_add(1).wrapping_add(offset);
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
        let value = self.mem.read(addr) as i8;
        // A = A - M - (1 - C)
        self.add_to_acc((-value - 1) as u8);
    }

    fn bit(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);

        self.update_neg_flag(value);
        self.update_overflow_flag(value >> 6 == 1);
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
        self.update_zero_flag(v1 - v2);
        self.update_neg_flag(v1 - v2);
    }

    fn cmp(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);
        self.compare(self.acc, value);
    }

    fn cpx(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);
        self.compare(self.rx, value);
    }

    fn cpy(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);
        self.compare(self.ry, value);
    }

    fn stack_push(&mut self, value: u8) {
        self.mem.write(self.sp as u16 + STACK_BOTTOM_LOC, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.mem.read(self.sp as u16 + STACK_BOTTOM_LOC)
    }

    fn php(&mut self) {
        let mut s = self.status.clone();
        s.insert(CPUStatus::BREAK);
        self.stack_push(s.bits());
    }

    fn plp(&mut self) {
        let s = self.stack_pop();
        self.status.bits = s;
        self.status.remove(CPUStatus::BREAK);
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

    pub fn load_program(&mut self, program: Vec<u8>) {
        let ram = [0u8; 0x2000];          // RAM [0x0000..0x2000]
        let io = [0u8; 0x2020];           // IO [0x2000..0x4020]
        let special = [0u8; 0x1FE0];      // mappers - special circuitry [0x4020..0x6000]
        let reserved = [0u8; 0x2000];     // reserved to a RAM space [0x6000..0x8000]
    
        // program [0x8000..0x10000]
    
        let mut bytes = [0u8; 0xFFFF];
        bytes[0x0000..0x2000].copy_from_slice(&ram[..]);
        bytes[0x2000..0x4020].copy_from_slice(&io[..]);
        bytes[0x4020..0x6000].copy_from_slice(&special[..]);
        bytes[0x6000..0x8000].copy_from_slice(&reserved[..]);
        bytes[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        bytes[0x8000..(0x8000 + program.len())].copy_from_slice(&program);
        
        self.load(bytes);
    }

    pub fn run(&mut self) {
        self.reset();
        self.interprect();
    }

    pub fn interprect(&mut self) -> () {
        let ref opcodes: HashMap<u8, &'static opcode::Opcode> = *opcode::OPCODES_MAP;
        loop {
            let op = self.mem.read(self.pc);
            self.pc += 1;
            let pc_state = self.pc;

            let code = opcodes.get(&op).expect(&format!("op: {:x} not exists or not impl.", op));

            match op {
                0x00 => {
                    return;
                }
                0xAA => {
                    self.tax();
                }
                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&code.mode);
                }
                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&code.mode);
                }
                // ADC
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&code.mode);
                }
                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&code.mode);
                }
                // ASL
                0x0A => {
                    self.asl_acc();
                }
                0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&code.mode);
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
                // PHP
                0x08 => {
                    self.php();
                }
                // PLP
                0x28 => {
                    self.plp();
                }
                _ => {

                }
            }

            if pc_state == self.pc {
                self.pc += (code.bytes - 1) as u16
            }
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    /* test for ADC */
    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
        let program = vec!(
            0x69, 0x10, 0x69, 0x20, 0x00
        );
        
        cpu.load_program(program);
        cpu.run();

        assert_eq!(cpu.acc, 0x30);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new();
        let program = vec!(
            0x69, 0xD0, 0x69, 0x90, 0x00
        );
        
        cpu.load_program(program);
        cpu.run();
        
        assert!(cpu.status.contains(CPUStatus::OVERFLOW));
    }

    /* test for SBC */
    #[test]
    fn test_sbc() {
        let mut cpu = CPU::new();
        let program = vec!(
            0x69, 0x10, 0xE9, 0x01, 0x00
        );
        
        cpu.load_program(program);
        cpu.run();
        
        assert_eq!(cpu.acc, 0x0E);
    }

        /* test for AND */
        #[test]
        fn test_and() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x69, 0x0F, 0x29, 0x11, 0x00
            );
            
            cpu.load_program(program);
            cpu.run();
            
            assert_eq!(cpu.acc, 0x01);
        }

        /* test for ASL */
        #[test]
        fn test_asl() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x06, 0xFF, 0x00
            );
            
            cpu.load_program(program);
            cpu.mem.write(0x00FF, 0x10);
            cpu.run();
            
            assert_eq!(cpu.mem.read(0x00FF), 0x20);
        }

        #[test]
        fn test_asl_acc() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x69, 0x10, 0x0A, 0x00
            );
            
            cpu.load_program(program);
            cpu.run();
            
            assert_eq!(cpu.acc, 0x20);
        }

        /* test for BRANCH */
        #[test]
        fn test_bcc() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x90, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.remove(CPUStatus::CARRY);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bcs() {
            let mut cpu = CPU::new();
            let program = vec!(
                0xB0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.insert(CPUStatus::CARRY);
            cpu.interprect();
            
            assert_eq!(cpu.acc, 0x21);  // because the CARRY bit has been set
        }

        #[test]
        fn test_beq() {
            let mut cpu = CPU::new();
            let program = vec!(
                0xF0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.insert(CPUStatus::ZERO);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bmi() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x30, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.insert(CPUStatus::NEGATIVE);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bne() {
            let mut cpu = CPU::new();
            let program = vec!(
                0xD0, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.remove(CPUStatus::ZERO);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bpl() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x10, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.remove(CPUStatus::NEGATIVE);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bvc() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x50, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.remove(CPUStatus::OVERFLOW);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        #[test]
        fn test_bvs() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x70, 0x03, 0x69, 0x10, 0x00, 0x69, 0x20
            );
            
            cpu.load_program(program);
            cpu.reset();
            cpu.status.insert(CPUStatus::OVERFLOW);
            cpu.interprect();

            assert_eq!(cpu.acc, 0x20);
        }

        /* test for COMPARE */
        #[test]
        fn test_cmp1() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x69, 0x10, 0xC9, 0x0F, 0x00
            );
            
            cpu.load_program(program);
            cpu.run();

            assert!(cpu.status.contains(CPUStatus::CARRY));
        }

        #[test]
        fn test_cmp2() {
            let mut cpu = CPU::new();
            let program = vec!(
                0x69, 0x10, 0xC9, 0x10, 0x00
            );
            
            cpu.load_program(program);
            cpu.run();

            assert!(cpu.status.contains(CPUStatus::CARRY));
            assert!(cpu.status.contains(CPUStatus::ZERO));
        }
}