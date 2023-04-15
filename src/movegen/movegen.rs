use cozy_chess::*;
use crate::movegen::movesorter::*;

#[derive(Clone, Debug)]
pub struct SortedMove {
	pub mv: Move,
	pub importance: i32,
	pub movetype: MoveType,
	pub is_killer: bool,
	pub is_countermove: bool,
	pub history: i32
}

impl SortedMove {
	pub fn new(mv: Move, importance: i32, movetype: MoveType) -> SortedMove {
		SortedMove {
			mv: mv,
			importance: importance,
			movetype: movetype,
			is_killer: false,
			is_countermove: false,
			history: 0
		}
	}

	pub fn insert_killer(&mut self, move_sorter: &mut MoveSorter, ply: i32, board: &Board) {
		if self.movetype == MoveType::Quiet {
			move_sorter.add_killer(self.mv, ply, board);
		}
	}

	pub fn insert_history(&mut self, move_sorter: &mut MoveSorter, depth: i32) {
		if self.movetype == MoveType::Quiet {
			move_sorter.add_history(self.mv, depth);
		}
	}

	pub fn insert_countermove(&mut self, move_sorter: &mut MoveSorter,  last_move: Option<Move>) {
		if self.movetype == MoveType::Quiet && !last_move.is_none() {
			move_sorter.add_countermove(self.mv, last_move.unwrap());
		}
	}

	pub fn decay_history(&mut self, move_sorter: &mut MoveSorter, depth: i32) {
		if self.movetype == MoveType::Quiet {
			move_sorter.decay_history(self.mv, depth);
		}
	}
}

pub struct MoveGen {
	pub sorter: MoveSorter
}

impl MoveGen {
	pub fn new() -> MoveGen {
		MoveGen {
			sorter: MoveSorter::new()
		}
	}

	pub fn move_gen(&mut self, board: &Board, tt_move: Option<Move>, ply: i32, skip_hash: bool, last_move: Option<Move>) -> Vec<SortedMove> {
		let mut move_list: Vec<SortedMove> = Vec::with_capacity(64);
		let color = board.side_to_move();
		let their_pieces = board.colors(!color);

		//capture move
		board.generate_moves(|moves| {
			let mut capture_moves = moves;
			capture_moves.to &= their_pieces;
			for mv in capture_moves {
				if Some(mv) == tt_move && skip_hash {
					continue;
				}
				move_list.push(SortedMove::new(mv, 0, MoveType::Loud));
			}
			false
		});

		//quiet move
		board.generate_moves(|moves| {
			let mut quiet_moves = moves;
			quiet_moves.to &= !their_pieces;
			for mv in quiet_moves {
				if Some(mv) == tt_move && skip_hash {
					continue;
				}
				move_list.push(SortedMove::new(mv, 0, MoveType::Quiet));
			}
			false
		});

		self.sorter.sort(&mut move_list, tt_move, board, ply, last_move);

		move_list
	}

	pub fn qmove_gen(&mut self, board: &Board, tt_move: Option<Move>, ply: i32) -> Vec<SortedMove> {
		let mut move_list: Vec<SortedMove> = Vec::with_capacity(64);
		let color = board.side_to_move();
		let their_pieces = board.colors(!color);
		board.generate_moves(|moves| {
			let mut capture_moves = moves;
			capture_moves.to &= their_pieces;
			for mv in capture_moves {
				move_list.push(SortedMove::new(mv, 0, MoveType::Loud));
			}
			false
		});

		self.sorter.sort(&mut move_list, tt_move, board, ply, None);

		move_list
	}
}