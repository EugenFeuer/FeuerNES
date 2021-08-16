use super::CPU;
use super::AddressMode;
use super::CPUStatus;
use super::With;

use crate::mem::Memory;

pub const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

pub const STACK_BOTTOM_LOC: u16 = 0x0100;
pub const STACK_RESET_LOC: u8 = 0xFD;

fn update_zero_flag(cpu: &mut CPU, flag: u8) {
    if flag == 0 {
        cpu.status.insert(CPUStatus::ZERO);
    } else {
        cpu.status.remove(CPUStatus::ZERO);
    }
}

fn update_neg_flag(cpu: &mut CPU, flag: u8) {
    if flag & 0b1000_0000 != 0 {
        cpu.status.insert(CPUStatus::NEGATIVE);
    } else {
        cpu.status.remove(CPUStatus::NEGATIVE);
    }
}

fn update_overflow_flag(cpu: &mut CPU, flag: bool) {
    if flag {
        cpu.status.insert(CPUStatus::OVERFLOW);
    } else {
        cpu.status.remove(CPUStatus::OVERFLOW);
    }
}

fn update_carry_flag(cpu: &mut CPU, flag: bool) {
    if flag {
        cpu.status.insert(CPUStatus::CARRY);
    } else {
        cpu.status.remove(CPUStatus::CARRY);
    }
}

fn add_to_acc(cpu: &mut CPU, data: u8) {
    let cur_carry: u16 = if cpu.status.contains(CPUStatus::CARRY) {
        1
    } else {
        0
    };

    // A = A + M + C
    let sum = cpu.acc as u16 +
                data     as u16 +
                cur_carry;

    // update flags
    update_carry_flag(cpu, sum > 0xFF);

    let res = sum as u8;
    // (M ^ result) & (N ^ result) & 0x80 != 0
    update_overflow_flag(cpu, (data ^ res) & (cpu.acc ^ res) & 0x80 != 0);

    cpu.acc = res;
}

pub fn adc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    add_to_acc(cpu, cpu.mem_read(addr));
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

fn branch(cpu: &mut CPU, flag: bool) {
    if flag {
        let offset = cpu.mem_read(cpu.pc) as i8;  // offset can be negative
        let dst = cpu.pc.wrapping_add(1).wrapping_add(offset as u16);
        cpu.pc = dst;
    }
}

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

pub fn sbc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr) as i8;
    // A = A - M - (1 - C)
    add_to_acc(cpu, (value.wrapping_neg().wrapping_sub(1)) as u8);
}

pub fn bit(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    update_neg_flag(cpu, value);
    update_overflow_flag(cpu, value & 0b0100_0000 == 1);
    update_zero_flag(cpu, cpu.acc & value);
}

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

fn compare(cpu: &mut CPU, v1: u8, v2: u8) {
    update_carry_flag(cpu, v1 >= v2);
    let res = v1.wrapping_sub(v2);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);
}

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

pub fn dec(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    let res = value.wrapping_sub(1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.mem_write(addr, res);
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

pub fn inc(cpu: &mut CPU, mode: &AddressMode) {
    let addr = cpu.get_operand_address(mode);
    let value = cpu.mem_read(addr);

    let res = value.wrapping_add(1);
    update_zero_flag(cpu, res);
    update_neg_flag(cpu, res);

    cpu.mem_write(addr, res);
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

fn stack_push(cpu: &mut CPU, value: u8) {
    cpu.mem_write(cpu.sp as u16 + STACK_BOTTOM_LOC, value);
    cpu.sp = cpu.sp.wrapping_sub(1);
}

fn stack_pop(cpu: &mut CPU) -> u8 {
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.mem_read(cpu.sp as u16 + STACK_BOTTOM_LOC)
}

fn stack_push_u16(cpu: &mut CPU, value: u16) {
    stack_push(cpu, (value >> 8) as u8);    // hi
    stack_push(cpu, value as u8);           // lo
}

fn stack_pop_u16(cpu: &mut CPU) -> u16 {
    let lo = stack_pop(cpu) as u16;
    let hi = stack_pop(cpu) as u16;

    hi << 8 | lo
}

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

pub fn sec(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::CARRY);
}

pub fn sed(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::DECIMAL);   
}

pub fn sei(cpu: &mut CPU) {
    cpu.status.insert(CPUStatus::INTERRUPT_DISABLE);
}

pub fn brk(cpu: &mut CPU) {
    stack_push_u16(cpu, cpu.pc);
    stack_push(cpu, cpu.status.bits());
    cpu.pc = cpu.mem_read_u16(RESET_INTERRUPT_MEM_LOC);
    cpu.status.insert(CPUStatus::BREAK);
}



#[cfg(test)]
mod test {
    use super::*;

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

    /* test for TRANSFER */
    #[test]
    fn test_tax() {
        
        let program = vec!(
            0x69, 0x10, 0xAA, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_tay() {
        
        let program = vec!(
            0x69, 0x10, 0xA8, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.run();

        assert_eq!(cpu.ry, 0x10);
    }

    #[test]
    fn test_txa() {
        
        let program = vec!(
            0x8A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tya() {
        
        let program = vec!(
            0x98, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.ry = 0x10;
        cpu.interprect();

        assert_eq!(cpu.acc, 0x10);
    }

    #[test]
    fn test_tsx() {
        
        let program = vec!(
            0xBA, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.sp = 0x10;
        cpu.interprect();

        assert_eq!(cpu.rx, 0x10);
    }

    #[test]
    fn test_txs() {
        
        let program = vec!(
            0x9A, 0x00
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.rx = 0x10;
        cpu.interprect();

        assert_eq!(cpu.sp, 0x10);
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

    /* test for JMP */
    #[test]
    fn test_jmp_absolute() {
        
        let program = vec!(
            0x4C, 0x05, 0x80, 0x69, 0x10, 0x69, 0x20, 0x0
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }

    #[test]
    fn test_jmp_indirect() {
        
        let program = vec!(
            0x6C, 0x00, 0x10, 0x69, 0x10, 0x69, 0x20, 0x0
        );
        
        let mut cpu = CPU::with(program.to_vec());
        cpu.reset();
        cpu.mem_write_u16(0x1000, 0x8005);
        cpu.interprect();

        assert_eq!(cpu.acc, 0x20);
    }
}