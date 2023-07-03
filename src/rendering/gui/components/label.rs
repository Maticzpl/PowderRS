use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::{Vector2, Zero};
use wgpu::Color;
use wgpu_glyph::{FontId, HorizontalAlign, VerticalAlign};

use crate::define_component;
use crate::rendering::gui::components::{Component, ComponentAlignment, ComponentBase};
use crate::rendering::gui::immediate_mode::gui_renderer::{Bounds, ImmediateGUI};

pub struct Label {
	base: ComponentBase,

	text:        String,
	font_size:   f32,
	color:       Color,
	font_id:     FontId,
	text_bounds: Bounds,
}

impl Label {
	pub fn new(
		text: &str,
		font_size: f32,
		color: Color,
		font_id: FontId,
		bounds: Bounds,
		offset: Vector2<f32>,
		gui: &mut ImmediateGUI,
		parent: Weak<RefCell<dyn Component>>,
	) -> Self {
		let mut base = ComponentBase::new(parent);

		let (size, calc_offset) = Self::calculate_size_and_offset(text, font_size, bounds, gui);

		base.size = size;
		base.offset = calc_offset + offset;
		base.user_offset = offset;

		Self {
			base,
			text: text.to_string(),
			font_size,
			color,
			font_id,
			text_bounds: bounds,
		}
	}

	fn calculate_size_and_offset(
		text: &str,
		font_size: f32,
		bounds: Bounds,
		gui: &mut ImmediateGUI,
	) -> (Vector2<f32>, Vector2<f32>) {
		let mut out_size = Vector2::zero();
		let mut offset = Vector2::zero();

		match bounds {
			Bounds::None => {
				out_size = gui.measure_text(&*text, font_size);
			}
			Bounds::Box {
				size,
				h_align,
				v_align,
			} => {
				match h_align {
					HorizontalAlign::Left => offset.x = 0.0,
					HorizontalAlign::Center => offset.x = size.x / 2.0,
					HorizontalAlign::Right => offset.x = size.x,
				}

				match v_align {
					VerticalAlign::Top => offset.y = 0.0,
					VerticalAlign::Center => offset.y = size.y / 2.0,
					VerticalAlign::Bottom => offset.y = size.y,
				}

				out_size = size;
			}
		}

		(out_size, offset)
	}

	fn recalculate_size_and_offset(&mut self, gui: &mut ImmediateGUI) {
		let (size, calc_offset) =
			Self::calculate_size_and_offset(&*self.text, self.font_size, self.text_bounds, gui);

		self.base.size = size;
		self.base.offset = calc_offset + self.base.user_offset;
	}

	// gui needs to be passed because it has logic for calculating text size >_>
	pub fn set_text(&mut self, text: &str, gui: &mut ImmediateGUI) {
		self.text = String::from(text);
		self.recalculate_size_and_offset(gui);
	}

	pub fn set_font_size(&mut self, size: f32, gui: &mut ImmediateGUI) {
		self.font_size = size;
		self.recalculate_size_and_offset(gui);
	}

	pub fn set_font(&mut self, font: FontId, gui: &mut ImmediateGUI) {
		self.font_id = font;
		self.recalculate_size_and_offset(gui);
	}

	pub fn set_bounds(&mut self, bounds: Bounds, gui: &mut ImmediateGUI) {
		self.text_bounds = bounds;
		self.recalculate_size_and_offset(gui);
	}

	pub fn set_color(&mut self, color: Color) {
		self.color = color;
	}
}

define_component! { Label,
	fn draw(&self, gui: &mut ImmediateGUI) {
		gui.queue_text(&self.text, self.base.get_screen_pos(), self.text_bounds, self.font_size, Some(self.color), Some(self.font_id));
		self.base.draw(gui);
	}
}
