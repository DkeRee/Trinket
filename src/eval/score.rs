pub struct Score {
	pub mg: i32,
	pub eg: i32
}

macro_rules! S {
	($x:expr, $y:expr) => {
		Score::new($x, $y)
	};
}

impl Score {
	pub const fn new(mg: i32, eg: i32) -> Score {
		Score {
			mg: mg,
			eg: eg
		}
	}

	pub fn eval(&self, phase: i32) -> i32 {
		((self.mg * (Self::TOTAL_PHASE - phase)) + (self.eg * phase)) / Self::TOTAL_PHASE
	}
}

impl Score {
	const TOTAL_PHASE: i32 = 256;
}