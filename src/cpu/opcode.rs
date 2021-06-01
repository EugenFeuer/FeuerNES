use crate::cpu::AddressMode;
use std::collections::HashMap;

pub struct Opcode {
    pub op: u8,
    pub name: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressMode
}

impl Opcode {
    fn new(op: u8, name: &'static str, bytes: u8, cycles: u8, mode: AddressMode) -> Self {
        Opcode {
            op: op,
            name: name,
            bytes: bytes,
            cycles: cycles,
            mode: mode
        }
    }
}

lazy_static! {
    pub static ref OPCODES: Vec<Opcode> = vec!(
        Opcode::new(0x00, "BRK", 1, 7, AddressMode::NoneAddressing),
        Opcode::new(0xAA, "TAX", 1, 2, AddressMode::NoneAddressing),
    
        Opcode::new(0xA9, "LDA", 2, 2, AddressMode::Immediate),
        Opcode::new(0xA5, "LDA", 2, 3, AddressMode::ZeroPage),
        Opcode::new(0xB5, "LDA", 2, 4, AddressMode::ZeroPageX),
        Opcode::new(0xAD, "LDA", 3, 4, AddressMode::Absolute),
        Opcode::new(0xBD, "LDA", 3, 4, AddressMode::AbsoluteX),
        Opcode::new(0xB9, "LDA", 3, 4, AddressMode::AbsoluteY),
        Opcode::new(0xA1, "LDA", 2, 6, AddressMode::IndirectX),
        Opcode::new(0xB1, "LDA", 2, 5, AddressMode::IndirectY),

        Opcode::new(0x85, "STA", 2, 3, AddressMode::ZeroPage),
        Opcode::new(0x95, "STA", 2, 4, AddressMode::ZeroPageX),
        Opcode::new(0x8D, "STA", 3, 4, AddressMode::Absolute),
        Opcode::new(0x9D, "STA", 3, 5, AddressMode::AbsoluteX),
        Opcode::new(0x99, "STA", 3, 5, AddressMode::AbsoluteY),
        Opcode::new(0x81, "STA", 2, 6, AddressMode::IndirectX),
        Opcode::new(0x91, "STA", 2, 6, AddressMode::IndirectX),

        Opcode::new(0x69, "ADC", 2, 2, AddressMode::Immediate),
        Opcode::new(0x65, "ADC", 2, 3, AddressMode::ZeroPage),
        Opcode::new(0x75, "ADC", 2, 4, AddressMode::ZeroPageX),
        Opcode::new(0x6D, "ADC", 3, 4, AddressMode::Absolute),
        Opcode::new(0x7D, "ADC", 3, 4, AddressMode::AbsoluteX),
        Opcode::new(0x79, "ADC", 3, 4, AddressMode::AbsoluteY),
        Opcode::new(0x61, "ADC", 2, 6, AddressMode::IndirectX),
        Opcode::new(0x71, "ADC", 2, 5, AddressMode::IndirectY),

        Opcode::new(0xE9, "SBC", 2, 2, AddressMode::Immediate),
        Opcode::new(0xE5, "SBC", 2, 3, AddressMode::ZeroPage),
        Opcode::new(0xF5, "SBC", 2, 4, AddressMode::ZeroPageX),
        Opcode::new(0xED, "SBC", 3, 4, AddressMode::Absolute),
        Opcode::new(0xFD, "SBC", 3, 4, AddressMode::AbsoluteX),
        Opcode::new(0xF9, "SBC", 3, 4, AddressMode::AbsoluteY),
        Opcode::new(0xE1, "SBC", 2, 6, AddressMode::IndirectX),
        Opcode::new(0xF1, "SBC", 2, 5, AddressMode::IndirectY),
    );

    pub static ref OPCODES_MAP: HashMap<u8, &'static Opcode> = {
        let mut map = HashMap::new();
        for code in &*OPCODES {
            map.insert(code.op, code);
        }
        map
    };
}