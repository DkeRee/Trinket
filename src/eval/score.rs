#[derive(Clone, Debug, PartialEq)]

pub struct Eval {
	pub score: i32,
	pub mate: bool
}

pub struct Score {
	mg: i32,
	eg: i32
}

macro_rules! S {
	($x:expr, $y:expr) => {
		Score::new($x, $y)
	};
}

impl Eval {
	pub fn new(score: i32, mate: bool) -> Eval {
		Eval {
			score: score,
			mate: mate
		}
	}
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
	//DRAW SCORE
	pub const DRAW: i32 = 0;

	//NUMBER TO BASE CHECKMATES OFF FROM
	pub const CHECKMATE_BASE: i32 = 30000;

	//NUMBER TO CHECK IF A POSITION IS A CHECKMATE. MAINLY USED IN TT.
	pub const CHECKMATE_DEFINITE: i32 = 20000;

	//THE TOTAL PHASE FOR OUR TAPERED EVAL
	const TOTAL_PHASE: i32 = 256;
}

pub fn eval_to_bytes(eval: i32) -> [u8; 4] {
	eval.to_le_bytes()
}

pub fn bytes_to_eval(bytes: [u8; 4]) -> i32 {
	i32::from_le_bytes(bytes)
}