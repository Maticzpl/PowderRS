mod element_bar;

use std::collections::HashMap;
use std::hash::Hash;
use cgmath::{Matrix4, Vector2, Vector3, SquareMatrix};
use glium::{Display, DrawParameters, implement_vertex, IndexBuffer, Program, Surface, uniform, VertexBuffer};
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;
use glium_glyph::glyph_brush::ab_glyph::{Font, FontRef};
use glium_glyph::{GlyphBrush, GlyphBrushBuilder};
use glium_glyph::glyph_brush::{Color, Extra, HorizontalAlign, Section, Text, Layout, VerticalAlign, FontId};
use crate::gui::element_bar::draw_element_bar;

use crate::sim::{WINH, WINW, XRES, YRES};

#[derive(Copy, Clone)]
pub struct GUIVert {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

implement_vertex!(GUIVert, pos, color);

pub struct GUI<'a, 'font> {
    font : GlyphBrush<'a, FontRef<'font>>,
    ind_buffer: IndexBuffer<u32>,
    vert_buffer : VertexBuffer<GUIVert>,
    program: Program,
    rect_num: u32,
    window_scale_ratio: Vector2<f32>,
}

impl GUI<'_, '_> {
    pub(crate) fn new(display: &Display) -> Self {
        let ttf: &[u8] = include_bytes!("../../ChakraPetch-Regular.ttf");
        let font = FontRef::try_from_slice(ttf).unwrap();

        let ttf: &[u8] = include_bytes!("../../ChakraPetch-Bold.ttf");
        let bold = FontRef::try_from_slice(ttf).unwrap();

        let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(display);
        let bold_id = glyph_brush.add_font(bold);

        let cap = 50usize;                                                                                          // Rectangles having 4 verts out of 2 triangles
        let ind_buffer : IndexBuffer<u32> = IndexBuffer::empty(display, PrimitiveType::TrianglesList, (cap as f32 * (6.0/4.0)) as usize).expect("Can't create index buffer");
        let vert_buffer : VertexBuffer<GUIVert> = VertexBuffer::empty(display, cap).expect("Can't create vert buffer");

        let program = match Program::new(display, ProgramCreationInput::SourceCode {
            vertex_shader: include_str!("../shaders/gui.vert"),
            fragment_shader: include_str!("../shaders/gui.frag"),

            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: false,
            uses_point_size: false,
        }) {
            Ok(res) => { res },
            Err(e) => { panic!("{}", e.to_string().replace("\\n","\n")); }
        };

        return Self {
            // fonts: HashMap::from([
            //     ("Regular".to_owned(), FontId(0)),
            //     ("Bold".to_owned(), bold_id)
            // ]),
            program,
            font: glyph_brush,
            ind_buffer,
            vert_buffer,
            rect_num: 0,
            window_scale_ratio: Vector2::new(0.0, 0.0)
        };
    }

    pub fn add_text(&mut self, text: &str, mut pos: Vector2<f32>, mut size: Vector2<f32>, font_size: f32, color: Option<Color>, font: Option<FontId>, h_align: Option<HorizontalAlign>, v_align: Option<VerticalAlign>){
        pos.x *= self.window_scale_ratio.x;
        pos.y *= self.window_scale_ratio.y;
        size.x *= self.window_scale_ratio.x;
        size.y *= self.window_scale_ratio.y;

        let layout = Layout::default().h_align(h_align.unwrap_or(HorizontalAlign::Center)).v_align(v_align.unwrap_or(VerticalAlign::Center));

        self.font.queue(
            Section::default()
                .with_text(vec![
                    Text::new(text)
                        .with_scale(font_size * self.window_scale_ratio.y)
                        .with_color(color.unwrap_or(Color::from([1.0, 1.0, 1.0, 1.0])))
                        .with_font_id(font.unwrap_or(FontId(0)))
                ])
                .with_screen_position((pos.x, pos.y))
                .with_bounds(size)
                .with_layout(layout)
        );
    }

    pub fn add_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {
        let w_only = Vector2::from([size.x, 0.0   ]);
        let h_only = Vector2::from([0.0   , size.y]);

        let vert_off = self.rect_num as usize * 4;
        self.vert_buffer.map()[vert_off + 0] = GUIVert {pos: pos.into(),             color: color.map(|v| v)};
        self.vert_buffer.map()[vert_off + 1] = GUIVert {pos: (pos + w_only).into(),  color: color.map(|v| v)};
        self.vert_buffer.map()[vert_off + 2] = GUIVert {pos: (pos + size).into(),    color: color.map(|v| v)};
        self.vert_buffer.map()[vert_off + 3] = GUIVert {pos: (pos + h_only).into(),  color: color.map(|v| v)};

        let off = self.rect_num as usize * 6;
        self.ind_buffer.map()[off + 0] = (vert_off  ) as u32;
        self.ind_buffer.map()[off + 1] = (vert_off+1) as u32;
        self.ind_buffer.map()[off + 2] = (vert_off+2) as u32;
        self.ind_buffer.map()[off + 3] = (vert_off  ) as u32;
        self.ind_buffer.map()[off + 4] = (vert_off+2) as u32;
        self.ind_buffer.map()[off + 5] = (vert_off+3) as u32;

        self.rect_num += 1;
    }

    pub fn draw_gui<F : Surface>(&mut self, display: &Display, frame: &mut F) {
        let (w, h) = display.get_framebuffer_dimensions();
        self.window_scale_ratio = Vector2::from([w as f32 / WINW as f32, h as f32 / WINH as f32]);

        let transform = Matrix4::new(
            2.0 / (w as f32), 0.0, 0.0, 0.0,
            0.0, 2.0 / (h as f32), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -1.0, -1.0, 0.0, 1.0
        );

        let uniforms = uniform! {
            transform: <Matrix4<f32> as Into<[[f32;4];4]>>::into(
                Matrix4::from_nonuniform_scale(2.0/WINW as f32, -2.0/WINH as f32, 1.0) *
                Matrix4::from_translation(Vector3::from([-(WINW as f32/2.0), -(WINH as f32/2.0), 0.0]))
            )
        };

        draw_element_bar(self);

        frame.draw(&self.vert_buffer, &self.ind_buffer, &self.program, &uniforms, &DrawParameters::default()).expect("GUI Draw error");
        self.font.draw_queued_with_transform(transform.into() ,display, frame);
        self.rect_num = 0;
    }

}