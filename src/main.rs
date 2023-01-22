mod sim;
mod types;
mod input;
mod render;

use crate::input::handle_input;
use crate::types::*;
use crate::sim::{Particle, Simulation};
use crate::render::raylib_renderer::RaylibRenderer;
use crate::render::renderer::Renderer;

fn main() {
    let mut sim = Simulation::new();

    let ren = RaylibRenderer::new();

    while ren.should_close() {
        handle_input(&mut sim, &ren);

        //sim.step();
        ren.draw(&sim);
    }
}
