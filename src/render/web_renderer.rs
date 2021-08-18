use gloo::render::{request_animation_frame, AnimationFrame};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlShader, WebGlProgram, WebGlBuffer, WebGlUniformLocation, WebGlTexture, WebGlRenderingContext as GL};
use yew::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};

use crate::bus;
use crate::cartridge;
use crate::cpu;
use crate::mem::Memory;
use crate::trace;

use std::mem;

use rand::Rng;

pub enum Message {
    Render(f64)
}

pub struct ScreenBufferData {
    vbo: Option<WebGlBuffer>,
    ibo: Option<WebGlBuffer>,
}

impl ScreenBufferData {
    pub fn new(vbo: Option<WebGlBuffer>,ibo: Option<WebGlBuffer>) -> Self {
        Self {
            vbo: vbo,
            ibo: ibo
        }
    }
}

pub struct ScreenProgramData {
    program: Option<WebGlProgram>,
    vertex_shader: Option<WebGlShader>,
    fragment_shader: Option<WebGlShader>,
    a_position: u32,
    a_texcoord: u32,
    u_time: Option<WebGlUniformLocation>,
    u_screen_tex: Option<WebGlUniformLocation>
}

impl ScreenProgramData {
    pub fn new(program: Option<WebGlProgram>, vertex_shader: Option<WebGlShader>, fragment_shader: Option<WebGlShader>, a_position: u32, a_texcoord: u32, u_time: Option<WebGlUniformLocation>, u_screen_tex: Option<WebGlUniformLocation>) -> Self {
        Self {
            program: program,
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            a_position: a_position,
            a_texcoord: a_texcoord,
            u_time: u_time,
            u_screen_tex: u_screen_tex
        }
    }
}

pub struct Screen{
    cpu: cpu::CPU,
    frame: u32,

    gl: Option<GL>,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
    _render_loop: Option<AnimationFrame>,

    _screen_program: Option<ScreenProgramData>,
    _screen_buffers: Option<ScreenBufferData>,
    _tex: Option<WebGlTexture>,
}

impl Component for Screen {
    type Message = Message;
    type Properties = ();
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            cpu: init_cpu(),
            frame: 0,

            gl: None,
            link: link,
            node_ref: NodeRef::default(),
            _render_loop: None,
            _screen_program: None,
            _screen_buffers: None,
            _tex: None
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        canvas.set_width(320);
        canvas.set_height(320);
        self.gl = Some(canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap());

        self.init();

        if _first_render {
            let handle = {
                let link = self.link.clone();
                request_animation_frame(move |time| link.send_message(Message::Render(time)))
            };
            self._render_loop = Some(handle);
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Render(ts) => {
                self.render_loop(ts);
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <canvas ref={self.node_ref.clone()} />
        }
    }
}

fn byte_to_color(byte: u8) -> (u8, u8, u8, u8) {
    match byte {
        0 => (0, 0, 0, 255),
        1 => (255, 255, 255, 255),
        2 | 9 => (128, 128, 128, 255),
        3 | 10 => (255, 0, 0, 255),
        4 | 11 => (0, 255, 0, 255),
        5 | 12 => (0, 0, 255, 255),
        6 | 13 => (255, 0, 255, 255),
        7 | 14 => (255, 255, 0, 255),
        _ => (0, 255, 255, 255)
    }
}

fn render(cpu: &mut cpu::CPU) -> Vec<u8> {
    let mut frame = vec![0u8; 32 * 32 * 4];
    let mut frame_idx = 0;
    for i in 0x200..0x600 {
        let color_idx = cpu.mem_read(i);

        // use web_sys::console;
        // console::log_1(&format!("color: {}", color_idx).into());

        let (b1, b2, b3, _) = byte_to_color(color_idx);
        frame[frame_idx] = b1;
        frame[frame_idx + 1] = b2;
        frame[frame_idx + 2] = b3;
        frame[frame_idx + 3] = 255;
        frame_idx += 4;
        // console::log_1(&format!("color: {}, {}, {}", b1, b2, b3).into());
    }

    frame
}

fn init_cpu() -> cpu::CPU {
    let bytes = include_bytes!("../../res/snake.nes");
    let cartridge = cartridge::Cartridge::new(&bytes.to_vec()).unwrap();
    let bus = bus::Bus::new(cartridge);
    let cpu = cpu::CPU::new(bus);
    cpu
}

impl Screen {
    pub fn start() {
        yew::start_app::<Screen>();
    }

    pub fn update_texture(&self, width: i32, height: i32, bytes: Vec<u8>) {
        let gl = self.gl.as_ref().expect("get gl context error");
        
        let js_data = js_sys::Uint8Array::from(bytes.as_slice());

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(GL::TEXTURE_2D, 0, GL::RGBA as i32, width, height, 0, GL::RGBA, GL::UNSIGNED_BYTE, Some(js_data.as_ref())).expect("upload texture data error");
    }

