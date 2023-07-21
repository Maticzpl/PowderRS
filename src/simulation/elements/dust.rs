use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Powder;

pub const EL_DUST: Element = Element {
	id:        2,
	name:      "DUST",
	col:       [220, 220, 0, 255],
	behaviour: Powder,
	density:   10,
	update:    None
};
