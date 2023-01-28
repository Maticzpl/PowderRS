use crate::types::*;

use crate::types::PartBehaviour::Powder;

pub const PT_DUST : PartType = PartType {
    id: 2,
    name: "DUST",
    col: [255,255,0],
    behaviour: Powder,
    density: 10,
    graphics: no_gfx,
    update: no_update,
};
