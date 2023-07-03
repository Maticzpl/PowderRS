use std::cell::RefCell;
use std::rc::{Rc, Weak};

use cgmath::Vector2;
use wgpu_glyph::ab_glyph::Rect;

use crate::rendering::gui::components::label::Label;
use crate::rendering::gui::components::root::Root;
use crate::rendering::gui::components::Component;
use crate::rendering::gui::components::ComponentAlignment::{Center, LeftTop, MiddleTop};
use crate::rendering::gui::fps_display::FPSDisplay;
use crate::rendering::gui::immediate_mode::gui_renderer::{Bounds, ImmediateGUI};

pub struct GameGUI<'a> {
	pub immediate_gui: ImmediateGUI<'a>,
	pub grid_size:     u32,
	pub cursor:        Rect,
	pub brush_size:    u32,
	pub gui_root:      Rc<RefCell<dyn Component>>,

	pub fps_displ: Rc<RefCell<FPSDisplay>>,
}

pub struct Display {} // TODO: REMOVE THIS IS FOR TESTING

impl GameGUI<'_> {
	// pub(crate) fn new(display: Rc<Display>) -> Self { TODO: rewrite
	//
	// 	let mut gui = ImmediateGUI::new(&display);
	// 	let root = Rc::new(RefCell::new(Root::new(display))) as Rc<RefCell<dyn Component>>;
	//
	// 	let weak = Rc::downgrade(&root);
	// 	let fps_displ = FPSDisplay::new(weak, &mut gui);
	// 	root.borrow_mut()
	// 		.add_child(Rc::clone(&fps_displ) as Rc<RefCell<dyn Component>>);
	//
	// 	Self {
	// 		fps_displ,
	// 		immediate_gui: gui,
	// 		grid_size: 0,
	// 		cursor: Rect::default(),
	// 		brush_size: 5,
	// 		gui_root: root,
	// 	}
	// }
}