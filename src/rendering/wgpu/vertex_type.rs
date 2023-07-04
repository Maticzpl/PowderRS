use std::fmt::Debug;
use wgpu::{VertexBufferLayout};
use std::mem;

pub trait VertexType<const ATTRIB_NUM: usize>: Copy + Clone + Debug + bytemuck::Pod + bytemuck::Zeroable {
    const ATTRIBS: [wgpu::VertexAttribute; ATTRIB_NUM];

    fn description() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}