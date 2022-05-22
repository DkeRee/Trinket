#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

use cozy_chess::*;
use crate::eval::eval_info::*;

struct Evaluator<'a> {
	board: &'a Board,
	color: Color
}

impl Evaluator<'_> {
	fn new(board: &Board, color: Color) -> Evaluator {
		Evaluator {
			board: board,
			color: color
		}
	}

	//evaluates piece weights + PST with tapered eval
	fn eval(&self) -> i32 {
		let phase = self.calculate_phase();
		let mut sum = 0;

		for &piece in &Piece::ALL {
			let pieces = self.board.colors(self.color) & self.board.pieces(piece);

			for square in pieces {
				let square_idx = self.square_index(square);

				match piece {
					Piece::Pawn => {
						sum += PAWN.eval(phase);
						sum += P[square_idx].eval(phase);
					},
					Piece::Knight => {
						sum += KNIGHT.eval(phase);
						sum += N[square_idx].eval(phase);
					},
					Piece::Bishop => {
						sum += BISHOP.eval(phase);
						sum += B[square_idx].eval(phase);
					},
					Piece::Rook => {
						sum += ROOK.eval(phase);
						sum += R[square_idx].eval(phase);
					},
					Piece::Queen => {
						sum += QUEEN.eval(phase);
						sum += Q[square_idx].eval(phase);
					},
					Piece::King => {
						sum += K[square_idx].eval(phase);
					}
				}
			}
		}

		sum
	}

	fn calculate_phase(&self) -> i32 {
		let mut phase = Self::TOTAL_PIECE_PHASE;

		let pawns = self.board.pieces(Piece::Pawn);
		let knights = self.board.pieces(Piece::Knight);
		let bishops = self.board.pieces(Piece::Bishop);
		let rooks = self.board.pieces(Piece::Rook);
		let queens = self.board.pieces(Piece::Queen);

		phase -= self.get_piece_amount(pawns) * Self::PAWN_PHASE;
		phase -= self.get_piece_amount(knights) * Self::KNIGHT_PHASE;
		phase -= self.get_piece_amount(bishops) * Self::BISHOP_PHASE;
		phase -= self.get_piece_amount(rooks) * Self::ROOK_PHASE;
		phase -= self.get_piece_amount(queens) * Self::QUEEN_PHASE;

		phase = (phase * 256 + (Self::TOTAL_PIECE_PHASE / 2)) / Self::TOTAL_PIECE_PHASE;
	
		phase
	}

	fn square_index(&self, square: Square) -> usize {
		if self.color == Color::White {
			square as usize
		} else {
			//mirrors square
			square as usize ^ 0x38
		}
	}

	fn get_piece_amount(&self, piece_type: BitBoard) -> i32 {
		let mut piece_amount = 0;
		for _piece in piece_type {
			piece_amount += 1;
		}
		piece_amount
	}
}

impl Evaluator<'_> {
	const PAWN_PHASE: i32 = 0;
	const KNIGHT_PHASE: i32 = 1;
	const BISHOP_PHASE: i32 = 1;
	const ROOK_PHASE: i32 = 2;
	const QUEEN_PHASE: i32 = 4;
	const TOTAL_PIECE_PHASE: i32 = 24;
}

pub fn evaluate(board: &Board) -> i32 {
	let mut eval = 0;

	let white_eval = Evaluator::new(board, Color::White);
	let black_eval = Evaluator::new(board, Color::Black);

	eval += white_eval.eval();
	eval -= black_eval.eval();

	if board.side_to_move() == Color::White {
		eval
	} else {
		-eval
	}
}