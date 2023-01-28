
use cgmath::{Matrix4, Vector2};
use glium::{Display, IndexBuffer, Surface, VertexBuffer};
use glium::index::PrimitiveType;
use glium_glyph::glyph_brush::ab_glyph::FontRef;
use glium_glyph::{GlyphBrush, GlyphBrushBuilder};
use glium_glyph::glyph_brush::{Color, Section, Text};
use crate::gl_renderer::Vert;

use crate::sim::{WINH, WINW, XRES, YRES};

pub struct GUI<'a, 'font> {
    font : GlyphBrush<'a, FontRef<'font>>,
    ind_buffer: IndexBuffer<u32>,
    vert_buffer : VertexBuffer<Vert>,
    rect_num: u32,
    window_scale_ratio: Vector2<f32>,
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
            vert_buffer,
            rect_num: 0,
            window_scale_ratio: Vector2::new(0.0, 0.0)
        };
    }

    pub fn add_text(&mut self, text: &str, mut pos: Vector2<f32>, size: f32, color: Option<Color>){
        pos.x *= self.window_scale_ratio.x;
        pos.y *= self.window_scale_ratio.y;

        self.font.queue(
            Section::default()
                .with_text(vec![
                    Text::new(text)
                        .with_scale(size * self.window_scale_ratio.y)
                        .with_color(color.unwrap_or(Color::from([1.0, 1.0, 1.0, 1.0])))
                ])
                .with_screen_position((pos.x, pos.y))
        );
    }

    pub fn add_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {
        let w_only = Vector2::from([size.x, 0.0   ]);
        let h_only = Vector2::from([0.0   , size.y]);

        // TODO, draw stuff below
        let off = self.rect_num as usize * 4;
        self.vert_buffer.map()[off + 0] = Vert {pos: pos.into(),             tex_coords: [0.0, 0.0]};
        self.vert_buffer.map()[off + 1] = Vert {pos: (pos + w_only).into(),  tex_coords: [1.0, 0.0]};
        self.vert_buffer.map()[off + 2] = Vert {pos: (pos + size).into(),    tex_coords: [1.0, 1.0]};
        self.vert_buffer.map()[off + 3] = Vert {pos: (pos + h_only).into(),  tex_coords: [0.0, 1.0]};

        //[off, off+1, off+2, off, off+2, off+3]
        let off = self.rect_num as usize * 6;
        self.ind_buffer.map()[off + 0] = (off  ) as u32;
        self.ind_buffer.map()[off + 1] = (off+1) as u32;
        self.ind_buffer.map()[off + 2] = (off+2) as u32;
        self.ind_buffer.map()[off + 3] = (off  ) as u32;
        self.ind_buffer.map()[off + 4] = (off+2) as u32;
        self.ind_buffer.map()[off + 5] = (off+3) as u32;

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

        self.add_text("gui go here", Vector2::from([0.0, YRES as f32]), 25.0, None);

        self.font.draw_queued_with_transform(transform.into() ,display, frame);
        self.rect_num = 0;
    }

}