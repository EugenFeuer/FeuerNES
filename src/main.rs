mod mem;
mod bus;
mod cpu;
mod opcode;
mod cartridge;
mod trace;
mod web_renderer;

#[macro_use]
extern crate lazy_static;

use rand::Rng;

fn main() {
    web_renderer::Screen::start();

    // let path = Path::new("./test.nes");
    
    // let mut nes_file = File::open(&path).unwrap();
    // let mut bytes = vec![];
    // nes_file.read_to_end(&mut bytes);
    // let cartridge = cartridge::Cartridge::new(&bytes).unwrap();

    // let bus = bus::Bus::new(cartridge);

    // let mut cpu = cpu::CPU::new(bus);

    // let mut rng = rand::thread_rng();
    // let mut frame : u32 = 0;
    // cpu.reset();
    // cpu.interprect_with_callback(move |cpu| {
    //     trace::trace(cpu, &frame);

    //     ::std::thread::sleep(std::time::Duration::new(0, 70000));
    //     frame += 1;
    // });

    println!("Hello, NES!");
}
