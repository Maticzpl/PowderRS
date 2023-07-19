#![feature(core_intrinsics)]

extern crate core;

mod input;
mod rendering;
mod sim;
mod types;

use std::collections::HashMap;
use std::rc::Rc;

use cgmath::{Vector2, Vector4, Zero};
use instant::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalPosition;

use crate::input::event_handling::{handle_events, InputData};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::{Particle, Simulation, WINH, WINW};

fn tick(ren: &mut Renderer, gui: &mut GameGUI, tick_state: &mut TickFnState) {
	// TODO: Move this elsewhere
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
	pub time_since_render: Instant,
	pub time_since_tick:   Instant,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
	let mut sim = Simulation::new();
	let ren = Renderer::new().await;
	let event_loop = ren.1;
	let ren = ren.0;
	let gui = GameGUI::new(Rc::clone(&ren.rendering_core));

	let tick_state = TickFnState {
		// TODO:  Move Those two elsewhere
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
		mouse_pos_vector:   Vector4::zero(),
		mouse_screen_pos:   Vector4::zero(),
		cursor_pos:         Vector2::zero(),
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
