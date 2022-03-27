use cozy_chess::*;

const MB: usize = 16;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeKind {
	Exact,
	UpperBound,
	LowerBound,
	Null
}

#[derive(Copy, Clone, Debug)]
pub struct TTSlot {
	pub best_move: Option<Move>,
	pub eval: i32,
	pub position: u64,
	pub depth: i32,
	pub node_kind: NodeKind
}

impl TTSlot {
	fn new(best_move: Option<Move>, eval: i32, position: u64, depth: i32, node_kind: NodeKind) -> TTSlot {
		TTSlot {
			best_move: best_move,
			eval: eval,
			position: position,
			depth: depth,
			node_kind: node_kind
		}
	}
}

pub struct TT {
	pub table: Vec<TTSlot>,
	length: u64
}

impl TT {
	pub fn new() -> TT {
		TT {
			table: vec![TTSlot::new(None, 0, 0, 0, NodeKind::Null); MB * 1024 * 1024 / std::mem::size_of::<TTSlot>()],
			length: (MB * 1024 * 1024 / std::mem::size_of::<TTSlot>()) as u64
		}
	}

	//adjust tricky mate scores to make valid eval
	fn add_mate_score(&self, eval: i32, searching_depth: i32, depth: i32) -> i32 {
		if eval < -30000 {
			eval + (searching_depth - depth);
		} else if eval > 30000 {
			eval - (searching_depth - depth);
		} else {
			return eval;
		}
		return eval;
	}

	fn remove_mate_score(&self, eval: i32, searching_depth: i32, depth: i32) -> i32 {
		if eval < -30000 {
			eval - (searching_depth - depth);
		} else if eval > 30000 {
			eval + (searching_depth - depth);
		} else {
			return eval;
		}
		return eval;
	}

	pub fn insert(&mut self, best_move: Option<Move>, eval: i32, position: u64, searching_depth: i32, depth: i32, node_kind: NodeKind) {
		self.table[(position % self.length) as usize] = TTSlot::new(best_move, self.remove_mate_score(eval, searching_depth, depth), position, depth, node_kind);
	}

	pub fn find(&self, position: u64, searching_depth: i32, depth: i32) -> TTSlot {
		let mut data = self.table[(position % self.length) as usize];
		data.eval = self.add_mate_score(data.eval, searching_depth, depth);
		data
	}
}