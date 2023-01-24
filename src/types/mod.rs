use crate::sim::{Particle, Simulation};

pub mod NONE;
pub use NONE::PT_NONE;

pub mod BRCK;
pub use BRCK::PT_BRCK;

pub mod DUST;
pub use DUST::PT_DUST;

pub mod WATR;
pub use WATR::PT_WATR;

pub const PT_TYPES : [PartType; 4] = [
    PT_NONE,
    PT_BRCK,
    PT_DUST,
    PT_WATR
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