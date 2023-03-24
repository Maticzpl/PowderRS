#![feature(is_some_and)]
#![feature(macro_metavar_expr)]

mod sim;
mod types;
mod gl_renderer;
mod gui;
mod input;

use std::collections::{HashMap};
use std::time::Instant;
use cgmath::{Vector2};
use glium::glutin::dpi::{PhysicalPosition, PhysicalSize};
use glium_glyph::glyph_brush::{HorizontalAlign, VerticalAlign};

use crate::gl_renderer::GLRenderer;
use crate::input::{handle_events, handle_input, InputData};
use crate::sim::{Particle, Simulation, WINH, WINW};

fn tick(sim: &mut Simulation, ren: &mut GLRenderer, input: &mut InputData, tick_state : &mut TickFnState) {
    handle_input(sim, ren, input, tick_state);
    let dt = tick_state.time_since_tick.elapsed().as_micros();
    let fps = 1000000 as f64 / dt as f64;
    tick_state.time_since_tick = Instant::now();

    //draw cap
    if tick_state.time_since_render.elapsed().as_micros() > (1000000/80){
        ren.gui.add_text(format!("{:.2}", fps).as_str(),
             Vector2::new(0.0,50.0), Vector2::new(200.0, 50.0), 50.0,
             None, None, Some(HorizontalAlign::Left), Some(VerticalAlign::Top));
        ren.draw(sim);
        tick_state.time_since_render = Instant::now();
    }

}


pub struct TickFnState {
    pub pan_started: bool,
    pub pan_start_pos: Vector2<f32>,
    pub pan_original: Vector2<f32>,
    pub brush_size: i32,
    pub paused: bool,
    pub time_since_render: Instant,
    pub time_since_tick: Instant,
}

fn main() {
    let mut sim = Simulation::new();
    let ren = GLRenderer::new();
    let event_loop = ren.1;
    let ren = ren.0;

    let tick_state = TickFnState {
        pan_started: false,
        pan_start_pos: Vector2 {x: 0.0, y: 0.0},
        pan_original: Vector2::from([0.0, 0.0]),
        brush_size: 5,
        paused: false,
        time_since_render: Instant::now(),
        time_since_tick: Instant::now(),
    };

    let input: InputData = InputData {
        mouse_buttons: HashMap::new(),
        prev_mouse_buttons: HashMap::new(),
        keys: HashMap::new(),
        prev_keys : HashMap::new(),
        mouse_pos: PhysicalPosition{ x:0.0, y:0.0 },
        scroll: 0.0,
        win_size: PhysicalSize { width: WINW as u32, height: WINH as u32 },
    };

    for i in 0..100 {
        sim.add_part(Particle{p_type:1, x: i+20, y: i+50});
        sim.add_part(Particle{p_type:1, x: i+20, y: i+80});
        sim.add_part(Particle{p_type:1, x: i+120, y: i+50});

        sim.add_part(Particle{p_type:1, x: i * 4, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+1, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+2, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+3, y: 450});

        sim.add_part(Particle{p_type:1, x: 0, y: 450-i});
        sim.add_part(Particle{p_type:1, x: 400, y: 450-i});
    }
    sim.add_part(Particle{p_type:1, x: (WINW / 2) as u32, y: (WINH / 2) as u32 });

    handle_events(event_loop, input, sim, ren, tick_state);
}