use proc_macros::import_elements;

import_elements!("src/simulation/elements");

use crate::simulation::Particle;

// TODO: Separate stuff

#[derive(Copy, Clone)]
pub enum ElementBehaviour {
	Skip,
	Solid,
	Powder,
	Fluid,
	Gas
}

#[derive(Clone)]
pub struct Element {
	pub id:        u16,
	pub name:      &'static str,
	pub col:       [u8; 4],
	pub behaviour: ElementBehaviour,
	pub density:   u16,
	pub update:    Option<fn(pt: &mut Particle)>,
	pub default:   Particle
}


pub struct ElementManager {
	pub elements: Vec<Element>
}

impl ElementManager {
	pub fn new() -> Self {
		Self {
			elements: vec![EL_NONE, EL_BRCK, EL_DUST, EL_WATR]
		}
	}

	fn get_element(&self, name: &str) -> Option<&Element> {
		self.elements.iter().find(|x| x.name == name)
	}
}
