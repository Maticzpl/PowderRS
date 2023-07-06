// use crate::rendering::gui::game_gui::GameGUI;
// use crate::sim::Simulation;

pub mod gui;
pub mod renderer;
pub mod texture_data;
pub mod vert;
pub mod render_utils;

// borrow checker hates me
// pub trait CameraController {
// 	fn get_zoom(&self) -> f32;
// 	fn get_pan(&self) -> Vector2<f32>;
//
// 	fn set_zoom(&mut self, zoom: f32);
// 	fn set_pan(&mut self, pan: Vector2<f32>);
//
// 	fn get_proj_matrix(&self) -> Matrix4<f32>;
// 	fn get_view_matrix(&self) -> Matrix4<f32>;
// 	fn get_model_matrix(&self) -> Matrix4<f32>;
//
// 	fn set_proj_matrix(&mut self, matrix: Matrix4<f32>);
// 	fn set_view_matrix(&mut self, matrix: Matrix4<f32>);
// 	fn set_model_matrix(&mut self, matrix: Matrix4<f32>);
// }
//
// pub trait SimulationRenderer {
// 	fn render(&mut self, sim: &Simulation, gui: &mut GameGUI);
//
// 	fn set_pixel(&mut self, x: usize, y: usize, color: u32);
// }
