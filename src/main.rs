mod mem;
mod bus;
mod cpu;
mod opcode;
mod cartridge;

#[macro_use]
extern crate lazy_static;

use rand::Rng;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use mem::Memory;

fn input(cpu: &mut cpu::CPU, event_pump: &mut EventPump) {
    for e in event_pump.poll_iter() {
        match e {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                println!("exit!");
                std::process::exit(0);
            }
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                cpu.bus.mem_write(0x00FF, 0x77);
            }
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                cpu.bus.mem_write(0x00FF, 0x73);
            }
            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                cpu.bus.mem_write(0x00FF, 0x61);
            }
            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                cpu.bus.mem_write(0x00FF, 0x64);
            }
            _ => { }
        }
    }
}

fn byte_to_color(byte: u8) -> Color {
    match byte {
        0 => Color::BLACK,
        1 => Color::WHITE,
        2 | 9 => Color::GRAY,
        3 | 10 => Color::RED,
        4 | 11 => Color::GREEN,
        5 | 12 => Color::BLUE,
        6 | 13 => Color::MAGENTA,
        7 | 14 => Color::YELLOW,
        _ => Color::CYAN
    }
}

fn render(cpu: &cpu::CPU, frame: &mut [u8; 32 * 32 * 3]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x200..0x600 {
        let color_idx = cpu.bus.mem_read(i);
        let (b1, b2, b3) = byte_to_color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }

    update
}

fn main() {
    let path = Path::new("./test.nes");
    
    let mut nes_file = File::open(&path).unwrap();
    let mut bytes = vec![];
    nes_file.read_to_end(&mut bytes);
    let cartridge = cartridge::Cartridge::new(&bytes).unwrap();

    let bus = bus::Bus::new(cartridge);

    let mut cpu = cpu::CPU::new(bus);
    
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();
    let window = video.window("demo", (32 * 10) as u32, (32 * 10) as u32).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = ctx.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let mut screen = [0u8; 32 * 32 * 3];
    let mut rng = rand::thread_rng();

    cpu.reset();
    cpu.interprect_with_callback(move |cpu| {
        input(cpu, &mut event_pump);
        cpu.bus.mem_write(0x00FE, rng.gen_range(1, 16));
        if render(cpu, &mut screen) {
            texture.update(None, &screen, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        ::std::thread::sleep(std::time::Duration::new(0, 70000));
    });

    println!("Hello, NES!");
}
