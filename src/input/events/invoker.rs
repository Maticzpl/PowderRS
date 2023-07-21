use log::warn;

use crate::input::event_handling::InputData;
use crate::input::events::do_brush_size::DoBrushSize;
use crate::input::events::do_camera_center::DoCameraCenter;
use crate::input::events::do_grid_size::DoGridSize;
use crate::input::events::do_lmb_tool::DoLmbTool;
use crate::input::events::do_pan::DoPan;
use crate::input::events::do_pause::DoPause;
use crate::input::events::do_rmb_tool::DoRmbTool;
use crate::input::events::do_tick::DoTick;
use crate::input::events::do_zoom::DoZoom;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyState, LogicalOperator};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::simulation::sim::Simulation;

pub struct InputEventInvoker {
	events: Vec<Box<dyn InputEvent>>
}

impl InputEventInvoker {
	pub fn new() -> Self {
		Self {
			events: vec![
				Box::from(DoTick {}),
				Box::from(DoPause {}),
				Box::from(DoLmbTool {}),
				Box::from(DoRmbTool {}),
				Box::from(DoPan::new()),
				Box::from(DoGridSize {}),
				Box::from(DoZoom {}),
				Box::from(DoCameraCenter {}),
				Box::from(DoBrushSize {}),
			]
		}
	}

	pub fn get_event(&self, name: &str) -> Option<&dyn InputEvent> {
		for event in self.events.as_slice() {
			if event.get_name() == name {
				return Some(event.as_ref());
			}
		}
		None
	}

	pub fn invoke(
		&self,
		sim: &mut Simulation,
		ren: &mut Renderer,
		gui: &mut GameGUI,
		input: &mut InputData
	) {
		for event in self.events.as_slice() {
			let mut previous: Option<bool> = None;

			for key in event.default_keys() {
				let triggered = match key.key {
					// TODO: Make input.key.. methods handle AnyKey
					AnyKey::Keyboard(keycode) => match key.state {
						KeyState::Pressed => input.key_just_pressed(&keycode),
						KeyState::Held => input.key_pressed(&keycode),
						KeyState::Released => input.key_just_released(&keycode),
						KeyState::NotHeld => !input.key_pressed(&keycode)
					},
					AnyKey::Mouse(button) => match key.state {
						KeyState::Pressed => input.mouse_just_pressed(&button),
						KeyState::Held => input.mouse_pressed(&button),
						KeyState::Released => input.mouse_just_released(&button),
						KeyState::NotHeld => !input.mouse_pressed(&button)
					}
				};

				if let Some(state) = previous {
					match key.combine_previous {
						Some(LogicalOperator::And) => previous = Some(state && triggered),
						Some(LogicalOperator::Or) => previous = Some(state || triggered),
						_ => {}
					}
				}
				else {
					previous = Some(triggered);
					if key.combine_previous.is_some() {
						warn!(
							"Key event {}: first KeyEvent in default_keys cannot use \"combine_previous\"",
							event.get_name()
						)
					}
				}
			}

			if previous.is_some_and(|x| x) {
				event.handle(sim, ren, gui, input);
			}
		}
	}
}
