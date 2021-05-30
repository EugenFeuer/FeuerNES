#[path = "cpu/cpu.rs"] mod cpu;

fn main() {
    let mut cpu = cpu::CPU::new();
    
    let ram = [0u8; 0x2000];          // RAM [0x0000..0x2000]
    let io = [0u8; 0x2020];           // IO [0x2000..0x4020]
    let special = [0u8; 0x1FE0];      // mappers - special circuitry [0x4020..0x6000]
    let reserved = [0u8; 0x2000];     // reserved to a RAM space [0x6000..0x8000]

    // program [0x8000..0x10000]
    let program: Vec<u8> = vec!(
        0xA9, 0x20, 0x0
    );

    let mut bytes = [0u8; 0xFFFF];
    bytes[0x0000..0x2000].copy_from_slice(&ram[..]);
    bytes[0x2000..0x4020].copy_from_slice(&io[..]);
    bytes[0x4020..0x6000].copy_from_slice(&special[..]);
    bytes[0x6000..0x8000].copy_from_slice(&reserved[..]);
    bytes[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);

    cpu.load(bytes);
    cpu.run();

    println!("Hello, NES!");
}
