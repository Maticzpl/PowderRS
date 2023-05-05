#![feature(macro_metavar_expr)]

mod input;
mod rendering;
mod sim;
mod types;

use std::collections::HashMap;
use std::time::Instant;

use cgmath::Vector2;
use glium::glutin::dpi::{PhysicalPosition, PhysicalSize};

use crate::input::{handle_events, handle_input, InputData};
use crate::rendering::gl_renderer::GLRenderer;
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::gui::immediate_mode::gui_renderer::Bounds;
use crate::sim::{Particle, Simulation, WINH, WINW};

// struct GameData<'a, T: SimulationRenderer, U: CameraController> {
// 	sim:        &'a mut Simulation,
// 	ren:        &'a mut T,
// 	cam:        &'a mut U,
// 	gui:        &'a mut GameGUI<'a>,
// 	input:      &'a mut InputData,
// 	tick_state: &'a mut TickFnState,
// }
// mut data: GameData<impl SimulationRenderer, impl CameraController>

fn tick(
	sim: &mut Simulation,
	ren: &mut GLRenderer,
	gui: &mut GameGUI,
	input: &mut InputData,
	tick_state: &mut TickFnState,
) {
	handle_input(sim, ren, gui, input, tick_state);

	let dt = tick_state.time_since_tick.elapsed().as_micros();
	let fps = 1000000 as f64 / dt as f64;
	tick_state.time_since_tick = Instant::now();

	// draw cap
	if tick_state.time_since_render.elapsed().as_micros() > (1000000 / 80) {
		gui.immediate_gui.queue_text(
			format!("{:.2}", fps).as_str(),
			Vector2::new(0.0, 50.0),
			Bounds::None,
			50.0,
			None,
			None,
		);
		ren.render(sim, gui);
		tick_state.time_since_render = Instant::now();
	}
}

pub struct TickFnState {
	pub pan_started:       bool,
	pub pan_start_pos:     Vector2<f32>,
	pub pan_original:      Vector2<f32>,
	pub brush_size:        i32,
	pub paused:            bool,
	pub time_since_render: Instant,
	pub time_since_tick:   Instant,
}

fn main() {
	let mut sim = Simulation::new();
	let ren = GLRenderer::new();
	let event_loop = ren.1;
	let ren = ren.0;
	let gui = GameGUI::new(&ren.display);

	let tick_state = TickFnState {
		// TODO: Ton of things here should be elsewhere
		pan_started:       false,
		pan_start_pos:     Vector2 { x: 0.0, y: 0.0 },
		pan_original:      Vector2::from([0.0, 0.0]),
		brush_size:        5,
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
		win_size:           PhysicalSize {
			width:  WINW as u32,
			height: WINH as u32,
		},
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

	handle_events(event_loop, input, sim, ren, gui, tick_state);
}
