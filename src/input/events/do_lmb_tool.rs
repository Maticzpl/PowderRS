use cgmath::num_traits::pow;
use winit::event::MouseButton;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::{Particle, Simulation};

pub struct DoLmbTool {}

impl InputEvent for DoLmbTool {
	fn get_name(&self) -> String {
		String::from("DoLmbTool")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Mouse(MouseButton::Left),
			state:            KeyState::Held,
			combine_previous: None,
		}]
	}

	fn handle(
		&self,
		sim: &mut Simulation,
		_ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData,
	) {
		let size = gui.brush_size as usize;
		let hs = size / 2;
		let (x, y) = (input.cursor_pos.x, input.cursor_pos.y);

		for i in 0..pow(size, 2) {
			// TODO: actual tools instead of hardcoded stuff, same for RMB
			sim.add_part(Particle {
				p_type: 2,
				x:      (x - hs + i / size) as u32,
				y:      (y - hs + i % size) as u32,
			});
		}
	}
}
