#[path = "cpu/cpu.rs"] mod cpu;

#[macro_use]
extern crate lazy_static;

fn main() {
    let mut cpu = cpu::CPU::new();
    
    let program: Vec<u8> = vec!(
        0x69, 0xD0, 0x69, 0x90, 0x00
    );

    cpu.load_program(program);
    cpu.run();

    println!("Hello, NES!");
}
