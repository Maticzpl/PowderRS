use std::borrow::Borrow;
use std::mem::discriminant;
use glium::*;
use glium::glutin::{ContextBuilder, NotCurrent};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;
use glium::program::ShaderType::Vertex;
use glium::uniforms::{EmptyUniforms, Uniforms, UniformsStorage};
use glium::vertex::VertexBufferAny;
use glium::vertex::VerticesSource::VertexBuffer;
use crate::sim;
use crate::sim::{Particle, Simulation, XRES, XYRES, YRES};
use std::time;
use std::time::Instant;
use glium::buffer::{Buffer, BufferMode, BufferType};
use glium::buffer::BufferType::UniformBuffer;
use glium::texture::{Dimensions, MipmapsOption, Texture2dDataSource, UncompressedFloatFormat};

#[derive(Copy, Clone)]
struct Vert {
    pos: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vert, pos, tex_coords);

pub struct GLRenderer<'a> {
    win_size: (i32, i32),
    event_loop: EventLoop<()>,
    display: Display,
    square: [Vert; 4],
    square_ind: [u32; 6],
    vert_buffer : vertex::VertexBuffer<Vert>,
    ind_buffer : IndexBuffer<u32>,
    program: Program,
    draw_params: DrawParameters<'a>,
    frame_start: Instant,
    counter: Instant,
    //tex_data : Vec<Vec<u8>>
}

impl GLRenderer<'_> {
    pub fn new(sim : &Simulation) -> Self {
        let win_size = (800, 600);

        let mut event_loop = glutin::event_loop::EventLoop::new();

        let wb = WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(win_size.0, win_size.1))
            .with_title("PowderRS")
            .with_resizable(true);
        let cb = glutin::ContextBuilder::new();//.with_vsync(true);

        let display = Display::new(wb, cb, &event_loop).unwrap();
        let square : [Vert; 4] = [
            Vert{
                pos: [-1f32, -1f32],
                tex_coords: [0f32, 0f32]
            },
            Vert{
                pos: [ 1f32, -1f32],
                tex_coords: [1f32, 0f32]
            },
            Vert
            {
                pos: [ 1f32,  1f32],
                tex_coords: [1f32, 1f32]
            },
            Vert
            {
                pos: [-1f32,  1f32],
                tex_coords: [0f32, 1f32]
            }
        ];
        let square_ind : [u32; 6] = [
            0, 1, 2, 0, 2, 3
        ];

        let ind_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &square_ind).expect("Can't create index buffer");
        let vert_buffer = vertex::VertexBuffer::new(&display, &square).expect("Can't create vert buffer");

        let shaders : ProgramCreationInput = ProgramCreationInput::SourceCode {
            vertex_shader: VERT_SHADER,
            fragment_shader: FRAG_SHADER,

            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: false,
            uses_point_size: false,
        };

        let program = Program::new(&display, shaders).expect("Shader stuff failed");

        let mut draw_params : DrawParameters = DrawParameters::default();

        Self {
            win_size,
            event_loop,
            display,
            square,
            square_ind,
            vert_buffer,
            ind_buffer,
            program,
            draw_params,
            frame_start: Instant::now(),
            counter: Instant::now(),
            //tex_data
        }
    }

    pub fn draw(&mut self, sim : &Simulation) {
        let dt = self.frame_start.elapsed().as_micros();
        if dt != 0 && self.counter.elapsed().as_secs() >= 1{
            let fps = 1000000 / dt;
            println!("{}", fps);
            self.counter = Instant::now();
        }


        let mut tex_data = vec![vec![(0u8, 0u8, 0u8); XRES]; YRES];

        for i in 0..sim.pmap.len() {
            let pt = sim.get_part(i).unwrap();
            let col = pt.get_type().col;
            tex_data[i/XRES][i%XRES] = (col[0],col[1],col[2]);
        }

        let mut tex : Texture2d = Texture2d::new(&self.display, tex_data).expect("Texture creation failed");
        let uniforms = uniform! {
            tex: &tex
        };

        self.frame_start = Instant::now();
        let (win_w, win_h) = self.win_size;
        let mut frame = self.display.draw();

        frame.draw(&self.vert_buffer, &self.ind_buffer, &self.program, &uniforms, &self.draw_params).expect("Draw error");

        frame.finish().expect("Swap buffers error");
    }

    pub fn should_close(&self) -> bool {
        return false;
    }

    pub fn get_window_size(&self) -> (i32, i32) {
        return self.win_size;
    }

    pub fn close(&self) {}
}

const VERT_SHADER : &str = r#"
#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords;
out vec2 v_tex_coords;

void main()
{
    gl_Position = vec4(pos, 0.0, 1.0);
    v_tex_coords = tex_coords;
}
"#;

const FRAG_SHADER : &str = r#"
#version 330 core
out vec4 FragColor;

in vec2 v_tex_coords;
uniform sampler2D tex;

void main()
{
    FragColor = texture(tex, v_tex_coords);
}
"#;