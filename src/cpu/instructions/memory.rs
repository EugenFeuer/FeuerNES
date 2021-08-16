use super::common::*;
use super::super::CPU;
use super::super::AddressMode;

use crate::mem::Memory;


pub fn dec(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    let res = value.wrapping_sub(1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.mem_write(addr, res);
}

pub fn inc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    let res = value.wrapping_add(1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.mem_write(addr, res);
}

pub fn sta(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    cpu.mem_write(addr, cpu.acc);
}

pub fn stx(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    cpu.mem_write(addr, cpu.rx);
}

pub fn sty(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    cpu.mem_write(addr, cpu.ry);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

}