    fn init_shader(&self, shader_type: u32, shader_code: &str) -> Option<WebGlShader> {
        let gl = self.gl.as_ref().expect("get gl context error");
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(&shader, shader_code);
        gl.compile_shader(&shader);

        Some(shader)
    }

    fn create_texture(&self, width: i32, height: i32) -> Option<WebGlTexture> {
        let gl = self.gl.as_ref().expect("get gl context error");
        
        let texture = gl.create_texture();
        gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        let mut data: Vec<u8> = vec![0u8; width as usize * height as usize * 4];
        
        for i in 0..width {
            for j in 0..height {
                let index = ((j * height + i) * 4) as usize;
                data[index] = i as u8;
                data[index + 1] = ((i + j) / 2) as u8;
                data[index + 2] = j as u8;
                data[index + 3] = 255;
            }
        }
        self.update_texture(width, height, data);
        gl.bind_texture(GL::TEXTURE_2D, None);

        texture
    }

    fn init(&mut self) {
        let gl = self.gl.as_ref().expect("gl init error");
        self.cpu.reset();

        // VBO
        let vertices: Vec<f32> = vec!(
            // vertex   // uv
            -1.0, -1.0,   0.0, 0.0,
            1.0,  -1.0,   1.0, 0.0,
            1.0,   1.0,   1.0, 1.0,
            -1.0,  1.0,   0.0, 1.0
        );
        let js_vertices = js_sys::Float32Array::from(vertices.as_slice());

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_vertices, GL::STATIC_DRAW);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);

        // IBO
        let indices: Vec<u16> = vec!(
            0, 1, 2,
            2, 3, 0
        );
        let js_indices = js_sys::Uint16Array::from(indices.as_slice());
        let ibo = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &js_indices, GL::STATIC_DRAW);
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);

        self._screen_buffers = Some(ScreenBufferData::new(Some(vbo), Some(ibo)));

        // Shaders
        let vs = self.init_shader(GL::VERTEX_SHADER, include_str!("../../res/screen.vs")).expect("create vs error");
        let fs = self.init_shader(GL::FRAGMENT_SHADER, include_str!("../../res/screen.fs")).expect("create fs error");

        let program = gl.create_program().expect("create program error");
        gl.attach_shader(&program, &vs);
        gl.attach_shader(&program, &fs);
        gl.link_program(&program);

        gl.use_program(Some(&program));

        let a_position = gl.get_attrib_location(&program, "aPosition") as u32;
        let a_texcoord = gl.get_attrib_location(&program, "aTexCoord") as u32;

        let u_time = gl.get_uniform_location(&program, "uTime");
        let u_screen_tex = gl.get_uniform_location(&program, "uScreenTex");

        self._screen_program = Some(ScreenProgramData::new(Some(program), Some(vs), Some(fs), a_position, a_texcoord, u_time, u_screen_tex));

        // Textures
        let texture = self.create_texture(32, 32);
        self._tex = texture;

        gl.use_program(None);
    }

    fn render_loop(&mut self, ts: f64) {
        // use web_sys::console;
        // console::log_1(&format!("ts: {}", ts).into());

        let gl = self.gl.as_ref().expect("gl init error");
        let program = self._screen_program.as_ref().expect("screen program error");
        let buffers = self._screen_buffers.as_ref().expect("screen buffers error");
        
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        gl.use_program(program.program.as_ref());
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, self._tex.as_ref());
        
        gl.uniform1f(program.u_time.as_ref(), ts as f32);
        gl.uniform2i(program.u_time.as_ref(), 320, 320);

        let size_of_f32 = mem::size_of::<f32>() as i32;
        gl.bind_buffer(GL::ARRAY_BUFFER, buffers.vbo.as_ref());

        gl.vertex_attrib_pointer_with_i32(program.a_position, 2, GL::FLOAT, false, 4 * size_of_f32, 0);
        gl.enable_vertex_attrib_array(program.a_position);

        gl.vertex_attrib_pointer_with_i32(program.a_texcoord, 2, GL::FLOAT, false, 4 * size_of_f32, 2 * size_of_f32);
        gl.enable_vertex_attrib_array(program.a_texcoord);
        
        gl.bind_buffer(GL::ARRAY_BUFFER, None);

        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, buffers.ibo.as_ref());
        gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_SHORT, 0);

        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.use_program(None);

        let frame = self.frame;
        // self.cpu.reset();
        for i in 0..240 {
            self.cpu.interprect_with_callback(move |cpu| {
                trace::trace(cpu, &frame);
                let mut rng = rand::thread_rng();
                cpu.bus.mem_write(0x00FE, rng.gen_range(1, 16));
            });
        }
        // use web_sys::console;
        // console::log_1(&format!("frame: {}", frame).into());
        self.frame += 1;

        let bytes = render(&mut self.cpu);
        self.update_texture(32, 32, bytes);

        let handle = {
            let link = self.link.clone();
            request_animation_frame(move |time| link.send_message(Message::Render(time)))
        };

        self._render_loop = Some(handle);
    }    
}
