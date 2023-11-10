use std::cell::RefCell;
use std::rc::Rc;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use instant::Instant;
use wgpu::util::DeviceExt;
use wgpu::{
	include_wgsl, ImageCopyTexture, ImageDataLayout, Origin3d, ShaderStages, TextureAspect,
	TextureFormat, TextureUsages
};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;

use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::render_utils;
use crate::rendering::render_utils::VertexType;
use crate::rendering::texture_data::TextureData;
use crate::rendering::timing::Timing;
use crate::rendering::vert::Vert;
use crate::simulation::sim::{Simulation, WINH, WINW, XRES, YRES};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
	1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0
);

// At 0 it's 1 gui pixel = 1 sim pixel at default camera pos and zoom
const GUI_PIXELATION: f32 = 0.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
	mat:     [[f32; 4]; 4],
	gui_mat: [[f32; 4]; 4],
	z:       f32,
	grid:    u32,
	padding: f64 /* Buffer is bound with size 136 where the shader expects 144 in group[0] compact index 0 */
}

pub struct Renderer {
	pub rendering_core: Rc<RefCell<render_utils::Core>>,
	pub pipeline:       render_utils::Pipeline,
	screen_texture:     render_utils::Texture,
	gui_texture:        render_utils::Texture,

	pub timings: Timing,

	window_scale_factor: Vector2<f32>, // TODO: maybe find a better name
	camera_zoom:         f32,
	camera_pan:          Vector2<f32>,

	proj_matrix:  Matrix4<f32>,
	view_matrix:  Matrix4<f32>,
	model_matrix: Matrix4<f32>
}

