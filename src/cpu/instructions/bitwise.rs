use super::common::*;
use super::super::CPU;
use super::super::AddressMode;

use crate::mem::Memory;

pub fn adc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let data = cpu.mem_read(addr);
    add_to_acc(cpu, data);
}

pub fn and(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let res = cpu.acc & cpu.mem_read(addr);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.acc = res;
}

pub fn ora(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let res = cpu.acc | cpu.mem_read(addr);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.acc = res;
}

pub fn eor(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let res = cpu.acc ^ cpu.mem_read(addr);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.acc = res;
}

pub fn rol_acc(cpu: &mut CPU) {
    let res = (cpu.acc << 1) | (0x01 & cpu.status.bits());

    update_carry_flag(cpu, cpu.acc >> 7 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.acc = res;
}

pub fn rol(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    let res = (value << 1) | (0x01 & cpu.status.bits());
    
    update_carry_flag(cpu, value >> 7 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.mem_write(addr, res);
}

pub fn ror_acc(cpu: &mut CPU) {
    let res = (cpu.acc >> 1) | (cpu.status.bits() << 7);

    update_carry_flag(cpu, cpu.acc & 0x01 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.acc = res;
}

pub fn ror(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    let res = (value >> 1) | (cpu.status.bits() << 7);

    update_carry_flag(cpu, value & 0x01 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.acc = res;
}

pub fn lsr_acc(cpu: &mut CPU) {
    let res = cpu.acc >> 1;

    update_carry_flag(cpu, cpu.acc & 0x1 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.acc = res;
}

pub fn lsr(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    let res = value >> 1;
    
    update_carry_flag(cpu, value & 0x01 == 1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
    cpu.mem_write(addr, res);
}

pub fn asl_acc(cpu: &mut CPU) {
    let mut res = cpu.acc;

    update_carry_flag(cpu, res >> 7 == 1);

    res <<= 1;
    update_neg_flag(cpu, res);
    update_zero_flag(cpu, res);
    cpu.acc = res;
}

pub fn asl(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let mut value = cpu.mem_read(addr);

    update_carry_flag(cpu, value >> 7 == 1);

    value <<= 1;
    update_neg_flag(cpu, value);
    update_zero_flag(cpu, value);
    
    cpu.mem_write(addr, value);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

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
        cpu.mem_write(0x00FF, 0x10);
        cpu.run();
        
        assert_eq!(cpu.mem_read(0x00FF), 0x20);
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
}