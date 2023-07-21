use instant::Instant;

pub struct Timing {
	pub time_since_frame: Instant,
	pub time_since_tick:  Instant
}

impl Timing {
	pub(crate) fn new() -> Self {
		Self {
			time_since_frame: Instant::now(),
			time_since_tick:  Instant::now()
		}
	}
}
