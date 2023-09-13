use std::intrinsics::floorf32;

use rand::prelude::*;
use rust_bresenham::Bresenham;

use crate::simulation::elements::*;
use crate::simulation::Particle;

// TODO: find a good window / sim size
pub const WINW: usize = (1.0 * 720.0) as usize;
pub const WINH: usize = (1.0 * 480.0) as usize;
pub const XRES: usize = WINW;
pub const YRES: usize = WINH;
pub const XYRES: usize = XRES * YRES;

type ParticleHandle = usize;

pub struct Simulation {
	pub parts:           Box<[Particle]>,
	pub pmap:            Box<[Option<usize>]>,
	pub paused:          bool,
	pub element_manager: ElementManager,
	part_count:          usize
}
impl Simulation {
	pub fn new() -> Self {
		let p = vec![Particle::default(); XYRES * 2].into_boxed_slice();
		let pm = vec![None; XYRES].into_boxed_slice();

		Self {
			parts:           p,
			pmap:            pm,
			paused:          false,
			element_manager: ElementManager::new(),
			part_count:      0
		}
	}

	/// Adds particle
	pub fn add_part(&mut self, part: Particle) -> Option<ParticleHandle> {
		if part.p_type == 0 {
			return None;
		}
		if self.get_id(part.x as usize, part.y as usize).is_some() {
			return None;
		}

		// find first empty spot
		for i in 0..self.parts.len() {
			if self.parts[i].p_type == 0 {
				self.part_count += 1;
				self.pmap[part.x as usize + (part.y as usize * XRES)] = Some(i);
				self.parts[i] = part;
				return Some(i);
			}
		}
		None
	}

	pub fn kill_part(&mut self, id: ParticleHandle) -> Result<(), ()> {
		if id >= self.parts.len() || self.parts[id].p_type == 0 {
			return Err(());
		}

		self.pmap[self.parts[id].x as usize + (self.parts[id].y as usize * XRES)] = None;
		self.parts[id] = Particle::default();
		self.part_count -= 1;
		Ok(())
	}

	pub fn get_part(&self, id: ParticleHandle) -> &Particle {
		&self.parts[id]
	}

	pub fn get_part_mut(&mut self, id: ParticleHandle) -> &mut Particle {
		&mut self.parts[id]
	}

	pub fn get_part_count(&self) -> usize {
		self.part_count
	}

	pub fn get_pmap(&self, x: usize, y: usize) -> Option<&Particle> {
		if x >= XRES || y >= YRES {
			return None;
		}

		let val = self.pmap[x + (y * XRES)];
		val?;
		Some(&self.parts[val.unwrap()])
	}

	pub fn get_id(&self, x: usize, y: usize) -> Option<ParticleHandle> {
		if x >= XRES || y >= YRES {
			return None;
		}

		self.pmap[x + (y * XRES)]
	}

	pub fn update_p_map(&mut self) {
		let mut counter = 0;

		self.pmap.fill(None);
		for i in 0..self.parts.len() {
			if counter >= self.part_count {
				break;
			}

			if self.parts[i].p_type != 0 {
				let index = self.parts[i].x as usize + (self.parts[i].y as usize * XRES);
				self.pmap[index] = Some(i);
				counter += 1;
			}
		}
	}

	fn kill_out_of_bounds(&mut self, pt_id: ParticleHandle) -> bool {
		let pt = &self.parts[pt_id];
		let x = pt.x as isize;
		let y = pt.y as isize;

		if x < 0 || x >= XRES as isize || y < 0 || y >= YRES as isize {
			self.parts[pt_id] = Particle::default();
			self.part_count -= 1;
			return true;
		}
		false
	}

	fn move_to(&mut self, pt_id: ParticleHandle, x: f32, y: f32) {
		let pt = &mut self.parts[pt_id];
		self.pmap[pt.x as usize + pt.y as usize * XRES] = None;

		pt.x = x;
		pt.y = y;

		if self.kill_out_of_bounds(pt_id) {
			return;
		}

		let pt = &mut self.parts[pt_id];
		self.pmap[pt.x as usize + pt.y as usize * XRES] = Some(pt_id);
	}

	// Borrow checker doesn't like stuff that results in subframe (:
	// So we pass an index instead of a reference
	// Returns true if collided
	fn velocity_move(&mut self, pt_id: ParticleHandle) -> bool {
		let pt = &self.parts[pt_id];

		let vx = pt.vx;
		let vy = pt.vy;
		let px = pt.x as isize;
		let py = pt.y as isize;
		let rx = (pt.x + vx) as isize;
		let ry = (pt.y + vy) as isize;

		if px != rx || py != ry {
			let mut points: Vec<_> = Bresenham::new((px, py), (rx, ry)).collect();
			points.push((rx, ry));

			let mut prev = (px, py);
			for (x, y) in points {
				if self.get_pmap(x as usize, y as usize).is_some() {
					self.move_to(pt_id, prev.0 as f32, prev.1 as f32);

					self.parts[pt_id].vx = 0.0;
					self.parts[pt_id].vy = 0.0;
					return true;
				}

				prev = (x, y);
			}
		}

		self.move_to(pt_id, pt.x + vx, pt.y + vy);
		false
	}

	fn powder_move(&mut self, pt_id: ParticleHandle) {
		let pt = &self.parts[pt_id];
		// todo
		if random() {
			let pos = (pt.x as usize + 1, (pt.y + 1.0) as usize);
			if self.get_pmap(pos.0, pos.1).is_none() {
				self.move_to(pt_id, pos.0 as f32, pos.1 as f32);
			}
		}
		else {
			let pos = (pt.x as usize - 1, (pt.y + 1.0) as usize);
			if self.get_pmap(pos.0, pos.1).is_none() {
				self.move_to(pt_id, pos.0 as f32, pos.1 as f32);
			}
		}
	}

	pub fn step(&mut self) {
		for pt_id in 0..self.parts.len() {
			let part = &mut self.parts[pt_id];

			let behaviour = part.get_type(&self.element_manager).behaviour;

			match behaviour {
				ElementBehaviour::Skip => {}
				ElementBehaviour::Solid => {}
				ElementBehaviour::Powder | ElementBehaviour::Fluid | ElementBehaviour::Gas => {
					part.vy += 0.1; // Gravity
					if self.velocity_move(pt_id) {
						self.powder_move(pt_id);
					}
				}
			}
		}

		self.update_p_map();
	}
}
