mod sim;
mod renderer;
mod types;

use raylib::prelude::*;
use crate::types::*;
use crate::renderer::Renderer;
use crate::sim::{Particle, Simulation};

fn main() {
    let (winW, winH) = (800, 600);
    let (mut rayl, thread) = raylib::init()
        .title("PowderRS")
        .size(winW, winH)
        .resizable()
        .build();

    println!("Raylib initialzied");

    let mut sim = Simulation::new();
    for i in 0..300 {
        //sim.add_part(Particle{p_type: PT_BRCK.id, x:20 + i, y: 40 + i / 2});
        // sim.add_part(Particle{p_type: PT_BRCK.id, x:18 + i, y: 50 + i});
        // sim.add_part(Particle{p_type: PT_BRCK.id, x:18 + i, y: 51 + i});
        //
        // sim.add_part(Particle{p_type: PT_BRCK.id, x:630 - i, y: 50 + i});
        // sim.add_part(Particle{p_type: PT_BRCK.id, x:630 - i, y: 51 + i});
        //
        // if i != 150 {
        //     sim.add_part(Particle{p_type: PT_BRCK.id, x:175 + i, y: 450});
        // } else {
        //     sim.add_part(Particle{p_type: PT_BRCK.id, x:175 + i, y: 451});
        //     sim.add_part(Particle{p_type: PT_BRCK.id, x:176 + i, y: 451});
        // }
    }

    let ren = Renderer::new();
    let mut cam = Camera2D::default();

    //rayl.set_target_fps(60);
    while !rayl.window_should_close() {
        cam.zoom = (rayl.get_screen_height() as f32 / winH as f32);
        cam.offset.x = (rayl.get_screen_width() as f32 - (winW as f32 * cam.zoom)) / 2f32;

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
        sim.step();

        //cam.target = Vector2 {x:400f32, y:300f32};

        let mut d = rayl.begin_drawing(&thread);
        let mut d = d.begin_mode2D(cam);
        d.clear_background(Color::BLACK);

        d.draw_rectangle_lines(0, 0, 775, 575, Color::RED);
        d.draw_fps(0,0);

        ren.draw_sim(d, &sim);
    }


}
