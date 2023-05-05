use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct GUIVert {
	pub pos:   [f32; 2],
	pub color: [f32; 4],
}
implement_vertex!(GUIVert, pos, color);
