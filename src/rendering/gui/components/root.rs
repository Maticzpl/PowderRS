use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::{Vector2, Zero};
use glium::Display;

use crate::rendering::gui::components::{Component, ComponentAlignment};
use crate::rendering::gui::immediate_mode::gui_renderer::ImmediateGUI;

pub struct Root {
	display: Rc<Display>,

	children: Vec<Rc<RefCell<dyn Component>>>,
}

impl Root {
	pub(crate) fn new(display: Rc<Display>) -> Self {
		Self {
			children: vec![],
			display,
		}
	}
}

impl Component for Root {
	fn get_size(&self) -> Vector2<f32> {
		let (w, h) = self.display.get_framebuffer_dimensions();

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

	fn set_offset(&mut self, offset: Vector2<f32>) {
		// dont do anything
	}

	fn add_offset(&mut self, offset: Vector2<f32>) {
		// dont do anything
	}

	fn set_alignment(&mut self, alignment: ComponentAlignment) {
		// dont do anything
	}

	fn set_size(&mut self, size: Vector2<f32>) {
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
