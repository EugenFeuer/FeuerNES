use crate::cpu;
use crate::cpu::AddressMode;
use crate::mem::Memory;
use crate::opcode;

use std::collections::HashMap;

pub struct TraceInfo {
    frame: u32,
    pc: u16,
    opcode: opcode::Opcode,
    sp: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    status: cpu::CPUStatus,
}

impl TraceInfo {
    pub fn new(frame: u32, cpu: &cpu::CPU) -> Self {
        let ref opcodes: HashMap<u8, &'static opcode::Opcode> = *opcode::OPCODES_MAP;
        let op = cpu.mem_read(cpu.pc);
        let opcode = opcodes
            .get(&op)
            .expect(&format!("op: {:x} not exists or not impl.", op));
        TraceInfo {
            frame: frame,
            pc: cpu.pc,
            opcode: **opcode,
            sp: cpu.sp,
            acc: cpu.acc,
            rx: cpu.rx,
            ry: cpu.ry,
            status: cpu.status,
        }
    }

    pub fn dump(self: Self) -> String {
        format!(
            "{} {} {} {} {} {} {} {:o}",
            self.frame, self.pc, self.opcode.name, self.sp, self.acc, self.rx, self.ry, self.status
        )
    }
}

pub fn trace(cpu: &cpu::CPU, frame: &u32) {
    println!("========== FRAME: {} ==========", frame);

    let _pc = cpu.pc;

    let trace_info = TraceInfo::new(*frame, cpu);
    let instruction = trace_info.opcode;

    let (addr, value) = match instruction.mode {
        AddressMode::Immediate | AddressMode::NoneAddressing => (0, 0),
        _ => {
            let _addr = cpu.get_absolute_address(&instruction.mode, _pc + 1);
            let _value = cpu.mem_read(_addr);
            (_addr, _value)
        }
    };

    // println!("{}", trace_info.dump());

    println!("PC: {:#02X}, INSTRUCT: {}", _pc, instruction.name);
    println!(
        "SP: {:#02X}, VALUE: {:#02X}",
        cpu.sp,
        cpu.mem_read(cpu.sp as u16 + 0x100)
    );
    println!("ACC: {:#02X}", cpu.acc);
    println!("RX: {:#02X}", cpu.rx);
    println!("RY: {:#02X}", cpu.ry);
    println!("STATUS: {:?}", cpu.status);
    println!("");

    println!(
        "mode: {:?}, addr: {:#02X}, value: {:#02X}",
        instruction.mode, addr, value
    );

    match instruction.mode {
        AddressMode::Immediate => {}
        ZeroPage => {}
        ZeroPageX => {}
        ZeroPageY => {}
        Absolute => {}
        AbsoluteX => {}
        AbsoluteY => {}
        IndirectX => {}
        IndirectY => {}
        NoneAddressing => {}
    }
}
