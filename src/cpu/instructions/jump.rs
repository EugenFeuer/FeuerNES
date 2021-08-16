use super::common::*;
use super::super::CPU;
use super::super::CPUStatus;
use super::super::AddressMode;

use crate::mem::Memory;

pub fn jsr(cpu: &mut CPU, mode: &AddressMode) {
    stack_push_u16(cpu, cpu.pc + 1);   // PC + 2 - 1
    let addr = cpu.get_operand_address(mode);
    cpu.pc = addr;
}

pub fn rts(cpu: &mut CPU) {
    cpu.pc = stack_pop_u16(cpu) + 1;
}

pub fn rti(cpu: &mut CPU) {
    cpu.status.bits = stack_pop(cpu);
    cpu.status.remove(CPUStatus::BREAK);
    cpu.pc = stack_pop_u16(cpu);
}

pub fn brk(cpu: &mut CPU) {
    stack_push_u16(cpu, cpu.pc);
    stack_push(cpu, cpu.status.bits());
    cpu.pc = cpu.mem_read_u16(RESET_INTERRUPT_MEM_LOC);
    cpu.status.insert(CPUStatus::BREAK);
}

// TODO: jmp

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;

}