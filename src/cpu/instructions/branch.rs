use super::common::*;
use super::super::CPU;
use super::super::CPUStatus;

pub fn bcc(cpu: &mut CPU) {
    branch(cpu, !cpu.status.contains(CPUStatus::CARRY));
}

pub fn bcs(cpu: &mut CPU) {
    branch(cpu, cpu.status.contains(CPUStatus::CARRY));
}

pub fn beq(cpu: &mut CPU) {
    branch(cpu, cpu.status.contains(CPUStatus::ZERO));
}

pub fn bmi(cpu: &mut CPU) {
    branch(cpu, cpu.status.contains(CPUStatus::NEGATIVE));
}

pub fn bne(cpu: &mut CPU) {
    branch(cpu, !cpu.status.contains(CPUStatus::ZERO));
}

pub fn bpl(cpu: &mut CPU) {
    branch(cpu, !cpu.status.contains(CPUStatus::NEGATIVE));
}

pub fn bvc(cpu: &mut CPU) {
    branch(cpu, !cpu.status.contains(CPUStatus::OVERFLOW));
}

pub fn bvs(cpu: &mut CPU) {
    branch(cpu, cpu.status.contains(CPUStatus::OVERFLOW));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

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
}