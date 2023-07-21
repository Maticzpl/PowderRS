// This directory contains some **very simple** abstractions for wgpu (just for my use case)
// That is because I hate how much boilerplate you have to write for simple stuff :P
pub mod core;
pub mod pipeline;
pub mod texture;
pub mod vertex_type;

pub use pipeline::{Pipeline, PipelineDescriptor, Shader, ShaderType};
pub use texture::Texture;
pub use vertex_type::VertexType;

pub use self::core::Core;
