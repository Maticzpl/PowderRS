use wgpu::VertexAttribute;

use crate::rendering::render_utils::vertex_type::VertexType;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
	pub pos:        [f32; 2],
	pub tex_coords: [f32; 2],
}

impl VertexType<2> for Vert {
	const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
}
