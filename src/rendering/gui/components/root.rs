use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::{Vector2, Zero};

use crate::rendering::gui::components::{Component, ComponentAlignment};
use crate::rendering::gui::immediate_mode::gui_renderer::ImmediateGUI;

pub struct Root {
	children: Vec<Rc<RefCell<dyn Component>>>
}

impl Root {
	pub(crate) fn new() -> Self {
		Self { children: vec![] }
	}
}

impl Component for Root {
	fn get_size(&self) -> Vector2<f32> {
		let (w, h) = (0, 0); // TODO window size here

		Vector2::new(w as f32, h as f32)
	}

	fn get_offset(&self) -> Vector2<f32> {
		Vector2::zero()
	}

	fn get_alignment(&self) -> ComponentAlignment {
		ComponentAlignment::LeftBottom
	}

	fn get_screen_pos(&self) -> Vector2<f32> {
		Vector2::zero()
	}

	fn set_offset(&mut self, _offset: Vector2<f32>) {
		// dont do anything
	}

	fn add_offset(&mut self, _offset: Vector2<f32>) {
		// dont do anything
	}

	fn set_alignment(&mut self, _alignment: ComponentAlignment) {
		// dont do anything
	}

	fn set_size(&mut self, _size: Vector2<f32>) {
		// ignore ):
	}

	fn get_parent(&self) -> Weak<RefCell<dyn Component>> {
		Weak::<RefCell<Root>>::new() // Its supposed to be none
	}

	fn add_child(&mut self, child: Rc<RefCell<dyn Component>>) {
		self.children.push(child);
	}

	fn draw(&self, gui: &mut ImmediateGUI) {
		for child in &self.children {
			child.borrow().draw(gui)
		}
	}
}
