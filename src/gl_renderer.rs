use glium::*;

use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;

use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};



use crate::sim::{Simulation, UI_MARGIN, WINH, WINW, XRES, YRES};

use std::time::{Instant};
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use glium::glutin::GlProfile;
use glium::texture::{MipmapsOption, UncompressedFloatFormat};
use glium_glyph::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::gui::GUI;

#[derive(Copy, Clone)]
pub struct Vert {
    pub pos: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vert, pos, tex_coords);

pub struct GLRenderer<'a> {
    pub camera_zoom: f32,
    pub camera_pan: Vector2<f32>,
    pub gui : GUI<'a, 'a>,
    pub grid_size : u32,
    pub cursor: Rect,
    display: Display,
    vert_buffer: VertexBuffer<Vert>,
    ind_buffer: IndexBuffer<u32>,
    program: Program,
    draw_params: DrawParameters<'a>,
    tex_filter: SamplerBehavior,
    texture: Texture2d,

    frame_start: Instant,
    timers: [Instant; 4],
    perf_sum: [u128; 3],
    fps_sum: f64,
    samples: u32,

    proj_matrix: Matrix4<f32>,
    pub view_matrix: Matrix4<f32>,
    pub model_matrix: Matrix4<f32>,
}

impl GLRenderer<'_> {
    pub fn new() -> (Self, EventLoop<()>) {
        let win_size = (WINW as u32, WINH as u32);

        let event_loop = glutin::event_loop::EventLoop::new();

        let wb = WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(win_size.0, win_size.1))
            .with_title("PowderRS")
            .with_resizable(true);
            //.with_transparent(true);
        let cb = glutin::ContextBuilder::new()
            .with_vsync(false)
            .with_gl_profile(GlProfile::Core);

        let display = Display::new(wb, cb, &event_loop).unwrap();

        let (w, h) = (WINW as f32 / 2.0, WINH as f32 / 2.0);

        let square : [Vert; 4] = [
            Vert{
                pos: [-w as f32, h as f32],
                tex_coords: [0f32, 1f32]
            },
            Vert{
                pos: [w as f32, h as f32],
                tex_coords: [1f32, 1f32]
            },
            Vert
            {
                pos: [w as f32, -h as f32],
                tex_coords: [1f32, 0f32]
            },
            Vert
            {
                pos: [-w as f32, -h as f32],
                tex_coords: [0f32, 0f32]
            }
        ];

        let square_ind : [u32; 6] = [
            0, 1, 2, 0, 2, 3
        ];

        let ind_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &square_ind).expect("Can't create index buffer");
        let vert_buffer = VertexBuffer::new(&display, &square).expect("Can't create vert buffer");

        let shaders : ProgramCreationInput = ProgramCreationInput::SourceCode {
            vertex_shader: include_str!("./shaders/main.vert"),
            fragment_shader: include_str!("./shaders/main.frag"),

            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: false,
            uses_point_size: false,
        };

        let program = match Program::new(&display, shaders) {
            Ok(res) => { res },
            Err(e) => { panic!("{}", e.to_string().replace("\\n","\n")); }
        };

        let draw_params : DrawParameters = DrawParameters::default();

        let proj_matrix : Matrix4<f32> = cgmath::ortho(-w, w, h, -h, -1.0, 1.0);
        let model_matrix : Matrix4<f32> = Matrix4::from_translation(Vector3{ x: -((UI_MARGIN / 2) as f32), y: -((UI_MARGIN / 2) as f32), z:0.0 }) *
            Matrix4::from_nonuniform_scale(XRES as f32/WINW as f32, YRES as f32/WINH as f32, 1.0);

        let tex_filter = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Linear,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..Default::default()
        };

        (Self {
            grid_size: 0,
            camera_zoom: 1.0,
            camera_pan: Vector2::from([0.0, 0.0]),
            gui: GUI::new(&display),
            cursor: Rect::default(),
            texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, XRES as u32, YRES as u32).expect("Can't create texture"),
            display,
            vert_buffer,
            ind_buffer,
            program,
            draw_params,
            frame_start: Instant::now(),
            timers: [Instant::now(); 4],
            tex_filter,
            proj_matrix,
            view_matrix: Matrix4::identity(),
            model_matrix,
            fps_sum: 0.0,
            samples: 0,
            perf_sum: [0; 3],
        }, event_loop)
    }

    pub fn draw(&mut self, sim: &mut Simulation) {
        // FPS counter
        let dt = self.frame_start.elapsed().as_micros();

        self.fps_sum += 1000000f64 / dt as f64;
        // self.perf_sum[0] += (self.timers[1] - self.frame_start).as_micros();
        // self.perf_sum[1] += (self.timers[2] - self.timers[1]).as_micros();
        // self.perf_sum[2] += (self.timers[3] - self.timers[2]).as_micros();
        self.samples += 1;

        self.gui.add_text(format!("{:.2}", self.fps_sum/self.samples as f64).as_str(),
                          Vector2::new(0.0,0.0), Vector2::new(150.0, 50.0), 50.0,
                          None, None, Some(HorizontalAlign::Left), Some(VerticalAlign::Top));

        if self.timers[0].elapsed().as_millis() >= 1000 {
            //let fps = (self.samples as f64 * 1000f64) / self.timers[0].elapsed().as_millis() as f64;
            //self.fps_sum / self.samples as f64;
            // println!("From {} samples: {:.2} fps ({:.2}ms) {} parts", self.samples, fps, 1000f64 / fps, sim.get_part_count());
            //
            // println!(
            //     " Timings:\n  Tex Gen: {}??s\n  Tex Write: {}??s\n  Frame Finish: {}??s",
            //     self.perf_sum[0] / self.samples as u128,
            //     self.perf_sum[1] / self.samples as u128,
            //     self.perf_sum[2] / self.samples as u128,
            // );
            self.perf_sum = [0; 3];
            self.fps_sum = 0.0;
            self.samples = 0;
            self.timers[0] = Instant::now();
        }
        self.frame_start = Instant::now();

        let view_matrix =
            Matrix4::from_scale(self.camera_zoom) *
                Matrix4::from_translation(Vector3{x:self.camera_pan.x, y:self.camera_pan.y, z:0.0});

        self.view_matrix = view_matrix;
        let camera_matrix = self.proj_matrix * self.view_matrix * self.model_matrix;

        // Generate texture
        //self.texture = Texture2d::empty_with_format(&self.display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, XRES as u32, YRES as u32).expect("Can't create texture");
        let mut tex_data = vec![vec![(0u8, 0u8, 0u8, 0u8); XRES]; YRES];
        let mut counter = 0;
        for i in 0..sim.parts.len() {
            if counter >= sim.get_part_count() {
                break;
            }
            let pt = sim.get_part(i);
            if pt.p_type != 0 {
                let col = pt.get_type().col;
                tex_data[pt.y as usize][pt.x as usize] = (col[0],col[1],col[2],pt.p_type as u8);
                counter += 1;
            }
        }
        self.draw_cursor(&mut tex_data);
        self.texture.write(Rect{width: XRES as u32, height: YRES as u32, bottom: 0, left: 0}, tex_data);


        let uniforms = uniform! {
            tex: glium::uniforms::Sampler(&self.texture, self.tex_filter),
            pvm: <Matrix4<f32> as Into<[[f32;4];4]>>::into(camera_matrix),
            grid: self.grid_size as i32,
        };

        let mut frame = self.display.draw();
        frame.clear_color(1.0/255.0,0.0,0.0,0.0);
        frame.draw(&self.vert_buffer, &self.ind_buffer, &self.program, &uniforms, &self.draw_params).expect("Draw error");

        self.gui.draw_gui(&self.display, &mut frame);

        frame.finish().expect("Swap buffers error");

    }

    fn blend_colors(col_a : (u8,u8,u8,u8), col_b : (u8,u8,u8,u8), t : f32) -> (u8,u8,u8,u8){
        let mut col = col_a;
        let rt = 1.0 - t;
        col.0 = (col.0 as f32 * rt) as u8 + (col_b.0 as f32 * t) as u8;
        col.1 = (col.1 as f32 * rt) as u8 + (col_b.1 as f32 * t) as u8;
        col.2 = (col.2 as f32 * rt) as u8 + (col_b.2 as f32 * t) as u8;
        col.3 = u8::saturating_add(col.3, col_b.3);

        col
    }

    fn draw_cursor(&self, tex_data : &mut Vec<Vec<(u8,u8,u8,u8)>>) {
        for i in 0..self.cursor.height {
            let x = self.cursor.left as usize;
            let rx = (self.cursor.left + self.cursor.width-1) as usize;
            let y = (self.cursor.bottom + i) as usize;
            tex_data[y][x] = GLRenderer::blend_colors(tex_data[y][x], (255,255,255,128), 0.5);
            tex_data[y][rx] = GLRenderer::blend_colors(tex_data[y][rx], (255,255,255,128), 0.5);
        }
        for i in 1..self.cursor.width-1 {
            let x = (self.cursor.left + i) as usize;
            let ry = (self.cursor.bottom + self.cursor.height-1) as usize;
            let y = self.cursor.bottom as usize;
            tex_data[y][x] = GLRenderer::blend_colors(tex_data[y][x], (255,255,255,128), 0.5);
            tex_data[ry][x] = GLRenderer::blend_colors(tex_data[ry][x], (255,255,255,128), 0.5);
        }
    }
}