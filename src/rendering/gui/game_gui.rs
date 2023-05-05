use glium::backend::glutin::Display;
use glium::Rect;

use crate::rendering::gui::immediate_mode::gui_renderer::ImmediateGUI;

pub struct GameGUI<'a> {
	pub immediate_gui: ImmediateGUI<'a, 'a>,
	pub grid_size:     u32,
	pub cursor:        Rect,
}

impl GameGUI<'_> {
	pub(crate) fn new(display: &Display) -> Self {
		Self {
			immediate_gui: ImmediateGUI::new(&display),
			grid_size:     0,
			cursor:        Rect::default(),
		}
	}
}
