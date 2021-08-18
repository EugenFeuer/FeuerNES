use super::super::AddressMode;
use super::super::CPUStatus;
use super::super::CPU;
use super::common::*;

use crate::mem::Memory;

pub fn clc(cpu: &mut CPU) {
    cpu.status.remove(CPUStatus::CARRY);
}

pub fn cld(cpu: &mut CPU) {
    cpu.status.remove(CPUStatus::DECIMAL);
}

pub fn cli(cpu: &mut CPU) {
    cpu.status.remove(CPUStatus::INTERRUPT_DISABLE);
}

pub fn clv(cpu: &mut CPU) {
    cpu.status.remove(CPUStatus::OVERFLOW);
}

pub fn sec(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::CARRY);
}

pub fn sed(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::DECIMAL);
}

pub fn sei(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::INTERRUPT_DISABLE);
}

pub fn bit(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    update_neg_flag(cpu, value);
    update_overflow_flag(cpu, value & 0b0100_0000 == 1);
    update_zero_flag(cpu, cpu.acc & value);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;
}
