use std::cell::{Cell, RefCell};
use std::mem::size_of;
use std::rc::Rc;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use glyphon::{FontSystem, Metrics, SwashCache, TextAtlas, TextRenderer};
use wgpu::util::{DeviceExt, StagingBelt};
use wgpu::{
	include_wgsl, BufferAddress, Color, CommandEncoder, MultisampleState, RenderPass, SurfaceError,
	TextureFormat, TextureView, TextureViewDescriptor
};

use crate::rendering::gui::immediate_mode::gui_vert::GUIVert;
use crate::rendering::render_utils::core::Core;
use crate::rendering::render_utils::pipeline::{Pipeline, PipelineDescriptor, Shader, ShaderType};
use crate::rendering::render_utils::vertex_type::VertexType;
use crate::simulation::sim::{WINH, WINW};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GUIUniforms {
	transform: [[f32; 4]; 4]
}

#[derive(Copy, Clone)]
pub enum Bounds {
	None,
	Box {
		size:    Vector2<f32>,
		h_align: HorizontalAlign,
		v_align: VerticalAlign
	}
}

pub struct ImmediateGUI {
	font_system:   FontSystem,
	font_cache:    SwashCache,
	font_atlas:    TextAtlas,
	font_renderer: TextRenderer,
	font_buffer:   glyphon::Buffer,

	rect_num:               u32,
	pipeline:               Pipeline,
	rendering_core:         Rc<RefCell<Core>>,
	pub belt:               StagingBelt,
	pub window_scale_ratio: Cell<Vector2<f32>>
}

