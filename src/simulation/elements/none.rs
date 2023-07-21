use crate::simulation::elements::Element;
use crate::simulation::elements::ElementBehaviour::Skip;
pub const EL_NONE: Element = Element {
	id:        0,
	name:      "NONE",
	col:       [0, 0, 0, 0],
	behaviour: Skip,
	density:   0,
	update:    None
};
