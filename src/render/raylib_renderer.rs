use raylib::prelude::*;
use crate::sim;
use crate::sim::Simulation;
use crate::render::renderer;
use crate::render::renderer::Renderer;

#[derive(Renderer)]
pub struct RaylibRenderer{
    win_size: (i32, i32),
    rayl: RaylibHandle,
    thread: RaylibThread,
    camera: Camera2D
}

impl Renderer for RaylibRenderer {
    fn draw(&self, sim : &Simulation) {
        cam.zoom = (rayl.get_screen_height() as f32 / winH as f32);
        cam.offset.x = (rayl.get_screen_width() as f32 - (winW as f32 * cam.zoom)) / 2f32;

        let mut d = rayl.begin_drawing(&thread);
        let mut d = d.begin_mode2D(cam);

        d.clear_background(Color::BLACK);
        d.draw_rectangle_lines(0, 0, 775, 575, Color::RED);
        ren.draw_sim(&mut d, &sim);
        d.draw_fps(0,0);
    }

    fn should_close(&self) -> bool {
        return self.rayl.window_should_close();
    }

    fn get_window_size(&self) -> (i32, i32) {
        return self.win_size;
    }

}

impl RaylibRenderer {
    pub fn new() -> Self {
        win_size = (800, 600);
        let (mut rayl, thread) = raylib::init()
            .title("PowderRS")
            .size(win_size.0, win_size.1)
            .resizable()
            .build();

        camera = Camera2D::default();

        Self {
            rayl,
            thread,
            win_size,
            camera
        }
    }

    pub fn get_handle(&self) -> &RaylibHandle {
        return &rayl;
    }

    fn draw_sim(&self, d: &mut RaylibMode2D<RaylibDrawHandle>, sim : &Simulation) {
        for i in 0..sim.parts.len() {
            let pt = sim.parts[i];
            if pt.p_type != 0 {
                let x = pt.x * sim::SCALE;
                let y = pt.y * sim::SCALE;
                d.draw_rectangle(x as i32, y as i32, sim::SCALE as i32, sim::SCALE as i32, (pt.get_type().graphics)(sim, &pt));
            }
        }
    }
}