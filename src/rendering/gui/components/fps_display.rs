use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::Vector2;
use instant::Instant;
use wgpu::Color;
use wgpu_glyph::FontId;

use crate::define_component;
use crate::rendering::gui::components::label::Label;
use crate::rendering::gui::components::ComponentAlignment::LeftTop;
use crate::rendering::gui::components::{Component, ComponentAlignment, ComponentBase};
use crate::rendering::gui::immediate_mode::gui_renderer::{Bounds, ImmediateGUI};

pub struct FPSDisplay {
	base: ComponentBase,

	pub time_since_frame: Instant,
	pub time_since_tick:  Instant,

	pub fps: f32,
	pub tps: f32,

	fps_label: Option<Rc<RefCell<Label>>>,
	tps_label: Option<Rc<RefCell<Label>>>,
}

impl FPSDisplay {
	pub fn new(
		parent: Weak<RefCell<dyn Component>>,
		mut gui: &mut ImmediateGUI,
	) -> Rc<RefCell<Self>> {
		let mut base = ComponentBase::new(parent);
		base.set_size(Vector2::new(110.0, 100.0));

		let root = Rc::new(RefCell::new(Self {
			base,
			time_since_frame: Instant::now(),
			time_since_tick: Instant::now(),
			fps: 0.0,
			tps: 0.0,
			fps_label: None,
			tps_label: None,
		}));

		// FPS
		let weak = Rc::downgrade(&root);

		let mut fps_label = Label::new(
			"FPS",
			40.0,
			Color::WHITE,
			FontId(0),
			Bounds::None,
			Vector2::new(0.0, 40.0),
			gui,
			weak,
		);
		fps_label.set_alignment(LeftTop);

		let fps_label = Rc::new(RefCell::new(fps_label));
		root.borrow_mut()
			.add_child(Rc::clone(&fps_label) as Rc<RefCell<dyn Component>>);
		root.borrow_mut().fps_label = Some(fps_label);

		// TPS
		let weak = Rc::downgrade(&root);

		let mut tps_label = Label::new(
			"TPS",
			40.0,
			Color::WHITE,
			FontId(0),
			Bounds::None,
			Vector2::new(0.0, 0.0),
			gui,
			weak,
		);
		tps_label.set_alignment(LeftTop);

		let tps_label = Rc::new(RefCell::new(tps_label));
		root.borrow_mut()
			.add_child(Rc::clone(&tps_label) as Rc<RefCell<dyn Component>>);
		root.borrow_mut().tps_label = Some(tps_label);

		root
	}
}

define_component! { FPSDisplay,
	fn draw(&self, gui: &mut ImmediateGUI) {
		let fps = self.fps_label.as_ref().unwrap();
		let tps = self.tps_label.as_ref().unwrap();
		fps.borrow_mut().set_text(format!("FPS: {:.2}", self.fps).as_str(), gui);
		tps.borrow_mut().set_text(format!("TPS: {:.2}", self.tps).as_str(), gui);
		fps.borrow_mut().set_offset(Vector2::new(0.0, 40.0 * gui.window_scale_ratio.get().y));

		self.base.draw(gui);
	}
}
