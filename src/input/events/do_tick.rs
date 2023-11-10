use winit::keyboard::{KeyCode, PhysicalKey};

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState, LogicalOperator};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::simulation::sim::Simulation;

pub struct DoTick {}

impl InputEvent for DoTick {
	fn get_name(&self) -> String {
		String::from("DoTick")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::KeyF)),
				state:            KeyState::Pressed,
				combine_previous: None
			},
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::KeyV)),
				state:            KeyState::Pressed,
				combine_previous: Some(LogicalOperator::Or)
			},
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::KeyN)),
				state:            KeyState::Pressed,
				combine_previous: Some(LogicalOperator::Or)
			},
			KeyEvent {
				key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::KeyJ)),
				state:            KeyState::Pressed,
				combine_previous: Some(LogicalOperator::Or)
			},
		]
	}

	fn handle(
		&self,
		sim: &mut Simulation,
		_ren: &mut Renderer,
		_gui: &mut GameGUI,
		_input: &mut InputData
	) {
		sim.step();
	}
}
