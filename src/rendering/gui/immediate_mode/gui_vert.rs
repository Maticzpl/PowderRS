use wgpu::VertexAttribute;
use crate::rendering::wgpu::vertex_type::VertexType;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GUIVert {
	pub pos:   [f32; 2],
	pub color: [f32; 4],
}

impl VertexType<2> for GUIVert {
	const ATTRIBS: [VertexAttribute; 2] =
		wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
}