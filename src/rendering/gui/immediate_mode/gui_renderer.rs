use cgmath::{Matrix4, Vector2, Vector3};
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;
use glium::{uniform, Display, DrawParameters, IndexBuffer, Program, Rect, Surface, VertexBuffer};
use glium_glyph::glyph_brush::ab_glyph::FontRef;
use glium_glyph::glyph_brush::{
	BuiltInLineBreaker, Color, FontId, GlyphCruncher, GlyphPositioner, HorizontalAlign, Layout,
	Section, SectionGeometry, Text, VerticalAlign,
};
use glium_glyph::{GlyphBrush, GlyphBrushBuilder};

use crate::rendering::gui::immediate_mode::gui_vert::GUIVert;
use crate::sim::{WINH, WINW};

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
	font: GlyphBrush<'a, FontRef<'a>>,
	ind_buffer: IndexBuffer<u32>,
	vert_buffer: VertexBuffer<GUIVert>,
	program: Program,
	rect_num: u32,
	pub window_scale_ratio: Vector2<f32>,
}

impl ImmediateGUI<'_> {
	pub(crate) fn new(display: &Display) -> Self {
		let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Regular.ttf");
		let font = FontRef::try_from_slice(ttf).unwrap();

		let ttf: &[u8] = include_bytes!("../../../../ChakraPetch-Bold.ttf");
		let bold = FontRef::try_from_slice(ttf).unwrap();

		let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(display);
		let _bold_id = glyph_brush.add_font(bold);

		let cap = 1024usize;
		let ind_buffer: IndexBuffer<u32> = IndexBuffer::empty(
			display,
			PrimitiveType::TrianglesList,
			(cap as f32 * (6.0 / 4.0)) as usize,
		)
		.expect("Can't create GUI index buffer");

		let vert_buffer: VertexBuffer<GUIVert> =
			VertexBuffer::empty(display, cap).expect("Can't create GUI vert buffer");

		let program = match Program::new(
			display,
			ProgramCreationInput::SourceCode {
				vertex_shader:   include_str!("../../shaders/gui.vert"),
				fragment_shader: include_str!("../../shaders/gui.frag"),

				tessellation_control_shader: None,
				tessellation_evaluation_shader: None,
				geometry_shader: None,
				transform_feedback_varyings: None,
				outputs_srgb: false,
				uses_point_size: false,
			},
		) {
			Ok(res) => res,
			Err(e) => {
				panic!("{}", e.to_string().replace("\\n", "\n"));
			}
		};

		return Self {
			// fonts: HashMap::from([
			//     ("Regular".to_owned(), FontId(0)),
			//     ("Bold".to_owned(), bold_id)
			// ]),
			program,
			font: glyph_brush,
			ind_buffer,
			vert_buffer,
			rect_num: 0,
			window_scale_ratio: Vector2::new(1.0, 1.0),
		};
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
		// pos.x *= self.window_scale_ratio.x;
		// pos.y *= self.window_scale_ratio.y;

		let mut section = Section::default()
			.add_text(
				Text::new(text)
					.with_scale(font_size * self.window_scale_ratio.y)
					.with_color(color.unwrap_or(Color::from([1.0, 1.0, 1.0, 1.0])))
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

	pub fn queue_rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Color) {
		let w_only = Vector2::from([size.x, 0.0]);
		let h_only = Vector2::from([0.0, size.y]);

		let vert_off = self.rect_num as usize * 4;
		self.vert_buffer.map()[vert_off + 0] = GUIVert {
			pos:   pos.into(),
			color: color.map(|v| v),
		};
		self.vert_buffer.map()[vert_off + 1] = GUIVert {
			pos:   (pos + w_only).into(),
			color: color.map(|v| v),
		};
		self.vert_buffer.map()[vert_off + 2] = GUIVert {
			pos:   (pos + size).into(),
			color: color.map(|v| v),
		};
		self.vert_buffer.map()[vert_off + 3] = GUIVert {
			pos:   (pos + h_only).into(),
			color: color.map(|v| v),
		};

		let off = self.rect_num as usize * 6;
		self.ind_buffer.map()[off + 0] = (vert_off) as u32;
		self.ind_buffer.map()[off + 1] = (vert_off + 1) as u32;
		self.ind_buffer.map()[off + 2] = (vert_off + 2) as u32;
		self.ind_buffer.map()[off + 3] = (vert_off) as u32;
		self.ind_buffer.map()[off + 4] = (vert_off + 2) as u32;
		self.ind_buffer.map()[off + 5] = (vert_off + 3) as u32;

		self.rect_num += 1;
	}

	pub fn draw_queued(&mut self, display: &Display, frame: &mut impl Surface) {
		let (w, h) = display.get_framebuffer_dimensions();
		self.window_scale_ratio = Vector2::from([w as f32 / WINW as f32, h as f32 / WINH as f32]);

		// Shapes transform
		let uniforms = uniform! {
			transform: <Matrix4<f32> as Into<[[f32;4];4]>>::into(
				Matrix4::from_nonuniform_scale(2.0/WINW as f32 , -2.0/WINH as f32, 1.0) *
				Matrix4::from_translation(Vector3::from([-(WINW as f32/2.0), -(WINH as f32/2.0), 0.0]))
			)
		};

		frame
			.draw(
				&self.vert_buffer,
				&self.ind_buffer,
				&self.program,
				&uniforms,
				&DrawParameters::default(),
			)
			.expect("GUI Draw error");

		self.font.draw_queued(display, frame);
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
