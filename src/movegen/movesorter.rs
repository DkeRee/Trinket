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
	see: See
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
					mv_info.importance = Self::HASHMOVE_SCORE;
					continue;
				}
			}

			mv_info.importance = match mv_info.mv.promotion {
				Some(Piece::Queen) => Self::QUEEN_PROMOS,
				Some(Piece::Rook) => Self::UNDERPROMOS,
				Some(Piece::Bishop) => Self::UNDERPROMOS,
				Some(Piece::Knight) => Self::UNDERPROMOS,
				None => 0,
				_ => unreachable!()
			};

			if mv_info.mv.promotion.is_some() {
				continue;
			}

			if mv_info.movetype == MoveType::Loud {
				let capture_score = self.see.see(board, mv_info.mv);

				if capture_score >= 0 {
					mv_info.importance = Self::WINNING_CAPTURES + capture_score;
				} else {
					mv_info.importance = Self::LOSING_CAPTURES + capture_score;
				}
				continue;
			}

			if mv_info.movetype == MoveType::Quiet {
				let history = self.get_history(mv_info.mv);

				if self.is_killer(mv_info.mv, board, ply) {
					mv_info.importance = Self::NOTABLE_QUIETS + history;
					mv_info.is_killer = true;
					continue;
				}

				if self.is_countermove(mv_info.mv, last_move) {
					mv_info.importance = Self::NOTABLE_QUIETS + history;
					mv_info.is_countermove = true;
					continue;
				}

				mv_info.importance = Self::QUIETS + history;
				mv_info.history = history;
				continue;
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
}

impl MoveSorter {
	//ORDER: Hash, Queen, Winning Captures, Notable Quiets, Quiets, Losing Captures, Underpromo

	const HASHMOVE_SCORE: i32 = 50000;
	const QUEEN_PROMOS: i32 = 40000;
	const WINNING_CAPTURES: i32 = 30000;
	const NOTABLE_QUIETS: i32 = 20000;
	const QUIETS: i32 = 5000;
	const LOSING_CAPTURES: i32 = -10000;
	const UNDERPROMOS: i32 = -20000;

/*
	const WINNING_CAPTURE: i32 = 10000;
    const KILLER_MOVE_SCORE: i32 = 2000;
	const COUNTERMOVE_SCORE: i32 = 1000;
   	const KNIGHT_PROMO: i32 = -5000;
	const BISHOP_PROMO: i32 = -6000;
	const ROOK_PROMO: i32 = -7000;
	const HISTORY_MOVE_OFFSET: i32 = -10000;
	const LOSING_CAPTURE: i32 = -30000;
*/

	const HISTORY_MAX: i32 = 2000;
}