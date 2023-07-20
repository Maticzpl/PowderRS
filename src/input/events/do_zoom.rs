use cgmath::{Matrix4, Vector3};
use winit::event::VirtualKeyCode;

use crate::input::event_handling::InputData;
use crate::input::events::input_event::{AnyKey, InputEvent, KeyEvent, KeyState};
use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::renderer::Renderer;
use crate::sim::{Simulation, WINH, WINW};

pub struct DoZoom {}

impl InputEvent for DoZoom {
	fn get_name(&self) -> String {
		String::from("DoZoom")
	}

	fn default_keys(&self) -> Vec<KeyEvent> {
		vec![KeyEvent {
			key:              AnyKey::Keyboard(VirtualKeyCode::LControl),
			state:            KeyState::Held,
			combine_previous: None,
		}]
	}

	fn handle(
		&self,
		_sim: &mut Simulation,
		ren: &mut Renderer,
		_gui: &mut GameGUI,
		input: &mut InputData,
	) {
		if input.scroll != 0.0 {
			let prev_zoom = ren.get_zoom(); // TODO: Remove signum here, fix scroll bug
			let change = input.scroll.signum() / 10.0 * (ren.get_zoom() * 2.0);
			let mut zoom = ren.get_zoom() + change;
			zoom = zoom.clamp(1.0, 15.0);
			ren.set_zoom(zoom);

			#[rustfmt::skip]
                let res =
                    Matrix4::from_translation( Vector3 { x: (WINW / 2) as f32, y: (WINH / 2) as f32, z: 0.0 }) *
                    Matrix4::from_translation(-Vector3 { x: ren.get_pan().x,   y: ren.get_pan().y,	 z: 0.0 }) *
                    Matrix4::from_scale(prev_zoom / zoom) *
                    Matrix4::from_translation( Vector3 { x: ren.get_pan().x,   y: ren.get_pan().y,	 z: 0.0 }) *
                    Matrix4::from_translation(-Vector3 { x: (WINW / 2) as f32, y: (WINH / 2) as f32, z: 0.0 }) *
                    input.mouse_pos_vector;

			ren.set_pan(ren.get_pan() + (res - input.mouse_pos_vector).truncate().truncate());

			input.scroll = 0f32; // Capture scroll
		}
	}
}
