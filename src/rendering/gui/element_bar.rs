use cgmath::Vector2;
use glium_glyph::glyph_brush::{Color, FontId};

use crate::gui::GUI;
use crate::sim::{UI_MARGIN, XRES, YRES};
use crate::types::PT_TYPES;

pub fn draw_element_bar(gui: &mut GUI) {
	let margin = 5.0;
	let size = Vector2::from([45.0, UI_MARGIN as f32 - (margin * 2.0)]);
	let mut x = XRES as f32;
	let y = YRES as f32 + margin;

	for pt_type in PT_TYPES {
		x -= size.x + margin;
		let pos = Vector2::from([x, y]);

		let mut col = [1.0f32; 4];
		col[0] = pt_type.col[0] as f32 / 255.0;
		col[1] = pt_type.col[1] as f32 / 255.0;
		col[2] = pt_type.col[2] as f32 / 255.0;

		let mut text_col = None;
		if col[0] + col[1] + col[2] > 3.0 / 2.0 {
			text_col = Some(Color::from([0.0, 0.0, 0.0, 1.0]));
		}

		gui.add_rect(pos, size, Color::from(col));
		gui.draw_text(
			pt_type.name,
			pos + size / 2.0,
			size,
			size.y - 2.0,
			text_col,
			Some(FontId(1)),
			None,
			None,
		);
	}
}
