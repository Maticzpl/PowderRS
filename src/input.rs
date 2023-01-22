use raylib::prelude::*;
use crate::render::raylib_renderer::RaylibRenderer;
use crate::render::renderer::Renderer;
use crate::sim::{Particle, Simulation};
use crate::types::*;


pub fn handle_input(sim : &mut Simulation, ren: &RaylibRenderer) {
    let rayl = ren.get_handle();

    // Temporary stuff
    if rayl.is_key_down(KeyboardKey::KEY_A) {
        for i in 0..775 {
            sim.add_part(Particle { p_type: PT_WATR.id, x: i, y: 9 });
        }
    }

    if rayl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
        let pos = rayl.get_mouse_position();
        let (mx, my) = (pos.x as usize, pos.y as usize);
        for i in 0..25 {
            if sim.get_pmap_val(mx + i%5, my + i/5) == 0 {
                sim.add_part(Particle { p_type: PT_DUST.id, x: mx + i % 5, y: my + i / 5 });
            }
        }
    }
}