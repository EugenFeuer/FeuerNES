// TODO: jmp

#[cfg(test)]
mod test {
    use super::*;

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