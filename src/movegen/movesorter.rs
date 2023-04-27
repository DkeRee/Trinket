use cozy_chess::*;
use crate::movegen::movegen::*;
use crate::movegen::see::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MoveType {
	Loud,
	Quiet
}

pub struct MoveSorter {
	killer_table: [[[Option<Move>; 2]; 100]; 2],
	history_table: [[i32; 64]; 64],
	countermove_table: [[Option<Move>; 64]; 64],
	pub see: See
}

impl MoveSorter {
	pub fn new () -> MoveSorter {
		MoveSorter {
			killer_table: [[[None; 2]; 100]; 2],
			history_table: [[0; 64]; 64],
			countermove_table: [[None; 64]; 64],
			see: See::new()
		}
	}

	pub fn sort(&mut self, move_list: &mut Vec<SortedMove>, tt_move: Option<Move>, board: &Board, ply: i32, last_move: Option<Move>) {
		for i in 0..move_list.len() {
			let mv_info = &mut move_list[i];

			if tt_move != None {
				if Some(mv_info.mv) == tt_move {
					mv_info.importance += Self::HASHMOVE_SCORE;
				}
			}

			if mv_info.movetype == MoveType::Quiet {
				if self.is_castling(mv_info.mv, board) {
					mv_info.importance += Self::CASTLING_SCORE;
				}

				if self.is_killer(mv_info.mv, board, ply) {
					mv_info.importance += Self::KILLER_MOVE_SCORE;
					mv_info.is_killer = true;
				}

				if self.is_countermove(mv_info.mv, last_move) {
					mv_info.importance += Self::COUNTERMOVE_SCORE;
					mv_info.is_countermove = true;
				}

				let history = self.get_history(mv_info.mv);
				mv_info.importance += Self::HISTORY_MOVE_OFFSET + history;
				mv_info.history = history;
			}

			if mv_info.movetype == MoveType::Loud {
				let capture_score = self.see.see(board, mv_info.mv);

				if capture_score >= 0 {
					mv_info.importance += capture_score + Self::WINNING_CAPTURE;
				} else {
					mv_info.importance += capture_score + Self::LOSING_CAPTURE;
				}
			}

			mv_info.importance += match mv_info.mv.promotion {
				Some(Piece::Queen) => Self::QUEEN_PROMO,
				Some(Piece::Rook) => Self::ROOK_PROMO,
				Some(Piece::Bishop) => Self::BISHOP_PROMO,
				Some(Piece::Knight) => Self::KNIGHT_PROMO,
				None => 0,
				_ => unreachable!()
			}
		}

		move_list.sort_by(|x, z| z.importance.cmp(&x.importance));
	}

	pub fn add_killer(&mut self, mv: Move, ply: i32, board: &Board) {
		if ply < 100 && !self.is_killer(mv, board, ply) {
			let color = board.side_to_move();
			let ply_slot = &mut self.killer_table[color as usize][ply as usize];

			ply_slot.rotate_right(1);
			ply_slot[0] = Some(mv);
		}
	}

	pub fn add_history(&mut self, mv: Move, depth: i32) {
		let history = self.history_table[mv.from as usize][mv.to as usize];
		let change = depth * depth;

		if !change.checked_mul(history).is_none() {
			self.history_table[mv.from as usize][mv.to as usize] += change - change * history / Self::HISTORY_MAX; //add quiet score into history table based on from and to squares
		}
	}

	pub fn add_countermove(&mut self, mv: Move, last_move: Move) {
		self.countermove_table[last_move.from as usize][last_move.to as usize] = Some(mv);
	}

	pub fn decay_history(&mut self, mv: Move, depth: i32) {
		let history = self.history_table[mv.from as usize][mv.to as usize];
		let change = depth * depth;

		if !change.checked_mul(history).is_none() {
			self.history_table[mv.from as usize][mv.to as usize] -= change + change * history / Self::HISTORY_MAX; //decay quiet score into history table based on from and to squares
		}
	}

	fn is_killer(&self, mv: Move, board: &Board, ply: i32) -> bool {
		if ply < 100 {
			let color = board.side_to_move();
			let ply_slot = self.killer_table[color as usize][ply as usize];

			for i in 0..ply_slot.len() {
				if !ply_slot[i].is_none() {
					if ply_slot[i].unwrap() == mv {
						return true;
					}
				}
			}
		}

		return false;
	}

	fn is_countermove(&self, mv: Move, last_move: Option<Move>) -> bool {
		if last_move.is_none() {
			return false;
		}

		return self.countermove_table[last_move.unwrap().from as usize][last_move.unwrap().to as usize] == Some(mv);
	}

	fn get_history(&self, mv: Move) -> i32 {
		return self.history_table[mv.from as usize][mv.to as usize];
	}

	fn is_castling(&self, mv: Move, board: &Board) -> bool {
		if mv.from == Square::E1 && (mv.to == Square::C1 || mv.to == Square::G1) && board.piece_on(mv.from).unwrap() == Piece::King {
			return true;
		} else if mv.from == Square::E8 && (mv.to == Square::C8 || mv.to == Square::G8) && board.piece_on(mv.from).unwrap() == Piece::King {
			return true;
		} else {
			return false;
		}
	}
}

impl MoveSorter {
	const HASHMOVE_SCORE: i32 = 25000;
	const WINNING_CAPTURE: i32 = 10000;
	const QUEEN_PROMO: i32 = 8000;
    const KILLER_MOVE_SCORE: i32 = 2000;
	const COUNTERMOVE_SCORE: i32 = 1000;
	const CASTLING_SCORE: i32 = 1000;
   	const KNIGHT_PROMO: i32 = -5000;
	const BISHOP_PROMO: i32 = -6000;
	const ROOK_PROMO: i32 = -7000;
	const HISTORY_MOVE_OFFSET: i32 = -10000;
	const LOSING_CAPTURE: i32 = -30000;

	const HISTORY_MAX: i32 = 2000;
}