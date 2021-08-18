use super::super::AddressMode;
use super::super::CPU;
use super::common::*;

use crate::mem::Memory;

pub fn sbc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr) as i8;
    // A = A - M - (1 - C)
    add_to_acc(cpu, (value.wrapping_neg().wrapping_sub(1)) as u8);
}

pub fn dex(cpu: &mut CPU) {
    cpu.rx = cpu.rx.wrapping_sub(1);
    update_zero_flag(cpu, cpu.rx);
    update_neg_flag(cpu, cpu.rx);
}

pub fn dey(cpu: &mut CPU) {
    cpu.ry = cpu.ry.wrapping_sub(1);
    update_zero_flag(cpu, cpu.ry);
    update_neg_flag(cpu, cpu.ry);
}

pub fn inx(cpu: &mut CPU) {
    cpu.rx = cpu.rx.wrapping_add(1);
    update_zero_flag(cpu, cpu.rx);
    update_neg_flag(cpu, cpu.rx);
}

pub fn iny(cpu: &mut CPU) {
    cpu.ry = cpu.ry.wrapping_add(1);
    update_zero_flag(cpu, cpu.ry);
    update_neg_flag(cpu, cpu.ry);
}

pub fn lda(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    cpu.acc = value;
    update_neg_flag(cpu, value);
    update_zero_flag(cpu, value);
}

pub fn ldx(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    cpu.rx = value;
    update_neg_flag(cpu, value);
    update_zero_flag(cpu, value);
}

pub fn ldy(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    cpu.ry = value;
    update_neg_flag(cpu, value);
    update_zero_flag(cpu, value);
}

pub fn tax(cpu: &mut CPU) {
    cpu.rx = cpu.acc;
    update_neg_flag(cpu, cpu.rx);
    update_zero_flag(cpu, cpu.rx);
}

pub fn tay(cpu: &mut CPU) {
    cpu.ry = cpu.acc;
    update_neg_flag(cpu, cpu.ry);
    update_zero_flag(cpu, cpu.ry);
}

pub fn txa(cpu: &mut CPU) {
    cpu.acc = cpu.rx;
    update_neg_flag(cpu, cpu.acc);
    update_zero_flag(cpu, cpu.acc);
}

pub fn tya(cpu: &mut CPU) {
    cpu.acc = cpu.ry;
    update_neg_flag(cpu, cpu.acc);
    update_zero_flag(cpu, cpu.acc);
}

pub fn tsx(cpu: &mut CPU) {
    cpu.rx = cpu.sp;
    update_neg_flag(cpu, cpu.rx);
    update_zero_flag(cpu, cpu.rx);
}

pub fn txs(cpu: &mut CPU) {
    cpu.sp = cpu.rx;
    update_neg_flag(cpu, cpu.sp);
    update_zero_flag(cpu, cpu.sp);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

    /* test for TRANSFER */
    #[test]
    fn test_tax() {
        let program = vec![0x69, 0x10, 0xAA, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_tay() {
        let program = vec![0x69, 0x10, 0xA8, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.ry, 0x10);
    }

    #[test]
    fn test_txa() {
        let program = vec![0x8A, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tya() {
        let program = vec![0x98, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.ry = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tsx() {
        let program = vec![0xBA, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.sp = 0x10;
        cpu.interprect();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_txs() {
        let program = vec![0x9A, 0x00];

        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.sp, 0x10);
    }
}
