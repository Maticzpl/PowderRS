use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Solid;

pub const EL_BRCK: Element = Element {
	id:        1,
	name:      "BRCK",
	col:       [128, 128, 128, 255],
	behaviour: Solid,
	density:   20,
	update:    None
};
