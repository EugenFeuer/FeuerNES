use super::super::CPUStatus;
use super::super::CPU;
use super::common::*;

pub fn php(cpu: &mut CPU) {
    let mut s = cpu.status.clone();
    // http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    s.insert(CPUStatus::BREAK);
    s.insert(CPUStatus::UNUSED);
    stack_push(cpu, s.bits());
}

pub fn plp(cpu: &mut CPU) {
    let s = stack_pop(cpu);
    cpu.status.bits = s;
    cpu.status.remove(CPUStatus::BREAK);
}

pub fn pha(cpu: &mut CPU) {
    stack_push(cpu, cpu.acc);
}

pub fn pla(cpu: &mut CPU) {
    cpu.acc = stack_pop(cpu);
    update_neg_flag(cpu, cpu.acc);
    update_zero_flag(cpu, cpu.acc);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;
}
