pub mod label;
pub mod root;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::{Vector2, Zero};
use wgpu::Color;

use crate::rendering::gui::immediate_mode::gui_renderer::ImmediateGUI;

#[derive(Copy, Clone)] #[rustfmt::skip]
pub enum ComponentAlignment {
	//                XXYY
	LeftTop = 		0b0000,
	MiddleTop = 	0b0100,
	RightTop = 		0b1000,

	LeftCenter = 	0b0001,
	Center = 		0b0101,
	RightCenter = 	0b1001,

	LeftBottom = 	0b0010,
	MiddleBottom = 	0b0110,
	RightBottom = 	0b1010,
}

impl ComponentAlignment {
	fn get_val(&self) -> Vector2<f32> {
		let val = *self as u8;
		let x = (val & 0b1100) >> 2;
		let y = val & 0b0011;
		Vector2::new(x as f32 / 2.0, y as f32 / 2.0)
	}
}

pub trait Component {
	fn get_size(&self) -> Vector2<f32>;
	fn get_offset(&self) -> Vector2<f32>;
	fn get_alignment(&self) -> ComponentAlignment;
	fn get_screen_pos(&self) -> Vector2<f32>;

	fn set_offset(&mut self, offset: Vector2<f32>);
	fn add_offset(&mut self, offset: Vector2<f32>);
	fn set_alignment(&mut self, alignment: ComponentAlignment);
	fn set_size(&mut self, size: Vector2<f32>);

	fn get_parent(&self) -> Weak<RefCell<dyn Component>>;

	fn add_child(&mut self, child: Rc<RefCell<dyn Component>>);

	fn draw(&self, gui: &mut ImmediateGUI);
}

pub struct ComponentBase {
	size:        Vector2<f32>,
	offset:      Vector2<f32>,
	user_offset: Vector2<f32>,
	alignment:   ComponentAlignment,

	parent:   Weak<RefCell<dyn Component>>,
	children: Vec<Rc<RefCell<dyn Component>>>,
}

impl ComponentBase {
	pub(crate) fn new(parent: Weak<RefCell<dyn Component>>) -> Self {
		Self {
			size: Vector2::zero(),
			offset: Vector2::zero(),
			user_offset: Vector2::zero(),
			children: vec![],
			parent,
			alignment: ComponentAlignment::LeftTop,
		}
	}
}

impl Component for ComponentBase {
	fn get_size(&self) -> Vector2<f32> {
		self.size
	}

	fn get_offset(&self) -> Vector2<f32> {
		self.offset
	}

	fn get_alignment(&self) -> ComponentAlignment {
		self.alignment
	}

	fn get_screen_pos(&self) -> Vector2<f32> {
		let mut pos = Vector2::zero();
		let mut parent_pos = Vector2::zero();
		if let Some(parent) = self.get_parent().upgrade() {
			let parent = parent.borrow();
			// Parent size gets multiplied by value from alignment creating a position in parent
			pos = parent.get_size();
			parent_pos = parent.get_screen_pos();
		}

		let align = self.get_alignment().get_val();
		pos = Vector2::new(pos.x * align.x, pos.y * align.y);
		pos -= Vector2::new(self.size.x * align.x, self.size.y * align.y);

		pos + self.offset + parent_pos
	}

	fn set_offset(&mut self, offset: Vector2<f32>) {
		self.offset -= self.user_offset;
		self.user_offset = offset;
		self.offset += offset;
	}

	fn add_offset(&mut self, offset: Vector2<f32>) {
		self.offset += offset;
		self.user_offset += offset;
	}

	fn set_size(&mut self, size: Vector2<f32>) {
		self.size = size;
	}

	fn set_alignment(&mut self, alignment: ComponentAlignment) {
		self.alignment = alignment;
	}

	fn get_parent(&self) -> Weak<RefCell<dyn Component>> {
		Weak::clone(&self.parent)
	}

	fn add_child(&mut self, child: Rc<RefCell<dyn Component>>) {
		self.children.push(child);
	}

	fn draw(&self, gui: &mut ImmediateGUI) {
		gui.queue_rect(
			self.get_screen_pos(),
			self.get_size(),
			Color {
				r: 1.0,
				g: 0.0,
				b: 0.0,
				a: 0.5,
			},
		);
		for child in &self.children {
			child.borrow().draw(gui)
		}
		// actual drawing happens in struct containing ComponentBase
	}
}

/// Implements trait Component for given struct
/// ```
/// define_component! { StructName,
///     fn draw(&self, gui: &ImmediateGUI) {
///        // Draw own stuff
///
///         // Draw children afterwards
///         self.base.draw(gui);
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_component {
	($name: ty, $code: item) => {
		impl Component for $name {
			fn get_size(&self) -> Vector2<f32> {
				self.base.get_size()
			}

			fn get_offset(&self) -> Vector2<f32> {
				self.base.get_offset()
			}

			fn get_alignment(&self) -> ComponentAlignment {
				self.base.get_alignment()
			}

			fn get_screen_pos(&self) -> Vector2<f32> {
				self.base.get_screen_pos()
			}

			fn set_offset(&mut self, offset: Vector2<f32>) {
				self.base.set_offset(offset);
			}

			fn add_offset(&mut self, offset: Vector2<f32>) {
				self.base.add_offset(offset);
			}

			fn set_alignment(&mut self, alignment: ComponentAlignment) {
				self.base.set_alignment(alignment);
			}

			fn set_size(&mut self, size: Vector2<f32>) {
				self.base.set_size(size);
			}

			fn get_parent(&self) -> Weak<RefCell<dyn Component>> {
				self.base.get_parent()
			}

			fn add_child(&mut self, child: Rc<RefCell<dyn Component>>){
				self.base.add_child(child)
			}

			$code
		}
	};
}
