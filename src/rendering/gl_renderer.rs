use std::cmp::max;
use std::intrinsics::{maxnumf32, minnumf32};
use std::rc::Rc;
use std::time::Instant;

use cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector2, Vector3};
use wgpu::{include_wgsl, PresentMode};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::rendering::gui::game_gui::GameGUI;
use crate::rendering::gui::immediate_mode::gui_renderer::Bounds;
use crate::sim::{Simulation, UI_MARGIN, WINH, WINW, XRES, YRES};

#[derive(Copy, Clone)]
pub struct Vert {
	pub pos:        [f32; 2],
	pub tex_coords: [f32; 2],
}

pub struct GLRenderer {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	pub(crate) size: PhysicalSize<u32>,
	pub window: Window,
	render_pipeline: wgpu::RenderPipeline,

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
					let dst = doc.get_element_by_id("wasm-example")?;
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
			present_mode: PresentMode::Immediate, // Use FiFo for vsync
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let (w, h) = (WINW as f32 / 2.0, WINH as f32 / 2.0);

		let square: [Vert; 4] = [
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

		let square_ind: [u32; 6] = [0, 1, 2, 0, 2, 3];

		// TODO: replace
		// let ind_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &square_ind)
		// 	.expect("Can't create index buffer");
		// let vert_buffer = VertexBuffer::new(&display, &square).expect("Can't create vert buffer");


		let shader = device.create_shader_module(include_wgsl!("./shaders/main.wgsl"));

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
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
				cull_mode: Some(wgpu::Face::Back),
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

		// let tex_filter = SamplerBehavior {
		// 	minify_filter: MinifySamplerFilter::Linear,
		// 	magnify_filter: MagnifySamplerFilter::Nearest,
		// 	..Default::default()
		// };

		(
			Self {
				window,
				surface,
				device,
				queue,
				config,
				size: win_size,
				render_pipeline,

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
		let camera_matrix = self.proj_matrix * self.view_matrix * self.model_matrix;
		/* //TODO: rewrite
		// // Generate texture
		// // self.texture = Texture2d::empty_with_format(&self.display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, XRES as u32, YRES as u32).expect("Can't create texture");
		// let mut tex_data = vec![vec![(0u8, 0u8, 0u8, 0u8); XRES]; YRES];
		// let mut counter = 0;
		// for i in 0..sim.parts.len() {
		// 	if counter >= sim.get_part_count() {
		// 		break;
		// 	}
		// 	let pt = sim.get_part(i);
		// 	if pt.p_type != 0 {
		// 		let col = pt.get_type().col;
		// 		tex_data[pt.y as usize][pt.x as usize] = (col[0], col[1], col[2], pt.p_type as u8);
		// 		counter += 1;
		// 	}
		// }
		// self.draw_cursor(&mut tex_data, &gui);
		// self.texture.write(
		// 	Rect {
		// 		width:  XRES as u32,
		// 		height: YRES as u32,
		// 		bottom: 0,
		// 		left:   0,
		// 	},
		// 	tex_data,
		// );
		//
		// let uniforms = uniform! {
		// 	tex: glium::uniforms::Sampler(&self.texture, self.tex_filter),
		// 	pvm: <Matrix4<f32> as Into<[[f32;4];4]>>::into(camera_matrix),
		// 	grid: gui.grid_size as i32,
		// 	z: 0f32,
		// };
		//
		// let mut frame = self.display.draw();
		// frame.clear_color(1.0 / 255.0, 0.0, 0.0, 0.0);
		//
		// frame
		// 	.draw(
		// 		&self.vert_buffer,
		// 		&self.ind_buffer,
		// 		&self.program,
		// 		&uniforms,
		// 		&self.draw_params,
		// 	)
		// 	.expect("Draw error");
		//
		// gui.gui_root.borrow().draw(&mut gui.immediate_gui);
		// gui.immediate_gui.draw_queued(&self.display, &mut frame);
		*/

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
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
			render_pass.draw(0..3, 0..1);
		}

		// submit will accept anything that implements IntoIter
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
	// todo uncomment
	// fn draw_cursor(&self, tex_data: &mut Vec<Vec<(u8, u8, u8, u8)>>, gui: &GameGUI) {
	// 	for i in 0..gui.cursor.height {
	// 		let x = gui.cursor.left as usize;
	// 		let rx = (gui.cursor.left + gui.cursor.width - 1) as usize;
	// 		let y = (gui.cursor.bottom + i) as usize;
	// 		tex_data[y][x] = GLRenderer::blend_colors(tex_data[y][x], (255, 255, 255, 128), 0.5);
	// 		tex_data[y][rx] = GLRenderer::blend_colors(tex_data[y][rx], (255, 255, 255, 128), 0.5);
	// 	}
	// 	for i in 1..gui.cursor.width - 1 {
	// 		let x = (gui.cursor.left + i) as usize;
	// 		let ry = (gui.cursor.bottom + gui.cursor.height - 1) as usize;
	// 		let y = gui.cursor.bottom as usize;
	// 		tex_data[y][x] = GLRenderer::blend_colors(tex_data[y][x], (255, 255, 255, 128), 0.5);
	// 		tex_data[ry][x] = GLRenderer::blend_colors(tex_data[ry][x], (255, 255, 255, 128), 0.5);
	// 	}
	// }

	fn set_pixel(&mut self, _x: usize, _y: usize, _color: u32) {
		todo!()
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
