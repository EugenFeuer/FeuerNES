mod mem;
mod bus;
mod cpu;
mod opcode;
mod cartridge;
mod trace;
mod render;

#[macro_use]
extern crate lazy_static;

fn main() {
    render::web_renderer::Screen::start();
}
