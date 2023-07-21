use cgmath::num_traits::pow;
use winit::event::MouseButton;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::simulation::sim::Simulation;

pub struct DoRmbTool {}

impl InputEvent for DoRmbTool {
	fn get_name(&self) -> String {
		String::from("DoRmbTool")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Mouse(MouseButton::Right),
			state:            KeyState::Held,
			combine_previous: None
		}]
	}

	fn handle(
		&self,
		sim: &mut Simulation,
		_ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData
	) {
		let size = gui.brush_size as usize;
		let hs = size / 2;
		let (x, y) = (input.cursor_pos.x, input.cursor_pos.y);

		for i in 0..pow(size, 2) {
			let val = sim.get_pmap_val(x - hs + i % size, y - hs + i / size);
			if let Some(part_id) = val {
				sim.kill_part(part_id).expect("Tried to kill invalid part");
			}
		}
	}
}
