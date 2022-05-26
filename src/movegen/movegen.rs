use cozy_chess::*;
use crate::movegen::movesorter::*;

#[derive(Clone, Debug)]
pub struct SortedMove {
	pub mv: Move,
	pub importance: i32,
	pub movetype: MoveType
}

impl SortedMove {
	pub fn new(mv: Move, importance: i32, movetype: MoveType) -> SortedMove {
		SortedMove {
			mv: mv,
			importance: importance,
			movetype: movetype
		}
	}
}

pub struct MoveGen {
	sorter: MoveSorter
}

impl MoveGen {
	pub fn new() -> MoveGen {
		MoveGen {
			sorter: MoveSorter::new()
		}
	}

	pub fn move_gen(&mut self, board: &Board, tt_move: Option<Move>) -> Vec<SortedMove> {
		let mut move_list: Vec<SortedMove> = Vec::with_capacity(64);
		let color = board.side_to_move();
		let their_pieces = board.colors(!color);

		//capture move
		board.generate_moves(|moves| {
			let mut capture_moves = moves;
			capture_moves.to &= their_pieces;
			for mv in capture_moves {
				move_list.push(SortedMove::new(mv, 0, MoveType::Loud));
			}
			false
		});

		//quiet move
		board.generate_moves(|moves| {
			let mut quiet_moves = moves;
			quiet_moves.to &= !their_pieces;
			for mv in quiet_moves {
				move_list.push(SortedMove::new(mv, 0, MoveType::Quiet));
			}
			false
		});

		self.sorter.sort(&mut move_list, tt_move, board);

		move_list
	}

	pub fn qmove_gen(&mut self, board: &Board) -> Vec<SortedMove> {
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

		self.sorter.sort(&mut move_list, None, board);

		move_list
	}
}