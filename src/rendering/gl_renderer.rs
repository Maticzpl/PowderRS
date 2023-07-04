use std::cmp::max;
use std::intrinsics::{floorf32, maxnumf32, minnumf32, size_of};
use std::rc::Rc;

use cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector2, Vector3};
use cgmath::num_traits::abs;
use instant::Instant;
use wgpu::{BindGroup, Buffer, Extent3d, ImageCopyTexture, ImageDataLayout, include_wgsl, Origin3d, PresentMode, Sampler, TextureAspect, TextureView, VertexBufferLayout};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::gui::immediate_mode::gui_renderer::Bounds;
use crate::rendering::texture_data::TextureData;
use crate::rendering::vert::Vert;
use crate::rendering::wgpu::pipeline::{Pipeline, Shader, ShaderType};
use crate::rendering::wgpu::vertex_type::VertexType;
use crate::sim::{Simulation, UI_MARGIN, WINH, WINW, XRES, YRES};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.0,
	0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
	mat: [[f32; 4]; 4],
	z: f32,
	grid: u32,
	padding: f64, // Dummy variable for padding
	//Buffer is bound with size 72 where the shader expects 80 in group[1] compact index 0
}

pub struct GLRenderer {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	pub(crate) size: PhysicalSize<u32>,
	pub window: Window,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	screen_texture: wgpu::Texture,
	screen_texture_size: Extent3d,
	screen_texture_view: TextureView,
	screen_texture_bind_group: BindGroup,
	uniform_bind_group: BindGroup,
	uniform_buffer: Buffer,

	frame_start: Instant,
	timers:      [Instant; 4],
	perf_sum:    [u128; 3],
	fps_sum:     f64,
	samples:     u32,

	camera_zoom: f32,
	camera_pan:  Vector2<f32>,

	proj_matrix:  Matrix4<f32>,
	view_matrix:  Matrix4<f32>,
	model_matrix: Matrix4<f32>,
}

impl GLRenderer {
	pub async fn new() -> (Self, EventLoop<()>) {
		let win_size = PhysicalSize::new(WINW as u32, WINH as u32);

		cfg_if::cfg_if! {
			if #[cfg(target_arch = "wasm32")] {
				std::panic::set_hook(Box::new(console_error_panic_hook::hook));
				console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
			} else {
				env_logger::init();
			}
		}

		let event_loop = EventLoop::new();
		let window = WindowBuilder::new().build(&event_loop).unwrap();
		window.set_inner_size(win_size);
		window.set_title("PowderRS");
		window.set_resizable(true);
		window.set_transparent(false); // (;

