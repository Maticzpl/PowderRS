pub mod gui;
mod render_utils;
pub mod renderer;
mod texture_data;
mod timing;
mod vert;

use cgmath::Vector2;

pub use crate::rendering::render_utils::core::Core;

pub type Rect = (Vector2<f32>, Vector2<f32>);
