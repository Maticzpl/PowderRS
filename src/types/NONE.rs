use raylib::color::Color;
use crate::types::*;
use crate::sim::Particle;
use crate::types::PartBehaviour::Skip;

pub const PT_NONE : PartType = PartType {
    id: 0,
    name: "NONE",
    col: Color::BLACK,
    behaviour: Skip,
    density: 0,
    update: no_update
};