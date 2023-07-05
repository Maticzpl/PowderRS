use crate::types::PartBehaviour::Powder;
use crate::types::*;

pub const PT_DUST: PartType = PartType {
	id:        2,
	name:      "DUST",
	col:       [255, 255, 0],
	behaviour: Powder,
	density:   10,
	graphics:  no_gfx,
	update:    no_update,
};
