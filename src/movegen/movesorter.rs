use cozy_chess::*;
use crate::movegen::movegen::*;
use crate::movegen::see::*;
use crate::movegen::boardwrapper::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MoveType {
	Loud,
	Quiet
}

pub struct MoveSorter {
	killer_table: [[[Option<Move>; 2]; 100]; 2],
	history_table: [[i32; 64]; 64],
	countermove_table: [[Option<Move>; 64]; 64],
	pawn_corrhist: [[f32; Self::CORRHIST_SIZE]; 2],
	non_pawn_corrhist: [[f32; Self::CORRHIST_SIZE]; 2],
	material_corrhist: [[f32; Self::CORRHIST_SIZE]; 2],
	see: See
}

impl MoveSorter {
	pub fn new () -> MoveSorter {
		MoveSorter {
			killer_table: [[[None; 2]; 100]; 2],
			history_table: [[0; 64]; 64],
			countermove_table: [[None; 64]; 64],
			pawn_corrhist: [[0.0; Self::CORRHIST_SIZE]; 2],
			non_pawn_corrhist: [[0.0; Self::CORRHIST_SIZE]; 2],
			material_corrhist: [[0.0; Self::CORRHIST_SIZE]; 2],
			see: See::new()
		}
	}

	pub fn sort(&mut self, move_list: &mut Vec<SortedMove>, tt_move: Option<Move>, board: &Board, ply: i32, last_move: Option<Move>) {
		for i in 0..move_list.len() {
			let mv_info = &mut move_list[i];

			if tt_move != None {
				if Some(mv_info.mv) == tt_move {
					mv_info.importance = Self::HASHMOVE_SCORE;
				}
			}

			let is_tt = mv_info.importance == Self::HASHMOVE_SCORE;

			if !is_tt {
				let is_promo = mv_info.mv.promotion != None;
				let mut base = 0;
				let mut increment = 0;

				if mv_info.movetype == MoveType::Loud {
					let capture_score = self.see.see(board, mv_info.mv);

					base = if capture_score > 0 {
						Self::WINNING_CAPTURE
					} else if capture_score == 0{
						Self::NEUTRAL_CAPTURE
					} else {
						Self::LOSING_CAPTURE
					};

					increment = capture_score;
				}
	
				if mv_info.movetype == MoveType::Quiet {
					base = Self::QUIET_MOVE;
	
					mv_info.is_killer = self.is_killer(mv_info.mv, board, ply);
					mv_info.is_countermove = self.is_countermove(mv_info.mv, last_move);
	
					if mv_info.is_killer || mv_info.is_countermove {
						base = 0;

						base += if mv_info.is_killer {
							Self::KILLER_QUIET
						} else {
							0
						};

						base += if mv_info.is_countermove {
							Self::COUNTER_QUIET
						} else {
							0
						};
					} else {
						let history = self.get_history(mv_info.mv);
						increment = history;
						mv_info.history = history;
					}
				}
	
				if is_promo {
					base = if mv_info.mv.promotion.unwrap() == Piece::Queen { 
						Self::PROMO
					} else { 
						Self::UNDER_PROMO
					};
				}

				mv_info.importance = base + increment;
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
		let change = depth * depth + 10;

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

	pub fn add_material_corrhist(&mut self, boardwrapper: &BoardWrapper, depth: i32, best_alpha: i32, static_eval: i32) {
		let idx = (boardwrapper.material_hash % Self::CORRHIST_SIZE as u64) as usize;
		let side = boardwrapper.board.side_to_move() as usize;
	
		let entry = &mut self.material_corrhist[side][idx];
	
		let weight = f32::min(depth as f32 * depth as f32 + 2.0, 62.0) / 596.0;
		*entry = *entry * (1.0 - weight) + ((best_alpha - static_eval) as f32).clamp(-81.0, 81.0) * 280.0 * weight;
	}

	pub fn add_pawn_corrhist(&mut self, boardwrapper: &BoardWrapper, depth: i32, best_alpha: i32, static_eval: i32) {
		let idx = (boardwrapper.pawn_hash % Self::CORRHIST_SIZE as u64) as usize;
		let side = boardwrapper.board.side_to_move() as usize;
	
		let entry = &mut self.pawn_corrhist[side][idx];
	
		let weight = f32::min(depth as f32 * depth as f32 + 2.0, 62.0) / 596.0;
		*entry = *entry * (1.0 - weight) + ((best_alpha - static_eval) as f32).clamp(-81.0, 81.0) * 280.0 * weight;
	}

	pub fn add_non_pawn_corrhist(&mut self, boardwrapper: &BoardWrapper, depth: i32, best_alpha: i32, static_eval: i32) {
		let idx_white = (boardwrapper.non_pawn_hash[Color::White as usize] % Self::CORRHIST_SIZE as u64) as usize;
		let idx_black = (boardwrapper.non_pawn_hash[Color::Black as usize] % Self::CORRHIST_SIZE as u64) as usize;
		let side_to_move = boardwrapper.board.side_to_move() as usize;
	
		let weight = f32::min(depth as f32 * depth as f32 + 2.0, 62.0) / 596.0;

		let entry_white = &mut self.non_pawn_corrhist[side_to_move][idx_white];
		*entry_white = *entry_white * (1.0 - weight) + ((best_alpha - static_eval) as f32).clamp(-81.0, 81.0) * 140.0 * weight;

		let entry_black = &mut self.non_pawn_corrhist[side_to_move][idx_black];	
		*entry_black = *entry_black * (1.0 - weight) + ((best_alpha - static_eval) as f32).clamp(-81.0, 81.0) * 140.0 * weight;
	}

	pub fn read_material_corrhist(&mut self, boardwrapper: &BoardWrapper) -> f32 {
		let material_hist = self.material_corrhist[boardwrapper.board.side_to_move() as usize][(boardwrapper.material_hash % Self::CORRHIST_SIZE as u64) as usize];
		material_hist / 289.0
	}

	pub fn read_pawn_corrhist(&mut self, boardwrapper: &BoardWrapper) -> f32 {
		let pawn_hist = self.pawn_corrhist[boardwrapper.board.side_to_move() as usize][(boardwrapper.pawn_hash % Self::CORRHIST_SIZE as u64) as usize];
		pawn_hist / 198.0
	}

	pub fn read_non_pawn_corrhist(&mut self, boardwrapper: &BoardWrapper) -> f32 {
		let side_to_move = boardwrapper.board.side_to_move() as usize;
		let idx_white = (boardwrapper.non_pawn_hash[Color::White as usize] % Self::CORRHIST_SIZE as u64) as usize;
		let idx_black = (boardwrapper.non_pawn_hash[Color::Black as usize] % Self::CORRHIST_SIZE as u64) as usize;
		let non_pawn_hist_white = self.non_pawn_corrhist[side_to_move][idx_white] / 202.0;
		let non_pawn_hist_black = self.non_pawn_corrhist[side_to_move][idx_black] / 202.0;

		non_pawn_hist_white + non_pawn_hist_black
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
	const HASHMOVE_SCORE: i32 = 1000000;

	const PROMO: i32 = 50000;
	const WINNING_CAPTURE: i32 = 50000;

	const NEUTRAL_CAPTURE: i32 = 30000;

	const KILLER_QUIET: i32 = 15000;
	const COUNTER_QUIET: i32 = 10000;
	const QUIET_MOVE: i32 = 0;

	const LOSING_CAPTURE: i32 = -50000;
	const UNDER_PROMO: i32 = -50000;

	const HISTORY_MAX: i32 = 2000;
	const CORRHIST_SIZE: usize = 16384;
}
//Ranking: TT, Promo, Good Loud Moves (further specifity by SEE), Best Quiets (further specifity by history), Quiets (furhter specifity by history), Bad Loud Moves = Underpromo
//TT will have no specifity, Promos have no specifity