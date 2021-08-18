mod bus;
mod cartridge;
mod cpu;
mod mem;
mod opcode;
mod ppu;
mod render;
mod trace;

#[macro_use]
extern crate lazy_static;

fn main() {
    render::web_renderer::Screen::start();
}
