#![feature(core_intrinsics)]

extern crate core;

mod input;
mod rendering;
mod sim;
mod types;

use std::collections::HashMap;
use std::rc::Rc;

use cgmath::Vector2;
use instant::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalPosition;

use crate::input::input_handling::{handle_events, handle_input, InputData};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::GLRenderer;
use crate::sim::{Particle, Simulation, WINH, WINW};

fn tick(
	sim: &mut Simulation,
	ren: &mut GLRenderer,
	gui: &mut GameGUI,
	input: &mut InputData,
	tick_state: &mut TickFnState,
) {
	handle_input(sim, ren, gui, input, tick_state);

	let dt = tick_state.time_since_tick.elapsed().as_micros();
	let tps = 1000000 as f64 / dt as f64;
	tick_state.time_since_tick = Instant::now();

	// draw cap
	if tick_state.time_since_render.elapsed().as_micros() > (1000000 / 80) {
		gui.fps_displ.borrow_mut().tps = tps as f32;
		let core = ren.rendering_core.borrow();
		core.window.request_redraw();

		tick_state.time_since_render = Instant::now();
	}
}

pub struct TickFnState {
	pub pan_started:       bool,
	pub pan_start_pos:     Vector2<f32>,
	pub pan_original:      Vector2<f32>,
	pub paused:            bool,
	pub time_since_render: Instant,
	pub time_since_tick:   Instant,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
	let mut sim = Simulation::new();
	let ren = GLRenderer::new().await;
	let event_loop = ren.1;
	let ren = ren.0;
	let gui = GameGUI::new(Rc::clone(&ren.rendering_core));

	let tick_state = TickFnState {
		// TODO: Ton of things here should be elsewhere
		pan_started:       false,
		pan_start_pos:     Vector2 { x: 0.0, y: 0.0 },
		pan_original:      Vector2::from([0.0, 0.0]),
		paused:            false,
		time_since_render: Instant::now(),
		time_since_tick:   Instant::now(),
	};

	let input: InputData = InputData {
		// TODO: Ton of things here should be elsewhere
		mouse_buttons:      HashMap::new(),
		prev_mouse_buttons: HashMap::new(),
		keys:               HashMap::new(),
		prev_keys:          HashMap::new(),
		mouse_pos:          PhysicalPosition { x: 0.0, y: 0.0 },
		scroll:             0.0,
	};

	for i in 0..100 {
		sim.add_part(Particle {
			p_type: 1,
			x:      i + 20,
			y:      i + 50,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      i + 20,
			y:      i + 80,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      i + 120,
			y:      i + 50,
		});

		sim.add_part(Particle {
			p_type: 1,
			x:      i * 4,
			y:      450,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      (i * 4) + 1,
			y:      450,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      (i * 4) + 2,
			y:      450,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      (i * 4) + 3,
			y:      450,
		});

		sim.add_part(Particle {
			p_type: 1,
			x:      0,
			y:      450 - i,
		});
		sim.add_part(Particle {
			p_type: 1,
			x:      400,
			y:      450 - i,
		});
	}
	sim.add_part(Particle {
		p_type: 1,
		x:      (WINW / 2) as u32,
		y:      (WINH / 2) as u32,
	});

	let rendering_core = ren.rendering_core.clone();
	handle_events(event_loop, input, sim, ren, gui, tick_state, rendering_core);
}
