use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Powder;
use crate::simulation::Particle;

const ID: u16 = 2;
pub const EL_DUST: Element = Element {
	id:        ID,
	name:      "DUST",
	col:       [220, 220, 0, 255],
	behaviour: Powder,
	density:   10,
	update:    None,
	default:   Particle::default().with_type(ID)
};
