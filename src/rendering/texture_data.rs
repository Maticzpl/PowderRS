use cgmath::{vec2, Vector2};

pub struct TextureData {
	data: Vec<u8>,
	size: Vector2<usize>
}

impl TextureData {
	pub fn new(w: usize, h: usize) -> Self {
		Self {
			data: vec![0u8; w * h * 4],
			size: vec2(w, h)
		}
	}

	#[inline]
	pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
		let pos = (x + (y * self.size.x)) * 4;
		(
			self.data[pos],
			self.data[pos + 1],
			self.data[pos + 2],
			self.data[pos + 3]
		)
	}

	#[inline]
	pub fn set_pixel(&mut self, x: usize, y: usize, val: (u8, u8, u8, u8)) {
		let pos = (x + (y * self.size.x)) * 4;
		(
			self.data[pos],
			self.data[pos + 1],
			self.data[pos + 2],
			self.data[pos + 3]
		) = val;
	}

	#[inline]
	pub fn as_slice(&self) -> &[u8] {
		self.data.as_slice()
	}
}
