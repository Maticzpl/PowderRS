use crate::types::PartBehaviour::Skip;
use crate::types::*;

pub const PT_NONE: PartType = PartType {
	id:        0,
	name:      "NONE",
	col:       [0, 0, 0],
	behaviour: Skip,
	density:   0,
	graphics:  no_gfx,
	update:    no_update,
};
