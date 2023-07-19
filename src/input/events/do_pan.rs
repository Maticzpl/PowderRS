use std::cell::Cell;

use cgmath::{Vector2, Zero};
use winit::event::MouseButton;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState, LogicalOperator};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::Simulation;

pub struct DoPan {
	pan_started:   Cell<bool>, // Uh oh
	pan_start_pos: Cell<Vector2<f32>>,
	pan_original:  Cell<Vector2<f32>>,
}

impl DoPan {
	pub fn new() -> Self {
		Self {
			pan_started:   Cell::new(false),
			pan_start_pos: Cell::new(Vector2::zero()),
			pan_original:  Cell::new(Vector2::zero()),
		}
	}
}

impl InputEvent for DoPan {
	fn get_name(&self) -> String {
		String::from("DoPan")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![
			KeyEvent {
				key:              AnyKey::Mouse(MouseButton::Middle),
				state:            KeyState::Held,
				combine_previous: None,
			},
			KeyEvent {
				key:              AnyKey::Mouse(MouseButton::Middle),
				state:            KeyState::Released,
				combine_previous: Some(LogicalOperator::Or),
			},
		]
	}

	fn handle(
		&self,
		_sim: &mut Simulation,
		ren: &mut Renderer,
		_gui: &mut GameGUI,
		input: &mut InputData,
	) {
		// TODO: Fix slight misalignment while panning and window maximized
		let mut pan_started = self.pan_started.get();

		if input.mouse_pressed(&MouseButton::Middle) {
			let mut pan_start_pos = self.pan_start_pos.get();
			let mut pan_original = self.pan_original.get();

			let mut pan = ren.get_pan();
			if !pan_started {
				pan_start_pos = input.mouse_screen_pos.truncate().truncate();
				pan_original = pan;
				pan_started = true;
			} else {
				pan = pan_original
					+ (input.mouse_screen_pos.truncate().truncate() - pan_start_pos)
						/ ren.get_zoom();

				ren.set_pan(pan);
			}

			self.pan_start_pos.set(pan_start_pos);
			self.pan_original.set(pan_original);
		} else {
			pan_started = false;
		}

		self.pan_started.set(pan_started);
	}
}
