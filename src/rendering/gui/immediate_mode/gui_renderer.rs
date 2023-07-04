use std::intrinsics::size_of;
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use wgpu::{BufferAddress, Color, CommandEncoder, Device, include_wgsl, RenderPass, TextureFormat, TextureView};
use wgpu::util::{DeviceExt, StagingBelt};
use wgpu_glyph::{BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, HorizontalAlign, Layout, Section, Text, VerticalAlign};
use wgpu_glyph::ab_glyph::FontRef;

use crate::rendering::gui::immediate_mode::gui_vert::GUIVert;
use crate::rendering::wgpu::core::Core;
use crate::rendering::wgpu::pipeline::{Pipeline, PipelineDescriptor, Shader, ShaderType};
use crate::rendering::wgpu::vertex_type::VertexType;
use crate::sim::{WINH, WINW};

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
		v_align: VerticalAlign,
	},
}

pub struct ImmediateGUI<'a> {
	font: GlyphBrush<(), FontRef<'a>>,
	rect_num: u32,
	pipeline: Pipeline,
	pub belt: StagingBelt,
	pub window_scale_ratio: Vector2<f32>,
}

impl ImmediateGUI<'_> {
	pub(crate) fn new(rendering_core: &Core) -> Self {
		// Font
		let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Regular.ttf");
		let font = FontRef::try_from_slice(ttf).unwrap();

		let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Bold.ttf");
		let bold = FontRef::try_from_slice(ttf).unwrap();

		let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&rendering_core.device, rendering_core.surface_format);
		let _bold_id = glyph_brush.add_font(bold);

		// Shapes
		let vertex_buffer = rendering_core.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("GUI Vertex Buffer"),
				contents: &[],
				usage: wgpu::BufferUsages::VERTEX,
			}
		);
		let index_buffer = rendering_core.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("GUI Index Buffer"),
				contents: &[],
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		let shaders = rendering_core.device.create_shader_module(include_wgsl!("../../shaders/gui.wgsl"));
		let vertex_desc = &[GUIVert::description()];
		let vert = Shader {
			module: &shaders,
			entry: "vs_main",
			shader_type: ShaderType::Vertex(vertex_desc)
		};
		let frag = Shader {
			module: &shaders,
			entry: "fs_main",
			shader_type: ShaderType::Fragment
		};

		let gui_pipeline = Pipeline::new(PipelineDescriptor {
			device: &rendering_core.device,
			name: "GUI",
			shaders: vec![vert, frag],
			uniform_defaults: GUIUniforms { transform: Matrix4::identity().into() },
			vert_buffer: vertex_buffer,
			vert_num: 0,
			ind_buffer: index_buffer,
			bindings: vec![],
			bindings_layout: vec![],
			format: rendering_core.surface_format,
		}).unwrap();

		Self {
			font: glyph_brush,
			rect_num: 0,
			window_scale_ratio: Vector2::new(1.0, 1.0),
			pipeline: gui_pipeline,
			belt: StagingBelt::new(1024)
		}
	}

	pub fn queue_text(
		&mut self,
		text: &str,
		mut pos: Vector2<f32>,
		bounds: Bounds,
		font_size: f32,
		color: Option<Color>,
		font: Option<FontId>,
	) {
		let color = match color {
			Some(c) => c,
			None => Color::WHITE
		};
		let color = [color.r as f32, color.g as f32, color.b as f32, color.a as f32]; // This is so stupid

		// pos.x *= self.window_scale_ratio.x;
		// pos.y *= self.window_scale_ratio.y;

		let mut section = Section::default()
			.add_text(
				Text::new(text)
					.with_scale(font_size * self.window_scale_ratio.y)
					.with_color(color)
					.with_font_id(font.unwrap_or(FontId(0))),
			)
			.with_screen_position((pos.x, pos.y));

		let (size, h_align, v_align) = match bounds {
			Bounds::Box {
				size,
				h_align,
				v_align,
			} => (size, h_align, v_align),
			Bounds::None => {
				if let Some(size) = self.font.glyph_bounds(section.clone()) {
					(
						Vector2::new(size.width(), size.height()),
						HorizontalAlign::Left,
						VerticalAlign::Top,
					)
				} else {
					(
						Vector2::new(0f32, 0f32),
						HorizontalAlign::Left,
						VerticalAlign::Top,
					)
				}
			}
		};

		let layout = Layout::Wrap {
			h_align,
			v_align,
			line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
		};

		self.font
			.queue(section.with_bounds(size).with_layout(layout));
	}

	pub fn queue_rect(&mut self, rendering_core: &Core, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {
		let w_only = Vector2::from([size.x, 0.0]);
		let h_only = Vector2::from([0.0, size.y]);

		let vert_off = self.rect_num as usize * 4;

		rendering_core.queue.write_buffer(
			&self.pipeline.vert_buffer,
			(vert_off * size_of::<GUIVert>()) as BufferAddress,
			bytemuck::cast_slice(&[
				GUIVert {
					pos:   pos.into(),
					color: [color.r as f32, color.g  as f32, color.b  as f32, color.a  as f32],
				},
				GUIVert {
					pos:   (pos + w_only).into(),
					color: [color.r as f32, color.g  as f32, color.b  as f32, color.a  as f32],
				},
				GUIVert {
					pos:   (pos + size).into(),
					color: [color.r as f32, color.g  as f32, color.b  as f32, color.a  as f32],
				},
				GUIVert {
					pos:   (pos + h_only).into(),
					color: [color.r as f32, color.g  as f32, color.b  as f32, color.a  as f32],
				}
			])
		);

		let off = self.rect_num as usize * 6;
		rendering_core.queue.write_buffer(
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

	pub fn draw_queued<'a>(&'a mut self, rendering_core: &mut Core, mut render_pass: RenderPass<'a>) {
		let (w, h) = (rendering_core.window_size.width, rendering_core.window_size.height);
		self.window_scale_ratio = Vector2::from([w as f32 / WINW as f32, h as f32 / WINH as f32]);

		self.pipeline.vert_num = (self.rect_num * 6) as usize;

		// Shapes transform
		let transform = <Matrix4<f32> as Into<[[f32;4];4]>>::into(
				Matrix4::from_nonuniform_scale(2.0/WINW as f32 , -2.0/WINH as f32, 1.0) *
				Matrix4::from_translation(Vector3::from([-(WINW as f32/2.0), -(WINH as f32/2.0), 0.0]))
			);
		let uniforms = GUIUniforms {
			transform
		};

		rendering_core.queue.write_buffer(&self.pipeline.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

		self.pipeline.draw(&mut render_pass);
	}

	pub fn finish_drawing<'a>(&mut self, rendering_core: &mut Core, view: &TextureView, encoder: &mut CommandEncoder) {
		let (w, h) = (rendering_core.window_size.width, rendering_core.window_size.height);
		self.font.draw_queued(&rendering_core.device, &mut self.belt, encoder, &view, w, h).unwrap();

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
