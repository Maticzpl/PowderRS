mod sim;
mod renderer;
mod types;

use raylib::prelude::*;
use crate::types::*;
use crate::renderer::Renderer;
use crate::sim::{Particle, Simulation};

fn main() {
    let (mut rayl, thread) = raylib::init()
        .title("Raylib in rust")
        .size(800, 600)
        .build();

    println!("Raylib initialzied");

    let mut sim = Simulation::new();
    for i in 0..300 {
        //sim.add_part(Particle{p_type: PT_BRCK.id, x:20 + i, y: 40 + i / 2});
        sim.add_part(Particle{p_type: PT_BRCK.id, x:18 + i, y: 50 + i});
        sim.add_part(Particle{p_type: PT_BRCK.id, x:18 + i, y: 51 + i});

        sim.add_part(Particle{p_type: PT_BRCK.id, x:626 - i, y: 50 + i});
        sim.add_part(Particle{p_type: PT_BRCK.id, x:626 - i, y: 51 + i});

        //sim.add_part(Particle{p_type: PT_BRCK.id, x:175 + i, y: 450});

        sim.add_part(Particle{p_type: PT_DUST.id, x:30 + (i % 10), y: 10 + (i / 10)});
    }

    let ren = Renderer::new();

    //rayl.set_target_fps(60);
    while !rayl.window_should_close() {
        let mut d = rayl.begin_drawing(&thread);
        sim.add_part(Particle{p_type: PT_DUST.id, x:30, y: 9});
        sim.add_part(Particle{p_type: PT_WATR.id, x:370, y: 9});
        sim.add_part(Particle{p_type: PT_DUST.id, x:600, y: 9});
        sim.step();
        d.clear_background(Color::BLACK);

        d.draw_rectangle_lines(0, 0, 775, 575, Color::RED);
        d.draw_fps(0,0);

        ren.draw_sim(d, &sim);
    }


}
