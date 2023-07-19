use winit::event::VirtualKeyCode;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::Simulation;

pub struct DoPause {}

impl InputEvent for DoPause {
	fn get_name(&self) -> String {
		String::from("DoPause")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Keyboard(VirtualKeyCode::Space),
			state:            KeyState::Pressed,
			combine_previous: None,
		}]
	}

	fn handle(
		&self,
		sim: &mut Simulation,
		_ren: &mut Renderer,
		_gui: &mut GameGUI,
		_input: &mut InputData,
	) {
		sim.paused = !sim.paused;
	}
}