		#[cfg(target_arch = "wasm32")]
		{
			use winit::platform::web::WindowExtWebSys;
			web_sys::window()
				.and_then(|win| win.document())
				.and_then(|doc| {
					let dst = doc.get_element_by_id("powderrs")?;
					let canvas = web_sys::Element::from(window.canvas());
					dst.append_child(&canvas).ok()?;
					Some(())
				})
				.expect("Couldn't append canvas to document body.");
		}

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			dx12_shader_compiler: Default::default(),
		});

		let surface = unsafe { instance.create_surface(&window) }.unwrap();

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::downlevel_webgl2_defaults()
				} else {
					wgpu::Limits::default()
				},
				label: None,
			},
			None,
		).await.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = surface_caps.formats.iter()
			.copied()
			.find(|f| f.is_srgb())
			.unwrap_or(surface_caps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: win_size.width,
			height: win_size.height,
			present_mode: PresentMode::Fifo,
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let (w, h) = (WINW as f32 / 2.0, WINH as f32 / 2.0);

		let square: &[Vert] = &[
			Vert {
				pos:        [-w as f32, h as f32],
				tex_coords: [0f32, 1f32],
			},
			Vert {
				pos:        [w as f32, h as f32],
				tex_coords: [1f32, 1f32],
			},
			Vert {
				pos:        [w as f32, -h as f32],
				tex_coords: [1f32, 0f32],
			},
			Vert {
				pos:        [-w as f32, -h as f32],
				tex_coords: [0f32, 0f32],
			},
		];

		let square_ind: &[u32] = &[0, 1, 2, 0, 2, 3];

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(square),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(square_ind),
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		let shader = device.create_shader_module(include_wgsl!("./shaders/main.wgsl"));

		let texture_size = wgpu::Extent3d {
			width: WINW as u32,
			height: WINH as u32,
			depth_or_array_layers: 1,
		};

		let screen_texture = device.create_texture(
			&wgpu::TextureDescriptor {
				size: texture_size,
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Rgba8Unorm,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				label: Some("screen_texture"),
				view_formats: &[],
			}
		);
		let screen_texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texture_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float {filterable: false},
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			}
		);

		let screen_texture_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &texture_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&screen_texture_view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&sampler),
					}
				],
				label: Some("texture_bind_group"),
			}
		);

		let proj_matrix: Matrix4<f32> = cgmath::ortho(-w, w, h, -h, -1.0, 1.0);
		let model_matrix: Matrix4<f32> = Matrix4::from_translation(Vector3 {
			x: -((UI_MARGIN / 2) as f32),
			y: -((UI_MARGIN / 2) as f32),
			z: 0.0,
		}) * Matrix4::from_nonuniform_scale(
			XRES as f32 / WINW as f32,
			YRES as f32 / WINH as f32,
			1.0,
		);

		let temp_val = Uniforms {
			mat: (proj_matrix * model_matrix * OPENGL_TO_WGPU_MATRIX).into(),
			z: 0.0,
			grid: 0,
			padding: 0f64,
		};

		let uniform_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Uniform Buffer"),
				contents: bytemuck::cast_slice(&[temp_val]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}
			],
			label: Some("uniform_bind_group_layout"),
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &uniform_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: uniform_buffer.as_entire_binding(),
				}
			],
			label: Some("uniform_bind_group"),
		});

		// Own abstraction test TODO: review -------------------------------------------------------
		let vertex_desc = &[Vert::description()];
		let vert = Shader {
			module: &shader,
			entry: "vs_main",
			shader_type: ShaderType::Vertex(vertex_desc)
		};
		let frag = Shader {
			module: &shader,
			entry: "fs_main",
			shader_type: ShaderType::Fragment
		};

		let test_pipeline = Pipeline::new(
			&device, "Rendering",
			vec![vert, frag], config.format,
			Uniforms {mat: Matrix4::identity().into(), grid: 0, z: 0.0, padding: 0.0}
		);
		// Own abstraction end TODO: review --------------------------------------------------------

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
				push_constant_ranges: &[],
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[
					Vert::description()
				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: None,
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		(
			Self {
				window,
				surface,
				device,
				queue,
				config,
				size: win_size,
				render_pipeline,
				vertex_buffer,
				index_buffer,
				screen_texture,
				screen_texture_size: texture_size,
				screen_texture_view,
				screen_texture_bind_group,
				uniform_buffer,
				uniform_bind_group,

				camera_zoom: 1.0,
				camera_pan: Vector2::from([0.0, 0.0]),

				proj_matrix,
				view_matrix: Matrix4::identity(),
				model_matrix,

				frame_start: Instant::now(),
				timers: [Instant::now(); 4],
				fps_sum: 0.0,
				samples: 0,
				perf_sum: [0; 3],
			},
			event_loop,
		)
	}
	pub fn render(&mut self, sim: &Simulation/*, gui: &mut GameGUI*/) -> Result<(), wgpu::SurfaceError> { // todo uncomment
		// FPS counter
		let dt = self.frame_start.elapsed().as_micros();

		self.fps_sum += 1000000f64 / dt as f64;
		self.samples += 1;

		//gui.fps_displ.borrow_mut().fps = self.fps_sum as f32 / self.samples as f32; //todo uncomment

		if self.timers[0].elapsed().as_millis() >= 1000 {
			self.perf_sum = [0; 3];
			self.fps_sum = 0.0;
			self.samples = 0;
			self.timers[0] = Instant::now();
		}
		self.frame_start = Instant::now();

		// WGPU stuff
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});


		// Adjust size
		let (ww, wh) = (self.size.width, self.size.height);
		let mut window_size = Vector2::new(ww as f32 / WINW as f32, wh as f32 / WINH as f32);
		window_size = window_size / minnumf32(window_size.x, window_size.y) as f32;

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
		let camera_matrix = Uniforms {
			mat: (OPENGL_TO_WGPU_MATRIX * self.proj_matrix * self.view_matrix * self.model_matrix).into(),
			z: 0.0,
			grid: 0, //gui.grid_size as i32
			padding: 0f64,
		};
		self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[camera_matrix]));

		// Generate texture
		let mut tex_data = TextureData::new(XRES, YRES);
		let mut counter = 0;
		for i in 0..sim.parts.len() {
			if counter >= sim.get_part_count() {
				break;
			}
			let pt = sim.get_part(i);
			if pt.p_type != 0 {
				let col = pt.get_type().col;
				tex_data.set_pixel(pt.x as usize, pt.y as usize, (col[0], col[1], col[2], pt.p_type as u8));
				counter += 1;
			}
		}

		//self.draw_cursor(&mut tex_data, &gui);

		self.queue.write_texture(
			ImageCopyTexture{
				texture: &self.screen_texture,
				aspect: TextureAspect::All,
				origin: Origin3d::ZERO,
				mip_level: 0
			},
			tex_data.as_slice(),
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * WINW as u32),
				rows_per_image: Some(WINH as u32),
			},
			self.screen_texture_size
		);

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.0,
							g: 0.0,
							b: 0.0,
							a: 1.0, // if window is transparent remember to make this 0
						}),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.screen_texture_bind_group, &[]);
			render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
			render_pass.draw_indexed(0..6, 0, 0..1);

			// gui.gui_root.borrow().draw(&mut gui.immediate_gui);
			// gui.immediate_gui.draw_queued(&self.display, &mut frame);
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
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

	// TODO: Move to gui?
	fn draw_cursor(&self, tex_data: &mut TextureData, gui: &GameGUI) {
		let width = (gui.cursor.max.x - gui.cursor.min.x) as usize;
		let height = (gui.cursor.max.y - gui.cursor.min.y) as usize;

		for i in 0..(height) {
			let x = gui.cursor.min.x as usize;
			let rx = gui.cursor.min.x as usize + width - 1;
			let y = gui.cursor.min.y as usize + i;
			tex_data.set_pixel(x, y,  GLRenderer::blend_colors(tex_data.get_pixel(x,  y), (255, 255, 255, 128), 0.5));
			tex_data.set_pixel(rx, y, GLRenderer::blend_colors(tex_data.get_pixel(rx, y), (255, 255, 255, 128), 0.5));
		}
		for i in 1..width - 1 {
			let x = gui.cursor.min.x as usize + i;
			let ry = gui.cursor.min.y as usize + height - 1;
			let y = gui.cursor.min.y as usize;
			tex_data.set_pixel(x, y,  GLRenderer::blend_colors(tex_data.get_pixel(x, y ), (255, 255, 255, 128), 0.5));
			tex_data.set_pixel(x, ry, GLRenderer::blend_colors(tex_data.get_pixel(x, ry), (255, 255, 255, 128), 0.5));
		}
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
