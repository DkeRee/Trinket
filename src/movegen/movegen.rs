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
}

pub fn move_gen(board: &Board) -> Vec<SortedMove> {
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

	move_list
}

pub fn qmove_gen(board: &Board) -> Vec<SortedMove> {
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

	move_list
}