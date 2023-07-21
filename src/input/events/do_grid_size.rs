use winit::event::VirtualKeyCode;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::Simulation;

pub struct DoGridSize {}

impl InputEvent for DoGridSize {
	fn get_name(&self) -> String {
		String::from("DoGridSize")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Keyboard(VirtualKeyCode::G),
			state:            KeyState::Held,
			combine_previous: None
		}]
	}

	fn handle(
		&self,
		_sim: &mut Simulation,
		_ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData
	) {
		if input.scroll != 0.0 {
			if gui.grid_size == 0 {
				gui.grid_size = 3;
			}

			let mut grid = gui.grid_size as i32;
			grid += input.scroll.clamp(-1f32, 1f32) as i32;
			gui.grid_size = grid.clamp(3, 50) as u32;

			if gui.grid_size == 3 {
				gui.grid_size = 0;
			}

			input.scroll = 0.0; // capture scroll
		}
	}
}
