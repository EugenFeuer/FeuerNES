use crate::cpu;
use crate::opcode;
use crate::mem::Memory;
use crate::cpu::AddressMode;

use std::collections::HashMap;

pub fn trace(cpu: &cpu::CPU, frame: &u32) {
    println!("========== FRAME: {} ==========", frame);

    let _pc = cpu.pc;

    let ref opcodes: HashMap<u8, &'static opcode::Opcode> = *opcode::OPCODES_MAP;
    let op = cpu.mem_read(_pc);

    let instruction = opcodes.get(&op).expect(&format!("op: {:x} not exists or not impl.", op));

    let (addr, value) = match instruction.mode {
        AddressMode::Immediate | AddressMode::NoneAddressing => { (0, 0) }
        _ => {
            let _addr = cpu.get_absolute_address(&instruction.mode, _pc + 1);
            let _value = cpu.mem_read(_addr);
            (_addr, _value)
        }
    };

    println!("PC: {:#02X}, INSTRUCT: {}", _pc, instruction.name);
    println!("SP: {:#02X}, VALUE: {:#02X}", cpu.sp, cpu.mem_read(cpu.sp as u16 + 0x100));
    println!("ACC: {:#02X}", cpu.acc);
    println!("RX: {:#02X}", cpu.rx);
    println!("RY: {:#02X}", cpu.ry);
    println!("STATUS: {:?}", cpu.status);
    println!("");

    println!("mode: {:?}, addr: {:#02X}, value: {:#02X}", instruction.mode, addr, value);

    match instruction.mode {
        AddressMode::Immediate => {

        }
        ZeroPage => {

        }
        ZeroPageX => {

        }
        ZeroPageY => {

        }
        Absolute => {

        }
        AbsoluteX => {

        }
        AbsoluteY => {

        }
        IndirectX => {

        }
        IndirectY => {

        }
        NoneAddressing => {
            
        }
    }
}