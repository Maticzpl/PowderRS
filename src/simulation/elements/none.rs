use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Skip;
use crate::simulation::Particle;

const ID: u16 = 0;
pub const EL_NONE: Element = Element {
	id:        ID,
	name:      "NONE",
	col:       [0, 0, 0, 0],
	behaviour: Skip,
	density:   0,
	update:    None,
	default:   Particle::default()
};
