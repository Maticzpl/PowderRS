use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Solid;
use crate::simulation::Particle;

const ID: u16 = 1u16;
pub const EL_BRCK: Element = Element {
	id:        ID,
	name:      "BRCK",
	col:       [128, 128, 128, 255],
	behaviour: Solid,
	density:   20,
	update:    None,
	default:   Particle::default().with_type(ID)
};
