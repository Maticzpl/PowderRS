use std::ops::{Add, Sub};
use cgmath::Vector2;
use glium::{Display, implement_vertex, IndexBuffer, Surface, VertexBuffer};
use glium::glutin::dpi::Size;
use glium::index::PrimitiveType;
use glium_glyph::glyph_brush::ab_glyph::FontRef;
use glium_glyph::{GlyphBrush, GlyphBrushBuilder};
use glium_glyph::glyph_brush::{Color, Section, Text};
use crate::gl_renderer::Vert;

use crate::sim::{XRES, YRES};

pub struct GUI<'a, 'font> {
    font : GlyphBrush<'a, FontRef<'font>>,
    ind_buffer: IndexBuffer<u32>,
    vert_buffer : VertexBuffer<Vert>,
}

impl GUI<'_, '_> {
    pub(crate) fn new(display: &Display) -> Self {
        let ttf: &[u8] = include_bytes!("../../ChakraPetch-Regular.ttf");
        let font = FontRef::try_from_slice(ttf).unwrap();

        let glyph_brush = GlyphBrushBuilder::using_font(font).build(display);

        let cap = 50usize;                                                                                          // Rectangles having 4 verts out of 2 triangles
        let ind_buffer : IndexBuffer<u32> = IndexBuffer::empty(display, PrimitiveType::TrianglesList, (cap as f32 * (6.0/4.0)) as usize).expect("Can't create index buffer");
        let vert_buffer : VertexBuffer<Vert> = VertexBuffer::empty(display, cap).expect("Can't create vert buffer");

        return Self {
            font: glyph_brush,
            ind_buffer,
            vert_buffer
        };
    }

    pub fn add_text(&mut self, text: &str, pos: Vector2<f32>, size: f32, color: Option<Color>){
        self.font.queue(
            Section::default()
                .with_text(vec![
                    Text::new(text)
                        .with_scale(size)
                        .with_color(color.unwrap_or(Color::from([1.0, 1.0, 1.0, 1.0])))
                ])
                .with_screen_position((pos.x, pos.y))
        );
    }

    pub fn add_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {

    }

    pub fn draw_gui<F : Surface>(&mut self, display: &Display, frame: &mut F) {
        self.add_text("BRCK DUST WATR", Vector2::from([0.0, YRES as f32]), 25.0, None);

        self.font.draw_queued(display, frame);
    }

}