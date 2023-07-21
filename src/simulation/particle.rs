use crate::simulation::elements::{Element, ElementManager, EL_NONE};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Particle {
	pub p_type: u16,
	pub prop1:  u16, /* Dummy properties :P TODO: Come up with property names that make sense (impossible) */
	pub prop2:  u16,
	pub prop3:  u16,
	pub x:      u16,
	pub y:      u16,
	pub vx:     u16,
	pub vy:     u16
}

impl Particle {
	pub fn get_type<'a>(&'a self, elements: &'a ElementManager) -> &Element {
		elements
			.elements
			.get(self.p_type as usize)
			.unwrap_or(&EL_NONE)
	}
}

impl Default for Particle {
	fn default() -> Self {
		Self {
			p_type: EL_NONE.id,
			prop1:  0,
			prop2:  0,
			prop3:  0,
			x:      0,
			y:      0,
			vx:     0,
			vy:     0
		}
	}
}
