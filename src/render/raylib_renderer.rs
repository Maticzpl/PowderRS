use raylib::prelude::*;
use crate::sim;
use crate::sim::Simulation;
use crate::render::renderer;
use crate::render::renderer::Renderer;

pub struct RaylibRenderer{
    win_size: (i32, i32),
    rayl: RaylibHandle,
    thread: RaylibThread,
    camera: Camera2D
}

impl Renderer for RaylibRenderer {
    fn draw(&mut self, sim : &Simulation) {
        let (winW, winH) = self.win_size;

        self.camera.zoom = self.rayl.get_screen_height() as f32 / winH as f32;
        self.camera.offset.x = (self.rayl.get_screen_width() as f32 - (winW as f32 * self.camera.zoom)) / 2f32;

        let mut d = self.rayl.begin_drawing(&self.thread);
        let mut d = d.begin_mode2D(self.camera);

        d.clear_background(Color::BLACK);
        d.draw_rectangle_lines(0, 0, 775, 575, Color::RED);

        for i in 0..sim.parts.len() {
            let pt = sim.parts[i];
            if pt.p_type != 0 {
                let x = pt.x * sim::SCALE;
                let y = pt.y * sim::SCALE;
                d.draw_rectangle(x as i32, y as i32, sim::SCALE as i32, sim::SCALE as i32, (pt.get_type().graphics)(sim, &pt));
            }
        }

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
        let win_size = (800, 600);
        let (rayl, thread) = raylib::init()
            .title("PowderRS")
            .size(win_size.0, win_size.1)
            .resizable()
            .build();

        let camera = Camera2D::default();

        Self {
            rayl,
            thread,
            win_size,
            camera
        }
    }

    pub fn get_handle(&self) -> &RaylibHandle {
        return &self.rayl;
    }
}