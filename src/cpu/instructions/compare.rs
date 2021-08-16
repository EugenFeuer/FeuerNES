use super::common::*;
use super::super::CPU;
use super::super::AddressMode;

use crate::mem::Memory;

pub fn cmp(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    compare(cpu, cpu.acc, value);
}

pub fn cpx(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    compare(cpu, cpu.rx, value);
}

pub fn cpy(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);
    compare(cpu, cpu.ry, value);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

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
}