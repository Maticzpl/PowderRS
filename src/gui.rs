use glium::backend::Facade;
use glium::{Display, Surface};
use glium_glyph::glyph_brush::ab_glyph::FontRef;
use glium_glyph::{GlyphBrush, GlyphBrushBuilder};
use glium_glyph::glyph_brush::{Color, Section, Text};
use crate::sim::{XRES, YRES};

pub struct GUI<'a, 'font> {
    glyph_brush : GlyphBrush<'a, FontRef<'font>>

}

impl GUI<'_, '_> {
    pub(crate) fn new(display: &Display) -> Self {
        let ttf: &[u8] = include_bytes!("../ChakraPetch-Regular.ttf");
        let font = FontRef::try_from_slice(ttf).unwrap();

        let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(display);

       return Self {
           glyph_brush
       };
    }
    //pub fn draw_gui<D : Sized, F : Surface>(&mut self, display: &D, mut frame: F)

    pub fn draw_gui<F : Surface>(&mut self, display: &Display, mut frame: &mut F) {

        self.glyph_brush.queue(
            Section::default()
                .with_text(vec![
                    Text::new("Test")
                        .with_scale(50.0)
                        .with_color(Color::from([1.0, 1.0, 1.0, 1.0]))
                ])
                .with_bounds((XRES as f32, YRES as f32))
                .with_screen_position((50.0, 50.0))
        );

        self.glyph_brush.draw_queued(display, frame);
    }

}