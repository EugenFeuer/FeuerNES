use super::super::CPUStatus;
use super::super::CPU;
use crate::mem::Memory;

pub const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

pub const STACK_BOTTOM_LOC: u16 = 0x0100;
pub const STACK_RESET_LOC: u8 = 0xFD;

/* status */
pub fn update_zero_flag(cpu: &mut CPU, flag: u8) {
    if flag == 0 {
        cpu.status.insert(CPUStatus::ZERO);
    } else {
        cpu.status.remove(CPUStatus::ZERO);
    }
}

pub fn update_neg_flag(cpu: &mut CPU, flag: u8) {
    if flag & 0b1000_0000 != 0 {
        cpu.status.insert(CPUStatus::NEGATIVE);
    } else {
        cpu.status.remove(CPUStatus::NEGATIVE);
    }
}

pub fn update_overflow_flag(cpu: &mut CPU, flag: bool) {
    if flag {
        cpu.status.insert(CPUStatus::OVERFLOW);
    } else {
        cpu.status.remove(CPUStatus::OVERFLOW);
    }
}

pub fn update_carry_flag(cpu: &mut CPU, flag: bool) {
    if flag {
        cpu.status.insert(CPUStatus::CARRY);
    } else {
        cpu.status.remove(CPUStatus::CARRY);
    }
}

/* stack */
pub fn stack_push(cpu: &mut CPU, value: u8) {
    cpu.mem_write(cpu.sp as u16 + STACK_BOTTOM_LOC, value);
    cpu.sp = cpu.sp.wrapping_sub(1);
}

pub fn stack_pop(cpu: &mut CPU) -> u8 {
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.mem_read(cpu.sp as u16 + STACK_BOTTOM_LOC)
}

pub fn stack_push_u16(cpu: &mut CPU, value: u16) {
    stack_push(cpu, (value >> 8) as u8); // hi
    stack_push(cpu, value as u8); // lo
}

pub fn stack_pop_u16(cpu: &mut CPU) -> u16 {
    let lo = stack_pop(cpu) as u16;
    let hi = stack_pop(cpu) as u16;

    hi << 8 | lo
}

/* compare */
pub fn compare(cpu: &mut CPU, v1: u8, v2: u8) {
    update_carry_flag(cpu, v1 >= v2);
    let res = v1.wrapping_sub(v2);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
}

/* branch */
pub fn branch(cpu: &mut CPU, flag: bool) {
    if flag {
        let offset = cpu.mem_read(cpu.pc) as i8; // offset can be negative
        let dst = cpu.pc.wrapping_add(1).wrapping_add(offset as u16);
        cpu.pc = dst;
    }
}

/* register */
pub fn add_to_acc(cpu: &mut CPU, data: u8) {
    let cur_carry: u16 = if cpu.status.contains(CPUStatus::CARRY) {
        1
    } else {
        0
    };

    // A = A + M + C
    let sum = cpu.acc as u16 + data as u16 + cur_carry;

    // update flags
    update_carry_flag(cpu, sum > 0xFF);

    let res = sum as u8;
    // (M ^ result) & (N ^ result) & 0x80 != 0
    update_overflow_flag(cpu, (data ^ res) & (cpu.acc ^ res) & 0x80 != 0);

    cpu.acc = res;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::CPUStatus;
    use crate::cpu::With;
}
