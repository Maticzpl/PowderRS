mod none;
mod brck;
mod dust;

use crate::sim::{Particle, Simulation};
pub use none::PT_NONE;
pub use brck::PT_BRCK;
pub use dust::PT_DUST;

pub const PT_TYPES: [PartType; 3] = [
    PT_NONE,
    PT_BRCK,
    PT_DUST
];

#[derive(Copy, Clone)]
pub enum PartBehaviour {
    Skip,
    Solid,
    Powder,
    Fluid,
    Gas
}

#[derive(Copy, Clone)]
pub struct PartType {
    pub id: u32,
    pub name: &'static str,
    pub col: [u8; 3],
    pub behaviour: PartBehaviour,
    pub density: u16,
    pub update: fn(pt : &mut Particle),
    pub graphics: fn(sim : &Simulation, pt : &Particle) -> [u8;3]
}
impl PartType {
    fn find(name : &str) -> usize {
        for (i, pt_type) in PT_TYPES.iter().enumerate() {
            if pt_type.name == name {
                return i;
            }
        }
        eprintln!("Type \"{}\" not found", name);
        return 0;
    }
}

pub fn no_update(_pt : &mut Particle) {}
pub fn no_gfx(_sim : &Simulation, pt : &Particle) -> [u8; 3]{
    return pt.get_type().col;
}