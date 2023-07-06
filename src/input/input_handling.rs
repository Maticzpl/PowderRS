use cgmath::num_traits::pow;
use cgmath::{Matrix4, Transform, Vector2, Vector3, Vector4, Zero};
use wgpu_glyph::ab_glyph::{Point, Rect};
use winit::dpi::PhysicalSize;
use winit::event::{MouseButton, VirtualKeyCode};

pub use crate::input::event_handling::{handle_events, InputData};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::GLRenderer;
use crate::sim::{Particle, Simulation, WINH, WINW, XRES, YRES};
pub use crate::TickFnState;

pub fn handle_input(
	sim: &mut Simulation,
	ren: &mut GLRenderer,
	gui: &mut GameGUI,
	input: &mut InputData,
	tick_state: &mut TickFnState,
) {
	if !tick_state.paused
		|| input.key_just_pressed(&VirtualKeyCode::F)
		|| input.key_just_pressed(&VirtualKeyCode::V)
		|| input.key_just_pressed(&VirtualKeyCode::J)
		|| input.key_just_pressed(&VirtualKeyCode::N)
	{
		sim.step();
	}

	// Toggle Pause
	if input.key_just_pressed(&VirtualKeyCode::Space) {
		tick_state.paused = !tick_state.paused;
	}

	let win_size: PhysicalSize<u32>;

	{
		win_size = ren.rendering_core.borrow().window_size;
	}

	// Correct mouse pos
	let mouse_pos = Vector4 {
		x: input.mouse_pos.x as f32,
		y: input.mouse_pos.y as f32,
		z: 0.0,
		w: 1.0,
	};
	let (sx, sy) = (
		win_size.width as f32 / WINW as f32,
		win_size.height as f32 / WINH as f32,
	);
	let mouse_screen_pos = Vector4 {
		x: mouse_pos.x / sx,
		y: mouse_pos.y / sy,
		z: 0.0,
		w: 1.0,
	};

	#[rustfmt::skip]
		let mouse_pos =
		Matrix4::from_translation( Vector3 { x: (WINW as f32 / 2.0), y: (WINH as f32 / 2.0), z: 0.0 }) *
			ren.get_view_matrix().inverse_transform().unwrap() *
			Matrix4::from_translation(-Vector3 { x: (WINW as f32 / 2.0), y: (WINH as f32 / 2.0), z: 0.0,}) *
			mouse_screen_pos;

	let (mut cursor_x, mut cursor_y) = (mouse_pos.x as usize, mouse_pos.y as usize);
	let hs = gui.brush_size as usize / 2usize; // todo uncomment
	cursor_x = cursor_x.clamp(hs, (XRES - hs - 1) as usize);
	cursor_y = cursor_y.clamp(hs, (YRES - hs - 1) as usize);

	// Brush stuff
	if input.mouse_pressed(&MouseButton::Left) {
		let size = gui.brush_size as usize;
		let (x, y) = (cursor_x, cursor_y);

		for i in 0..pow(size, 2) {
			sim.add_part(Particle {
				p_type: 2,
				x:      (x - hs + i / size) as u32,
				y:      (y - hs + i % size) as u32,
			});
		}
	}
	if input.mouse_pressed(&MouseButton::Right) {
		let size = gui.brush_size as usize;
		let (x, y) = (cursor_x, cursor_y);

		for i in 0..pow(size, 2) {
			let val = sim.get_pmap_val((x - hs + i % size) as usize, (y - hs + i / size) as usize);
			if val.is_some() {
				sim.kill_part(val.unwrap())
					.expect("Tried to kill invalid part");
			}
		}
	}

	// Panning
	if input.mouse_pressed(&MouseButton::Middle) {
		let mouse = Vector2 {
			x: mouse_screen_pos.x,
			y: mouse_screen_pos.y,
		};
		let mut pan = ren.get_pan();
		if !tick_state.pan_started {
			tick_state.pan_start_pos = mouse;
			tick_state.pan_original = pan;
			tick_state.pan_started = true;
		} else {
			pan = tick_state.pan_original + (mouse - tick_state.pan_start_pos) / ren.get_zoom();
			ren.set_pan(pan);
		}
	} else {
		tick_state.pan_started = false;
	}

	// Grid size
	if input.key_pressed(&VirtualKeyCode::G) && input.scroll != 0.0 {
		if gui.grid_size == 0 {
			gui.grid_size = 3;
		}

		let mut grid = gui.grid_size as i32;
		grid += input.scroll.clamp(-1f32, 1f32) as i32;
		gui.grid_size = grid.clamp(3, 50) as u32;

		if gui.grid_size == 3 {
			gui.grid_size = 0;
		}

		input.scroll = 0.0;
	}

	// Zooming
	if input.key_pressed(&VirtualKeyCode::LControl) && input.scroll != 0.0 {
		let prev_zoom = ren.get_zoom();
		let change = input.scroll / 10.0 * (ren.get_zoom() * 2.0);
		let mut zoom = ren.get_zoom() + change;
		zoom = zoom.clamp(1.0, 15.0);
		ren.set_zoom(zoom);

		#[rustfmt::skip]
			let res =
			Matrix4::from_translation( Vector3 { x: (WINW / 2) as f32, y: (WINH / 2) as f32, z: 0.0 }) *
				Matrix4::from_translation(-Vector3 { x: ren.get_pan().x,   y: ren.get_pan().y,	 z: 0.0 }) *
				Matrix4::from_scale(prev_zoom / zoom) *
				Matrix4::from_translation( Vector3 { x: ren.get_pan().x,   y: ren.get_pan().y,	 z: 0.0 }) *
				Matrix4::from_translation(-Vector3 { x: (WINW / 2) as f32, y: (WINH / 2) as f32, z: 0.0 }) *
				mouse_pos;

		ren.set_pan(ren.get_pan() + (res - mouse_pos).truncate().truncate());
	} else if input.scroll != 0.0 {
		let mut speed = 1;
		if input.key_pressed(&VirtualKeyCode::LShift) {
			speed = 2;
		}
		gui.brush_size =
			(gui.brush_size as i32 + input.scroll.signum() as i32 * speed).clamp(1, 40) as u32;
	}

	// Camera center
	if input.key_just_pressed(&VirtualKeyCode::L) {
		ren.set_pan(Vector2::zero());
		ren.set_zoom(1.0);
	}


	gui.cursor = Rect {
		min: Point {
			x: (cursor_x - hs) as f32,
			y: (cursor_y - hs) as f32,
		},
		max: Point {
			x: (cursor_x - hs) as f32 + gui.brush_size as f32,
			y: (cursor_y - hs) as f32 + gui.brush_size as f32,
		},
	};

	input.scroll = 0.0;
}
