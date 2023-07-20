use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use cgmath::{Matrix4, Transform, Vector2, Vector3, Vector4};
use instant::Instant;
use log::error;
use wgpu_glyph::ab_glyph::{Point, Rect};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::ElementState::Pressed;
use winit::event::MouseScrollDelta::{LineDelta, PixelDelta};
use winit::event::{Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::input::events::invoker::InputEventInvoker;
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::rendering::Core;
use crate::sim::{Simulation, WINH, WINW, XRES, YRES};

pub struct InputData {
	pub mouse_buttons:      HashMap<MouseButton, bool>,
	pub prev_mouse_buttons: HashMap<MouseButton, bool>,
	pub keys:               HashMap<VirtualKeyCode, bool>,
	pub prev_keys:          HashMap<VirtualKeyCode, bool>,
	pub mouse_pos:          PhysicalPosition<f64>,
	pub scroll:             f32,

	pub mouse_pos_vector: Vector4<f32>,
	pub mouse_screen_pos: Vector4<f32>,
	pub cursor_pos:       Vector2<usize>, // clamped mouse pos vector
}

impl InputData {
	pub fn key_pressed(&self, key: &VirtualKeyCode) -> bool {
		self.keys.get(key).is_some()
	}
	pub fn key_just_pressed(&self, key: &VirtualKeyCode) -> bool {
		self.keys.get(key).is_some() && self.prev_keys.get(key).is_none()
	}
	pub fn key_just_released(&self, key: &VirtualKeyCode) -> bool {
		self.keys.get(key).is_none() && self.prev_keys.get(key).is_some()
	}
	pub fn mouse_pressed(&self, button: &MouseButton) -> bool {
		self.mouse_buttons.get(button).is_some()
	}
	pub fn mouse_just_pressed(&self, button: &MouseButton) -> bool {
		self.mouse_buttons.get(button).is_some() && self.prev_mouse_buttons.get(button).is_none()
	}
	pub fn mouse_just_released(&self, button: &MouseButton) -> bool {
		self.mouse_buttons.get(button).is_none() && self.prev_mouse_buttons.get(button).is_some()
	}
}
fn tick(ren: &mut Renderer, gui: &mut GameGUI) {
	let mut display = gui.fps_display.borrow_mut();
	let dt = display.time_since_tick.elapsed().as_micros();
	let tps = 1000000f64 / dt as f64;
	display.time_since_tick = Instant::now();

	// draw cap
	if display.time_since_frame.elapsed().as_micros() > (1000000 / 65) {
		display.tps = tps as f32;
		let core = ren.rendering_core.borrow();
		core.window.request_redraw();

		let time_left = Duration::from_micros(1000000 / 60).as_micros() as i128
			- display.time_since_frame.elapsed().as_micros() as i128
			- 54i128;

		if time_left > 0 {
			sleep(Duration::from_micros(time_left as u64));
		}
		display.time_since_frame = Instant::now();
	}
}
pub fn handle_events(
	event_loop: EventLoop<()>,
	mut input: InputData,
	mut sim: Simulation,
	mut ren: Renderer,
	mut gui: GameGUI<'static>,
	rendering_core: Rc<RefCell<Core>>,
) {
	let invoker = InputEventInvoker::new();

	event_loop.run(move |event, _, flow| {
		let core = rendering_core.borrow();
		let win_id = core.window.id();
		let size = core.window_size;
		drop(core);

		match event {
			Event::WindowEvent {
				event: ev,
				window_id,
				..
			} if win_id == window_id => {
				match ev {
					WindowEvent::CloseRequested => {
						flow.set_exit();
					}
					WindowEvent::MouseInput { button, state, .. } => {
						if state == Pressed {
							input.mouse_buttons.insert(button, true);
						} else {
							input.mouse_buttons.remove(&button);
						}
					}
					WindowEvent::MouseWheel { delta, .. } => {
						if let LineDelta(_x, y) = delta {
							input.scroll = y;
						}
						if let PixelDelta(PhysicalPosition { x: _, y }) = delta {
							if input.scroll == 0.0 {
								input.scroll = y as f32;
							}
						}
						// Can happen with horizontal scroll
						if input.scroll != 0.0 {
							input.scroll = input.scroll.signum();
						}
					}
					WindowEvent::CursorMoved { position: pos, .. } => {
						input.mouse_pos.x = pos.x;
						input.mouse_pos.y = pos.y;
					}
					WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								virtual_keycode: Some(key),
								state,
								..
							},
						..
					} => {
						// println!("{:?} k-s {}",key,_scan);
						if state == Pressed {
							input.keys.insert(key, true);
						} else {
							input.keys.remove(&key);
						}
					}
					WindowEvent::Resized { 0: size } => {
						ren.resize(size);
					}
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
						ren.resize(*new_inner_size);
					}
					_ => {}
				}
			}
			Event::RedrawRequested(window_id) if window_id == win_id => {
				match ren.render(&sim, &mut gui) {
					Ok(_) => {}
					Err(wgpu::SurfaceError::Lost) => ren.resize(size),
					Err(wgpu::SurfaceError::OutOfMemory) => flow.set_exit(),
					Err(e) => error!("{:?}", e),
				}
			}
			Event::MainEventsCleared => {
				// TODO: Clean this up
				let win_size: PhysicalSize<u32>;
				{
					win_size = ren.rendering_core.borrow().window_size;
				}

				let mouse_pos = Vector4 {
					x: input.mouse_pos.x as f32,
					y: input.mouse_pos.y as f32,
					z: 0.0,
					w: 1.0,
				};
				let scale_factor = Vector2::new(
					win_size.width as f32 / WINW as f32,
					win_size.height as f32 / WINH as f32,
				);

				input.mouse_screen_pos = Vector4 {
					x: mouse_pos.x / scale_factor.x,
					y: mouse_pos.y / scale_factor.y,
					z: 0.0,
					w: 1.0,
				};

				#[rustfmt::skip]
				let mouse_pos =
					Matrix4::from_translation( Vector3 { x: (WINW as f32 / 2.0), y: (WINH as f32 / 2.0), z: 0.0 }) *
					ren.get_view_matrix().inverse_transform().unwrap() *
					Matrix4::from_translation(-Vector3 { x: (WINW as f32 / 2.0), y: (WINH as f32 / 2.0), z: 0.0,}) *
					input.mouse_screen_pos;
				input.mouse_pos_vector = mouse_pos;

				let (mut cursor_x, mut cursor_y) = (mouse_pos.x as usize, mouse_pos.y as usize);
				let mut hs = gui.brush_size as usize / 2usize;
				cursor_x = cursor_x.clamp(hs, XRES - hs - (gui.brush_size % 2) as usize);
				cursor_y = cursor_y.clamp(hs, YRES - hs - (gui.brush_size % 2) as usize);
				input.cursor_pos = Vector2::new(cursor_x, cursor_y);

				invoker.invoke(&mut sim, &mut ren, &mut gui, &mut input);
				if !sim.paused {
					invoker
						.get_event("DoTick")
						.unwrap()
						.handle(&mut sim, &mut ren, &mut gui, &mut input);
				}

				// Clamp again because brush_size can be modified in invoker.invoke()
				hs = gui.brush_size as usize / 2usize;
				cursor_x = cursor_x.clamp(hs, XRES - hs - (gui.brush_size % 2) as usize);
				cursor_y = cursor_y.clamp(hs, YRES - hs - (gui.brush_size % 2) as usize);
				input.cursor_pos = Vector2::new(cursor_x, cursor_y);

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

				tick(&mut ren, &mut gui);
				input.prev_keys = input.keys.clone();
				input.prev_mouse_buttons = input.mouse_buttons.clone();
			}
			_ => *flow = ControlFlow::Poll,
		}
	});
}
