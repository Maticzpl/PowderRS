use raylib::color::Color;
use crate::types::*;
use crate::sim::Particle;
use crate::types::PartBehaviour::Solid;

pub const PT_BRCK : PartType = PartType {
    id: 1,
    name: "BRCK",
    col: Color::GRAY,
    behaviour: Solid,
    density: 20,
    update: no_update
};