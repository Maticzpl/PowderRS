use cgmath::{Vector2, Zero};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::simulation::sim::Simulation;

pub struct DoCameraCenter {}

impl InputEvent for DoCameraCenter {
	fn get_name(&self) -> String {
		String::from("DoCameraCenter")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Keyboard(PhysicalKey::Code(KeyCode::KeyL)),
			state:            KeyState::Pressed,
			combine_previous: None
		}]
	}

	fn handle(
		&self,
		_sim: &mut Simulation,
		ren: &mut Renderer,
		_gui: &mut GameGUI,
		_input: &mut InputData
	) {
		ren.set_pan(Vector2::zero());
		ren.set_zoom(1.0);
	}
}
