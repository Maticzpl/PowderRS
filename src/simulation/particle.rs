use crate::simulation::elements::{Element, ElementManager, EL_NONE};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Particle {
	pub p_type: u16, // p_ because "type" is a keyword
	pub prop1:  u16, /* Dummy properties :P TODO: Come up with property names that make sense (impossible) */
	pub prop2:  u16,
	pub prop3:  u16,
	pub x:      f32,
	pub y:      f32,
	pub vx:     f32,
	pub vy:     f32
}

impl Particle {
	//            am lazy ok?
	pub fn new<T: Into<f32>>(p_type: u16, x: T, y: T) -> Self {
		let mut v = Self::default();
		v.p_type = p_type;
		v.x = x.into();
		v.y = y.into();
		v
	}

	pub const fn with_type(mut self, p_type: u16) -> Self {
		self.p_type = p_type;
		self
	}

	pub fn get_type<'a>(&'a self, elements: &'a ElementManager) -> &Element {
		elements
			.elements
			.get(self.p_type as usize)
			.unwrap_or(&EL_NONE)
	}

	// Not using trait cause it doesnt support const
	pub const fn default() -> Self {
		Self {
			p_type: 0, // Just hope that EL_NONE.id is always 0.
			prop1:  0,
			prop2:  0,
			prop3:  0,
			x:      0f32,
			y:      0f32,
			vx:     0f32,
			vy:     0f32
		}
	}
}