impl Renderer {
	pub async fn new() -> (Self, EventLoop<()>) {
		let event_loop = EventLoop::new().unwrap();
		let rendering_core = render_utils::Core::new(
			"PowderRS",
			PhysicalSize::new(WINW as u32, WINH as u32),
			&event_loop
		)
		.await;
		rendering_core.window.set_resizable(true);
		rendering_core.window.set_transparent(false); // (;

		let (w, h) = (WINW as f32 / 2.0, WINH as f32 / 2.0);

		let square: &[Vert] = &[
			Vert {
				pos:        [-w, h],
				tex_coords: [0f32, 1f32]
			},
			Vert {
				pos:        [w, h],
				tex_coords: [1f32, 1f32]
			},
			Vert {
				pos:        [w, -h],
				tex_coords: [1f32, 0f32]
			},
			Vert {
				pos:        [-w, -h],
				tex_coords: [0f32, 0f32]
			},
			// This needs to be duplicated for GUI
			Vert {
				pos:        [-w, h],
				tex_coords: [2f32, 1f32]
			},
			Vert {
				pos:        [w, h],
				tex_coords: [3f32, 1f32]
			},
			Vert {
				pos:        [w, -h],
				tex_coords: [3f32, 0f32]
			},
			Vert {
				pos:        [-w, -h],
				tex_coords: [2f32, 0f32]
			}
		];

		let square_ind: &[u32] = &[
			0, 1, 2, 0, 2, 3, // SIM rect
			4, 5, 6, 4, 6, 7 // GUI rect
		];

		let vertex_buffer =
			rendering_core
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label:    Some("Vertex Buffer"),
					contents: bytemuck::cast_slice(square),
					usage:    wgpu::BufferUsages::VERTEX
				});

		let index_buffer =
			rendering_core
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label:    Some("Index Buffer"),
					contents: bytemuck::cast_slice(square_ind),
					usage:    wgpu::BufferUsages::INDEX
				});

		let shader = rendering_core
			.device
			.create_shader_module(include_wgsl!("./shaders/main.wgsl"));

		let texture_size = wgpu::Extent3d {
			width: WINW as u32,
			height: WINH as u32,
			depth_or_array_layers: 1
		};

		let screen_texture = render_utils::Texture::new(
			&rendering_core.device,
			texture_size,
			TextureFormat::Rgba8UnormSrgb, // TODO: Make sure im doing srgb correctly
			TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			ShaderStages::FRAGMENT,
			"Screen"
		);

		let gui_texture = render_utils::Texture::new(
			&rendering_core.device,
			wgpu::Extent3d {
				width: (WINW as f32 / (1.0 + GUI_PIXELATION)) as u32,
				height: (WINH as f32 / (1.0 + GUI_PIXELATION)) as u32,
				depth_or_array_layers: 1
			},
			rendering_core.surface_format, // TextureFormat::Rgba8UnormSrgb,
			TextureUsages::TEXTURE_BINDING |
				TextureUsages::COPY_DST |
				TextureUsages::RENDER_ATTACHMENT,
			ShaderStages::FRAGMENT,
			"GUI"
		);

		let proj_matrix: Matrix4<f32> = cgmath::ortho(-w, w, h, -h, -1.0, 1.0);
		let model_matrix: Matrix4<f32> = Matrix4::from_nonuniform_scale(
			XRES as f32 / WINW as f32,
			YRES as f32 / WINH as f32,
			1.0
		);

		let temp_val = Uniforms {
			mat:     (proj_matrix * model_matrix * OPENGL_TO_WGPU_MATRIX).into(),
			gui_mat: (proj_matrix * model_matrix * OPENGL_TO_WGPU_MATRIX).into(),
			z:       0.0,
			grid:    0,
			padding: 0f64
		};

		let vertex_desc = &[Vert::description()];
		let vert = render_utils::Shader {
			module:      &shader,
			entry:       "vs_main",
			shader_type: render_utils::ShaderType::Vertex(vertex_desc)
		};
		let frag = render_utils::Shader {
			module:      &shader,
			entry:       "fs_main",
			shader_type: render_utils::ShaderType::Fragment
		};

		let pipeline = render_utils::Pipeline::new(render_utils::PipelineDescriptor {
			device:           &rendering_core.device,
			name:             "Rendering",
			shaders:          vec![vert, frag],
			uniform_defaults: temp_val,
			vert_buffer:      vertex_buffer,
			vert_num:         square_ind.len(),
			ind_buffer:       index_buffer,
			bindings:         vec![
				screen_texture.bind_group.clone(),
				gui_texture.bind_group.clone(),
			],
			bindings_layout:  vec![
				screen_texture.bind_group_layout.clone(),
				gui_texture.bind_group_layout.clone(),
			],
			format:           rendering_core.surface_format
		});

		(
			Self {
				rendering_core: Rc::new(RefCell::new(rendering_core)),
				pipeline: pipeline.unwrap(),

				screen_texture,
				gui_texture,

				window_scale_factor: Vector2::new(1.0, 1.0),
				camera_zoom: 1.0,
				camera_pan: Vector2::from([0.0, 0.0]),

				proj_matrix,
				view_matrix: Matrix4::identity(),
				model_matrix,

				timings: Timing::new()
			},
			event_loop
		)
	}
	pub fn render(
		&mut self,
		sim: &Simulation,
		gui: &mut GameGUI
	) -> Result<(), wgpu::SurfaceError> {
		let dt = self.timings.time_since_frame.elapsed().as_micros();
		let mut display = gui.fps_display.borrow_mut();
		display.fps = 1000000f64 / dt as f64;

		self.timings.time_since_frame = Instant::now();
		drop(display);

		let core = self.rendering_core.borrow();

		// Adjust size
		let (ww, wh) = (core.window_size.width, core.window_size.height);
		let mut window_size = Vector2::new(ww as f32 / WINW as f32, wh as f32 / WINH as f32);
		window_size = window_size / window_size.x.min(window_size.y);
		self.window_scale_factor = window_size;

		#[rustfmt::skip]
		let view_matrix =
			Matrix4::from_nonuniform_scale( 1.0 / window_size.x, 1.0 / window_size.y, 1.0) *
			Matrix4::from_scale(self.camera_zoom) *
			Matrix4::from_translation(Vector3 {
				x: self.camera_pan.x,
				y: self.camera_pan.y,
				z: 0.0,
			});

		self.view_matrix = view_matrix;
		let unifs = Uniforms {
			mat:     (OPENGL_TO_WGPU_MATRIX *
				self.proj_matrix * self.view_matrix *
				self.model_matrix)
				.into(),
			gui_mat: (OPENGL_TO_WGPU_MATRIX * self.proj_matrix).into(),
			z:       0.0,
			grid:    gui.grid_size,
			padding: 0f64
		};
		core.queue.write_buffer(
			&self.pipeline.uniform_buffer,
			0,
			bytemuck::cast_slice(&[unifs])
		);

		// Generate texture
		let mut tex_data = TextureData::new(XRES, YRES);
		let mut counter = 0;
		for i in 0..sim.parts.len() {
			if counter >= sim.get_part_count() {
				break;
			}
			let pt = sim.get_part(i);
			if pt.p_type != 0 {
				let col = pt.get_type(&sim.element_manager).col;
				tex_data.set_pixel(
					pt.x as usize,
					pt.y as usize,
					(col[0], col[1], col[2], pt.p_type as u8)
				);
				counter += 1;
			}
		}

		self.draw_cursor(&mut tex_data, gui);

		core.queue.write_texture(
			ImageCopyTexture {
				texture:   &self.screen_texture.texture,
				aspect:    TextureAspect::All,
				origin:    Origin3d::ZERO,
				mip_level: 0
			},
			tex_data.as_slice(),
			ImageDataLayout {
				offset:         0,
				bytes_per_row:  Some(4 * self.screen_texture.size.width),
				rows_per_image: Some(self.screen_texture.size.height)
			},
			self.screen_texture.size
		);

		// WGPU stuff This is a bit messy, well that's the price you pay not using unsafe rust :P
		gui.gui_root.borrow().draw(&mut gui.immediate_gui);
		drop(core);
		gui.immediate_gui
			.draw_to_texture(&self.gui_texture.texture)?;

		let core = self.rendering_core.borrow_mut();
		let mut encoder = core
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder")
			});

		let view = self.pipeline.create_window_view(&core)?;
		let mut render_pass = self
			.pipeline
			.begin_render_pass(&view, &mut encoder, false)?;

		self.pipeline.draw(&mut render_pass);

		drop(core);
		drop(render_pass);
		self.pipeline
			.submit_frame(&mut self.rendering_core.borrow_mut(), encoder, true);

		Ok(())
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		let mut core = self.rendering_core.borrow_mut();

		if new_size.width > 0 && new_size.height > 0 {
			core.window_size = new_size;
			core.config.width = new_size.width;
			core.config.height = new_size.height;
			core.surface.configure(&core.device, &core.config);
		}
	}

	fn blend_colors(col_a: (u8, u8, u8, u8), col_b: (u8, u8, u8, u8), t: f32) -> (u8, u8, u8, u8) {
		let mut col = col_a;
		let rt = 1.0 - t;
		col.0 = (col.0 as f32 * rt) as u8 + (col_b.0 as f32 * t) as u8;
		col.1 = (col.1 as f32 * rt) as u8 + (col_b.1 as f32 * t) as u8;
		col.2 = (col.2 as f32 * rt) as u8 + (col_b.2 as f32 * t) as u8;
		col.3 = u8::saturating_add(col.3, col_b.3);

		col
	}

	fn draw_cursor(&self, tex_data: &mut TextureData, gui: &GameGUI) {
		let width = (gui.cursor.1.x - gui.cursor.0.x) as usize;
		let height = (gui.cursor.1.y - gui.cursor.0.y) as usize;

		for i in 0..(height) {
			let x = gui.cursor.0.x as usize;
			let rx = gui.cursor.0.x as usize + width - 1;
			let y = gui.cursor.0.y as usize + i;
			tex_data.set_pixel(
				x,
				y,
				Renderer::blend_colors(tex_data.get_pixel(x, y), (255, 255, 255, 128), 0.4)
			);
			tex_data.set_pixel(
				rx,
				y,
				Renderer::blend_colors(tex_data.get_pixel(rx, y), (255, 255, 255, 128), 0.4)
			);
		}
		for i in 1..width - 1 {
			let x = gui.cursor.0.x as usize + i;
			let ry = gui.cursor.0.y as usize + height - 1;
			let y = gui.cursor.0.y as usize;
			tex_data.set_pixel(
				x,
				y,
				Renderer::blend_colors(tex_data.get_pixel(x, y), (255, 255, 255, 128), 0.4)
			);
			tex_data.set_pixel(
				x,
				ry,
				Renderer::blend_colors(tex_data.get_pixel(x, ry), (255, 255, 255, 128), 0.4)
			);
		}
	}

	pub fn get_window_scale_factor(&self) -> Vector2<f32> {
		self.window_scale_factor
	}

	pub fn get_zoom(&self) -> f32 {
		self.camera_zoom
	}

	pub fn get_pan(&self) -> Vector2<f32> {
		self.camera_pan
	}

	pub fn set_zoom(&mut self, zoom: f32) {
		self.camera_zoom = zoom;
	}

	pub fn set_pan(&mut self, pan: Vector2<f32>) {
		self.camera_pan = pan;
	}

	pub fn get_proj_matrix(&self) -> Matrix4<f32> {
		self.proj_matrix
	}

	pub fn get_view_matrix(&self) -> Matrix4<f32> {
		self.view_matrix
	}

	pub fn get_model_matrix(&self) -> Matrix4<f32> {
		self.model_matrix
	}

	pub fn set_proj_matrix(&mut self, matrix: Matrix4<f32>) {
		self.proj_matrix = matrix;
	}

	pub fn set_view_matrix(&mut self, matrix: Matrix4<f32>) {
		self.view_matrix = matrix;
	}

	pub fn set_model_matrix(&mut self, matrix: Matrix4<f32>) {
		self.model_matrix = matrix;
	}
}
