use winit::event::VirtualKeyCode;

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
				key:              AnyKey::Keyboard(VirtualKeyCode::F),
				state:            KeyState::Pressed,
				combine_previous: None
			},
			KeyEvent {
				key:              AnyKey::Keyboard(VirtualKeyCode::V),
				state:            KeyState::Pressed,
				combine_previous: Some(LogicalOperator::Or)
			},
			KeyEvent {
				key:              AnyKey::Keyboard(VirtualKeyCode::N),
				state:            KeyState::Pressed,
				combine_previous: Some(LogicalOperator::Or)
			},
			KeyEvent {
				key:              AnyKey::Keyboard(VirtualKeyCode::J),
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
