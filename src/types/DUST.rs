use raylib::color::Color;
use crate::types::*;
use crate::sim::Particle;
use crate::types::PartBehaviour::Powder;

pub const PT_DUST : PartType = PartType {
    id: 2,
    name: "DUST",
    col: Color::YELLOW,
    behaviour: Powder,
    density: 10,
    update
};


pub fn update(pt : &mut Particle) {

}