impl ImmediateGUI {
	pub(crate) fn new(rendering_core: Rc<RefCell<Core>>) -> Self {
		let core = rendering_core.borrow();

		// Font
		let font_system = FontSystem::new();
		let font_cache = SwashCache::new();
		let font_atlas = TextAtlas::new(&core.device, &core.queue, TextureFormat::Bgra8UnormSrgb);
		let font_renderer = TextRenderer::new(
			&mut font_atlas,
			&core.device,
			MultisampleState::default(),
			None
		);
		let font_buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

		font_buffer.set_size(
			&mut font_system,
			core.window_size.width as f32,
			core.window_size.height as f32
		);

		// let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Regular.ttf");
		// let font = FontRef::try_from_slice(ttf).unwrap();
		//
		// let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Bold.ttf");
		// let bold = FontRef::try_from_slice(ttf).unwrap();
		//
		// let mut glyph_brush =
		// 	GlyphBrushBuilder::using_font(font).build(&core.device, core.surface_format);
		// let _bold_id = glyph_brush.add_font(bold);

		const TRIG_CAP: usize = 1024usize;

		// Shapes
		let vertex_buffer = core
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label:    Some("GUI Vertex Buffer"),
				contents: bytemuck::cast_slice(
					vec![
						GUIVert {
							pos:   [0.0, 0.0],
							color: [0.0, 0.0, 0.0, 0.0]
						};
						TRIG_CAP
					]
					.as_slice()
				),
				usage:    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
			});
		let index_buffer = core
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label:    Some("GUI Index Buffer"),
				contents: bytemuck::cast_slice(vec![0u32; TRIG_CAP * 2].as_slice()),
				usage:    wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
			});

		let shaders = core
			.device
			.create_shader_module(include_wgsl!("../../shaders/gui.wgsl"));
		let vertex_desc = &[GUIVert::description()];
		let vert = Shader {
			module:      &shaders,
			entry:       "vs_main",
			shader_type: ShaderType::Vertex(vertex_desc)
		};
		let frag = Shader {
			module:      &shaders,
			entry:       "fs_main",
			shader_type: ShaderType::Fragment
		};

		let gui_pipeline = Pipeline::new(PipelineDescriptor {
			device:           &core.device,
			name:             "GUI",
			shaders:          vec![vert, frag],
			uniform_defaults: GUIUniforms {
				transform: Matrix4::identity().into()
			},
			vert_buffer:      vertex_buffer,
			vert_num:         0,
			ind_buffer:       index_buffer,
			bindings:         vec![],
			bindings_layout:  vec![],
			format:           core.surface_format
		})
		.unwrap();

		drop(core);

		Self {
			rect_num: 0,
			window_scale_ratio: Cell::new(Vector2::new(1.0, 1.0)),
			pipeline: gui_pipeline,
			belt: StagingBelt::new(1024),
			rendering_core,
			font_system,
			font_cache,
			font_atlas,
			font_renderer,
			font_buffer
		}
	}

	pub fn queue_text(
		&mut self,
		text: &str,
		pos: Vector2<f32>,
		bounds: Bounds,
		font_size: f32,
		color: Option<Color>,
		font: Option<FontId>
	) {
		let color = match color {
			Some(c) => c,
			None => Color::WHITE
		};
		let color = [
			color.r as f32,
			color.g as f32,
			color.b as f32,
			color.a as f32
		]; // This is so stupid

		let section = Section::default()
			.add_text(
				Text::new(text)
					.with_scale(font_size * self.window_scale_ratio.get().y)
					.with_color(color)
					.with_font_id(font.unwrap_or(FontId(0)))
			)
			.with_screen_position((pos.x, pos.y));

		let (size, h_align, v_align) = match bounds {
			Bounds::Box {
				size,
				h_align,
				v_align
			} => (size, h_align, v_align),
			Bounds::None => {
				if let Some(size) = self.font.glyph_bounds(section.clone()) {
					(
						Vector2::new(size.width(), size.height()),
						HorizontalAlign::Left,
						VerticalAlign::Top
					)
				}
				else {
					(
						Vector2::new(0f32, 0f32),
						HorizontalAlign::Left,
						VerticalAlign::Top
					)
				}
			}
		};

		let layout = Layout::Wrap {
			h_align,
			v_align,
			line_breaker: BuiltInLineBreaker::UnicodeLineBreaker
		};

		self.font
			.queue(section.with_bounds(size).with_layout(layout));
	}

	pub fn queue_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {
		let core = self.rendering_core.borrow();

		let w_only = Vector2::from([size.x, 0.0]);
		let h_only = Vector2::from([0.0, size.y]);

		let vert_off = self.rect_num as usize * 4;

		core.queue.write_buffer(
			&self.pipeline.vert_buffer,
			(vert_off * size_of::<GUIVert>()) as BufferAddress,
			bytemuck::cast_slice(&[
				GUIVert {
					pos:   pos.into(),
					color: [
						color.r as f32,
						color.g as f32,
						color.b as f32,
						color.a as f32
					]
				},
				GUIVert {
					pos:   (pos + w_only).into(),
					color: [
						color.r as f32,
						color.g as f32,
						color.b as f32,
						color.a as f32
					]
				},
				GUIVert {
					pos:   (pos + size).into(),
					color: [
						color.r as f32,
						color.g as f32,
						color.b as f32,
						color.a as f32
					]
				},
				GUIVert {
					pos:   (pos + h_only).into(),
					color: [
						color.r as f32,
						color.g as f32,
						color.b as f32,
						color.a as f32
					]
				}
			])
		);

		let off = self.rect_num as usize * 6;
		core.queue.write_buffer(
			&self.pipeline.ind_buffer,
			(off * size_of::<u32>()) as BufferAddress,
			bytemuck::cast_slice(&[
				vert_off as u32,
				(vert_off + 1) as u32,
				(vert_off + 2) as u32,
				vert_off as u32,
				(vert_off + 2) as u32,
				(vert_off + 3) as u32
			])
		);

		self.rect_num += 1;
	}

	pub fn draw_to_texture(&mut self, texture: &wgpu::Texture) -> Result<(), SurfaceError> {
		let view = texture.create_view(&TextureViewDescriptor {
			label:             Some("GUI Fullscreen texture view descriptor"),
			format:            None,
			dimension:         None,
			aspect:            Default::default(),
			base_mip_level:    0,
			mip_level_count:   None,
			base_array_layer:  0,
			array_layer_count: None
		});

		let core = self.rendering_core.borrow_mut();
		// Is this the correct way to do this?
		let mut encoder = core
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("GUI Render Encoder")
			});
		drop(core);

		let render_pass = self.pipeline.begin_render_pass(&view, &mut encoder, true)?;

		self.draw_queued(render_pass);
		self.finish_drawing(&view, &mut encoder);
		self.pipeline
			.submit_frame(&mut self.rendering_core.borrow_mut(), encoder, false);

		self.belt.recall();
		Ok(())
	}

	pub fn draw_queued<'a>(&'a self, mut render_pass: RenderPass<'a>) {
		let core = self.rendering_core.borrow();
		let (w, h) = (core.window_size.width, core.window_size.height);

		self.window_scale_ratio.set(Vector2::from([
			w as f32 / WINW as f32,
			h as f32 / WINH as f32
		]));
		self.pipeline.vert_num.set((self.rect_num * 6) as usize);

		// Shapes transform
		let transform = <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(
			Matrix4::from_nonuniform_scale(2.0 / WINW as f32, -2.0 / WINH as f32, 1.0) *
				Matrix4::from_translation(Vector3::from([
					-(WINW as f32 / 2.0),
					-(WINH as f32 / 2.0),
					0.0
				]))
		);
		let uniforms = GUIUniforms { transform };

		core.queue.write_buffer(
			&self.pipeline.uniform_buffer,
			0,
			bytemuck::cast_slice(&[uniforms])
		);

		self.pipeline.draw(&mut render_pass);
	}

	pub fn finish_drawing(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
		let core = self.rendering_core.borrow();

		let (w, h) = (core.window_size.width, core.window_size.height);
		self.font
			.draw_queued(&core.device, &mut self.belt, encoder, view, w, h)
			.unwrap();

		self.belt.finish();
		self.rect_num = 0;
	}

	pub fn measure_text(&mut self, text: &str, font_size: f32) -> Vector2<f32> {
		let size = self
			.font
			.glyph_bounds(Section::default().add_text(Text::new(text).with_scale(font_size)))
			.unwrap();

		Vector2::new(size.width(), size.height())
	}
}
