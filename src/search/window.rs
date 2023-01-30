#[derive(Clone)]
pub struct Window {
	pub alpha: i32,
	pub beta: i32
}

impl Window {
	pub fn new() -> Window {
		Window {
			alpha: Self::ALPHA_BOUND,
			beta: Self::BETA_BOUND
		}
	}

	pub fn is_pv_children(&self, score: i32) -> bool {
		self.alpha < score && score < self.beta
	}

	pub fn can_raise_alpha(&self, score: i32) -> bool {
		score > self.alpha
	}

	pub fn cutoff(&self) -> bool {
		self.beta >= self.alpha
	}

	pub fn is_pv(&self) -> bool {
		self.beta > self.alpha + 1
	}

	pub fn set_alpha(&mut self, alpha: i32) {
		self.alpha = alpha;
	}

	pub fn set_beta(&mut self, beta: i32) {
		self.beta = beta;
	}

	pub fn create_null(&self, around: i32) -> Window {
		Window {
			alpha: around,
			beta: around + 1
		}
	}

	pub fn flip(&self) -> Window {
		Window {
			alpha: -self.beta,
			beta: -self.alpha
		}
	}
}

impl Window {
	pub const ALPHA_BOUND: i32 = -i32::MAX;
	pub const BETA_BOUND: i32 = i32::MAX;
}