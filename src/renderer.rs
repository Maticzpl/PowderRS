use raylib::prelude::*;
use crate::sim;
use crate::sim::Simulation;

pub struct Renderer{
}
impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw_sim(&self, mut d: RaylibMode2D<RaylibDrawHandle>, sim : &Simulation) {
        for i in 0..sim.parts.len() {
            let pt = sim.parts[i];
            if pt.p_type != 0 {
                if sim::SCALE == 1 {
                    d.draw_pixel(pt.x as i32, pt.y as i32, (pt.get_type().graphics)(sim, &pt));
                } else {
                    let x = pt.x * sim::SCALE;
                    let y = pt.y * sim::SCALE;
                    d.draw_rectangle(x as i32, y as i32, sim::SCALE as i32, sim::SCALE as i32, (pt.get_type().graphics)(sim, &pt));
                }
            }
        }
    }
}