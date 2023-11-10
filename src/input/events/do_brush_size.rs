use winit::keyboard::{KeyCode, PhysicalKey};

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState, LogicalOperator};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::simulation::sim::Simulation;

pub struct DoBrushSize {}

impl InputEvent for DoBrushSize {
	fn get_name(&self) -> String {
		String::from("DoBrushSize")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::ShiftLeft)),
				state:            KeyState::Held,
				combine_previous: None
			},
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::ShiftLeft)),
				state:            KeyState::NotHeld,
				combine_previous: Some(LogicalOperator::Or)
			},
		]
	}

	fn handle(
		&self,
		_sim: &mut Simulation,
		_ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData
	) {
		if input.scroll != 0.0 {
			let mut speed = 1;
			if input.key_pressed(&PhysicalKey::Code(KeyCode::ShiftLeft)) {
				speed = 2;
			}
			gui.brush_size =
				(gui.brush_size as i32 + input.scroll.signum() as i32 * speed).clamp(1, 40) as u32;
		}
	}
}
