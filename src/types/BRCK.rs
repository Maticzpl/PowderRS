use crate::types::*;

use crate::types::PartBehaviour::Solid;

pub const PT_BRCK : PartType = PartType {
    id: 1,
    name: "BRCK",
    col: [128,128,128],
    behaviour: Solid,
    density: 20,
    graphics: no_gfx,
    update: no_update,
};