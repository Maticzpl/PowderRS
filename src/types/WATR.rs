use raylib::color::Color;
use crate::types::*;
use crate::sim::Particle;
use crate::types::PartBehaviour::Fluid;

pub const PT_WATR : PartType = PartType {
    id: 3,
    name: "WATR",
    col: Color::SKYBLUE,
    behaviour: Fluid,
    density: 5,
    update
};


pub fn update(pt : &mut Particle) {

}