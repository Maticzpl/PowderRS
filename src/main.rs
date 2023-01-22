#![feature(box_syntax)]

mod sim;
mod types;
mod input;
mod gl_renderer;
use crate::gl_renderer::GLRenderer;
use crate::input::handle_input;
use crate::types::*;
use crate::sim::{Particle, Simulation};

fn main() {
    let mut sim = Simulation::new();

    // for i in 0..100 {
    //     sim.add_part(Particle{p_type:1, x:i, y:1});
    // }

    let mut ren = GLRenderer::new(&sim);
    while !ren.should_close() {
        //handle_input(&mut sim, &ren);

        //sim.step();
        ren.draw(&sim);
    }
    ren.close();
}
