use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event::ElementState::Pressed;
use winit::event::MouseScrollDelta::LineDelta;
use winit::event_loop::EventLoop;


use crate::rendering::gl_renderer::GLRenderer;
use crate::rendering::gui::game_gui::GameGUI;
use crate::sim::Simulation;
use crate::{tick, TickFnState};
use crate::rendering::wgpu::core::Core;

pub struct InputData {
	pub mouse_buttons:      HashMap<MouseButton, bool>,
	pub prev_mouse_buttons: HashMap<MouseButton, bool>,
	pub keys:               HashMap<VirtualKeyCode, bool>,
	pub prev_keys:          HashMap<VirtualKeyCode, bool>,
	pub mouse_pos:          PhysicalPosition<f64>,
	pub scroll:             f32,
	pub win_size:           PhysicalSize<u32>,
}

impl InputData {
	pub fn key_pressed(&self, key: &VirtualKeyCode) -> bool {
		self.keys.get(key).is_some()
	}
	pub fn key_just_pressed(&self, key: &VirtualKeyCode) -> bool {
		self.keys.get(key).is_some() && self.prev_keys.get(key).is_none()
	}
	pub fn mouse_pressed(&self, button: &MouseButton) -> bool {
		self.mouse_buttons.get(button).is_some()
	}
	pub fn mouse_just_pressed(&self, button: &MouseButton) -> bool {
		self.mouse_buttons.get(button).is_some() && self.prev_mouse_buttons.get(button).is_none()
	}
}

pub fn handle_events(
	event_loop: EventLoop<()>,
	mut input: InputData,
	mut sim: Simulation,
	mut ren: GLRenderer,
	mut gui: GameGUI<'static>,
	mut tick_state: TickFnState,
	rendering_core: Rc<RefCell<Core>>,
) {
	event_loop.run(move |event, _, flow| {
		let core = rendering_core.borrow();
		let win_id = core.window.id();
		let size = core.window_size;
		drop(core);

		match event {
			Event::WindowEvent { event: ev, window_id, .. } if win_id == window_id => {
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
					WindowEvent::MouseWheel {
						delta: LineDelta(_x, y),
						..
					} => {
						input.scroll = y;
					}
					WindowEvent::CursorMoved { position: pos, .. } => {
						input.mouse_pos.x = pos.x as f64;
						input.mouse_pos.y = pos.y as f64;
					}
					WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								virtual_keycode: key,
								state,
								scancode: _scan,
								..
							},
						..
					} => {
						// println!("{:?} k-s {}",key,_scan);

						if key.is_some() {
							let key = key.unwrap();
							if state == Pressed {
								input.keys.insert(key, true);
							} else {
								input.keys.remove(&key);
							}
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
					Err(wgpu::SurfaceError::Lost) => {
						ren.resize(size)
					},
					Err(wgpu::SurfaceError::OutOfMemory) => flow.set_exit(),
					Err(e) => eprintln!("{:?}", e),
				}
			}
			Event::MainEventsCleared => {
				tick(&mut sim, &mut ren, &mut gui, &mut input, &mut tick_state);
				input.prev_keys = input.keys.clone();
				input.prev_mouse_buttons = input.mouse_buttons.clone();
			}
			_ => {}
		}
	});
}
