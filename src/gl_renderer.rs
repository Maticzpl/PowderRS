use std::borrow::Borrow;
use std::convert::identity;
use std::mem::discriminant;
use glium::*;
use glium::glutin::{ContextBuilder, NotCurrent};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;
use glium::program::ShaderType::Vertex;
use glium::uniforms::{EmptyUniforms, MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior, Uniforms, UniformsStorage};
use glium::vertex::VertexBufferAny;
use glium::vertex::VerticesSource::VertexBuffer;
use crate::sim;
use crate::sim::{Particle, Simulation, WINH, WINW, XRES, XYRES, YRES};
use std::time;
use std::time::Instant;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
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
    pub should_close: bool,
    display: Display,
    square: [Vert; 4],
    square_ind: [u32; 6],
    vert_buffer: vertex::VertexBuffer<Vert>,
    ind_buffer: IndexBuffer<u32>,
    program: Program,
    draw_params: DrawParameters<'a>,
    frame_start: Instant,
    counter: Instant,
    tex_filter: SamplerBehavior,
    camera_matrix: Matrix4<f32>,
}

impl GLRenderer<'_> {
    pub fn new(sim : &Simulation) -> (Self, EventLoop<()>) {
        let win_size = (WINW as u32, WINH as u32);

        let mut event_loop = glutin::event_loop::EventLoop::new();

        let wb = WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(win_size.0, win_size.1))
            .with_title("PowderRS")
            .with_resizable(true);
        let cb = glutin::ContextBuilder::new().with_vsync(false);

        let display = Display::new(wb, cb, &event_loop).unwrap();

        let mut square : [Vert; 4] = [
            Vert{
                pos: [-1 as f32, 1 as f32],
                tex_coords: [0f32, 1f32]
            },
            Vert{
                pos: [1 as f32, 1 as f32],
                tex_coords: [1f32, 1f32]
            },
            Vert
            {
                pos: [1 as f32, -1 as f32],
                tex_coords: [1f32, 0f32]
            },
            Vert
            {
                pos: [-1 as f32, -1 as f32],
                tex_coords: [0f32, 0f32]
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

        let (w,h) = (XRES as f32 / 2.0, YRES as f32 / 2.0);
        let camera_matrix : Matrix4<f32>; //pvm
        let proj_matrix : Matrix4<f32> = cgmath::ortho(-1.0, 1.0, 1.0, -1.0, -1.0, 1.0);
        let view_matrix : Matrix4<f32> = Matrix4::identity(); 
        let model_matrix : Matrix4<f32> = Matrix4::identity();

        camera_matrix = proj_matrix * view_matrix * model_matrix;


        let tex_filter = uniforms::SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..Default::default()
        };



        (Self {
            should_close: false,
            display,
            square,
            square_ind,
            vert_buffer,
            ind_buffer,
            program,
            draw_params,
            frame_start: Instant::now(),
            counter: Instant::now(),
            tex_filter,
            camera_matrix
        }, event_loop)
    }

    pub fn draw(&mut self, sim : &Simulation) {
        let dt = self.frame_start.elapsed().as_micros();
        if dt != 0 && self.counter.elapsed().as_secs() >= 1{
            let fps = 1000000f64 / dt as f64;
            println!("{} - {}", fps, sim.get_part_count());
            self.counter = Instant::now();
        }
        self.frame_start = Instant::now();

        let mut tex_data = vec![vec![(50u8, 0u8, 0u8); XRES]; YRES];

        let mut counter = 0;
        for i in 0..sim.parts.len() {
            if counter >= sim.get_part_count() {
                break;
            }
            let pt = sim.get_part(i).unwrap();
            if pt.p_type != 0 {
                let col = pt.get_type().col;
                tex_data[pt.y as usize][pt.x as usize] = (col[0],col[1],col[2]);
                counter += 1;
            }
        }


        let mut tex : Texture2d = Texture2d::new(&self.display, tex_data).expect("Texture creation failed");
        let uniforms = uniform! {
            tex: glium::uniforms::Sampler(&tex, self.tex_filter),
            pvm: <Matrix4<f32> as Into<[[f32;4];4]>>::into(self.camera_matrix)
        };

        let mut frame = self.display.draw();
        frame.clear_color(0.0,0.0,0.0,0.0);
        frame.draw(&self.vert_buffer, &self.ind_buffer, &self.program, &uniforms, &self.draw_params).expect("Draw error");

        frame.finish().expect("Swap buffers error");
    }
}

const VERT_SHADER : &str = r#"
#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords;
out vec2 v_tex_coords;

uniform mat4 pvm;

void main()
{
    gl_Position = vec4(pos, 0.0, 1.0) * pvm;
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