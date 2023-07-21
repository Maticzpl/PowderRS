use winit::event::{MouseButton, VirtualKeyCode};

use crate::input::event_handling::InputData;
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::Simulation;

/// `PRESSED` will make the event run only on the first frame of the key being pressed  
/// `HELD` will make the event run on every frame the key is pressed
/// `RELEASED` will make the event run only on the first frame the key is released
/// `NotHeld` will make the event run on every frame the key is released
pub enum KeyState {
	Pressed,
	Held,
	Released,
	NotHeld
}

pub enum LogicalOperator {
	And,
	Or
}

pub enum AnyKey {
	Keyboard(VirtualKeyCode),
	Mouse(MouseButton)
}

/// Describes a single key required to trigger an input event  
/// `combine_previous` defines the logical operator for combining with previous key event  
/// The logical operators are applied from left to right with no other rules regarding the order
pub struct KeyEvent {
	pub key:              AnyKey,
	pub state:            KeyState,
	pub combine_previous: Option<LogicalOperator>
}

pub trait InputEvent {
	fn get_name(&self) -> String; // Dynamic dispatch requires &self but it really shouldn't be used
	fn default_keys(&self) -> Vec<KeyEvent>;
	fn handle(
		&self,
		sim: &mut Simulation,
		ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData
	);
}
