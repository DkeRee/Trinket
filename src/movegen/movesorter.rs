use cozy_chess::*;
use crate::movegen::movegen::*;
use crate::movegen::see::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MoveType {
	Loud,
	Quiet
}

pub struct MoveSorter {
	see: See
}

impl MoveSorter {
	pub fn new () -> MoveSorter {
		MoveSorter {
			see: See::new()
		}
	}

	pub fn sort(&mut self, move_list: &mut Vec<SortedMove>, tt_move: Option<Move>, board: &Board) {
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
	const WINNING_CAPTURE: i32 = 10;
	const QUEEN_PROMO: i32 = 8;
	const ROOK_PROMO: i32 = 7;
	const BISHOP_PROMO: i32 = 6;
	const KNIGHT_PROMO: i32 = 5;
	const CASTLING_SCORE: i32 = 1;
	const LOSING_CAPTURE: i32 = -30000;
}