#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

use cozy_chess::*;
use crate::eval::score::*;
use crate::eval::eval_info::*;
use crate::eval::draw_oracle::*;

const PAWN_PHASE: i32 = 0;
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;
const TOTAL_PIECE_PHASE: i32 = 24;

pub fn calculate_phase(board: &Board) -> i32 {
	let mut phase = TOTAL_PIECE_PHASE;

	let pawns = board.pieces(Piece::Pawn);
	let knights = board.pieces(Piece::Knight);
	let bishops = board.pieces(Piece::Bishop);
	let rooks = board.pieces(Piece::Rook);
	let queens = board.pieces(Piece::Queen);

	phase -= pawns.len() as i32 * PAWN_PHASE;
	phase -= knights.len() as i32 * KNIGHT_PHASE;
	phase -= bishops.len() as i32 * BISHOP_PHASE;
	phase -= rooks.len() as i32 * ROOK_PHASE;
	phase -= queens.len() as i32 * QUEEN_PHASE;

	phase = (phase * 256 + (TOTAL_PIECE_PHASE / 2)) / TOTAL_PIECE_PHASE;
	
	phase
}

struct Evaluator<'a> {
	board: &'a Board,
	phase: i32,
	color: Color
}

impl Evaluator<'_> {
	fn new(board: &Board, phase: i32, color: Color) -> Evaluator {
		Evaluator {
			board: board,
			phase: phase,
			color: color
		}
	}

	//evaluates piece weights + PST with tapered eval
	fn eval(&self) -> i32 {
		let mut sum = 0;

		for &piece in &Piece::ALL {
			let pieces = self.board.colors(self.color) & self.board.pieces(piece);

			for square in pieces {
				let square_idx = self.square_index(square);

				match piece {
					Piece::Pawn => {
						sum += PAWN.eval(self.phase);
						sum += P[square_idx].eval(self.phase);
					},
					Piece::Knight => {
						sum += KNIGHT.eval(self.phase);
						sum += N[square_idx].eval(self.phase);
					},
					Piece::Bishop => {
						sum += BISHOP.eval(self.phase);
						sum += B[square_idx].eval(self.phase);
					},
					Piece::Rook => {
						sum += ROOK.eval(self.phase);
						sum += R[square_idx].eval(self.phase);
					},
					Piece::Queen => {
						sum += QUEEN.eval(self.phase);
						sum += Q[square_idx].eval(self.phase);
					},
					Piece::King => {
						sum += K[square_idx].eval(self.phase);
					}
				}
			}
		}

		//load in extra calculations
		sum += self.connected_pawns();
		sum += self.get_mobility();
		sum += self.virtual_mobility();
		sum += self.bishop_pair();
		sum += self.passed_pawns();
		sum += self.pawn_island();
		sum += self.isolated_pawn();
		sum += self.rook_files();
		sum += self.king_on_risky_file();

		sum
	}

	fn get_mobility_weight(&self, piece: Piece) -> &[Score] {
		match piece {
			Piece::Pawn => &PAWN_MOBILITY,
			Piece::Knight => &KNIGHT_MOBILITY,
			Piece::Bishop => &BISHOP_MOBILITY,
			Piece::Rook => &ROOK_MOBILITY,
			Piece::Queen => &QUEEN_MOBILITY,
			Piece::King => &KING_MOBILITY
		}
	}

	fn king_on_risky_file(&self) -> i32 {
		let mut penalty = 0;

		let pawns = self.board.pieces(Piece::Pawn);
		let our_pawns = self.board.colors(self.color) & pawns;
		let our_king_file = self.board.king(self.color).file();

		if (pawns & our_king_file.bitboard()).is_empty() {
			penalty += KING_ON_OPEN_FILE.eval(self.phase);
		} else if (our_pawns & our_king_file.bitboard()).is_empty() {
			penalty += KING_ON_SEMI_OPEN_FILE.eval(self.phase);
		}

		penalty
	}

	fn connected_pawns(&self) -> i32 {
		let mut bonus = 0;
		let our_pawns = self.board.colors(self.color) & self.board.pieces(Piece::Pawn);

		for pawn in our_pawns {
			for supporting_location in get_pawn_attacks(pawn, !self.color) {
				if !(supporting_location.bitboard() & our_pawns).is_empty() {
					//we have one of our pawns on this square, supporting the checking pawn
					bonus += CONNECTED_PASSED_PAWN.eval(self.phase);
				}
			}
		}

		bonus
	}

	fn virtual_mobility(&self) -> i32 {
		let occupied = self.board.occupied();
		let my_king = self.board.king(self.color);

		let virtual_queen_moves = (get_bishop_moves(my_king, occupied) | get_rook_moves(my_king, occupied)) & !self.board.colors(self.color);
		let mobility = virtual_queen_moves.len();

		VIRTUAL_MOBILITY[mobility as usize].eval(self.phase)
	}

	fn get_mobility(&self) -> i32 {
		let mut score = 0;
		let our_pieces = self.board.colors(self.color);
		let occupied = self.board.occupied();

		for &piece in &Piece::ALL {
			let our_this_piece = our_pieces & self.board.pieces(piece);
			let mobility_weight = self.get_mobility_weight(piece);

			//Sum up number of moves that our pieces have that can have, including loud moves.
			for square in our_this_piece {
				let mut feasible_moves = BitBoard::EMPTY;

				match piece {
					Piece::Pawn => {
						feasible_moves |= get_pawn_quiets(square, self.color, occupied) | (get_pawn_attacks(square, self.color) & !our_pieces);
					},
					Piece::Knight => {
						feasible_moves |= get_knight_moves(square) & !our_pieces;
					},
					Piece::Bishop => {
						feasible_moves |= get_bishop_moves(square, BitBoard::EMPTY) & !our_pieces;
					},
					Piece::Rook => {
						feasible_moves |= get_rook_moves(square, BitBoard::EMPTY) & !our_pieces;
					},
					Piece::Queen => {
						feasible_moves |= (get_bishop_moves(square, BitBoard::EMPTY) | get_rook_moves(square, BitBoard::EMPTY)) & !our_pieces;
					},
					Piece::King => {
						feasible_moves |= get_king_moves(square) & !our_pieces;
					}
				}

				score += mobility_weight[feasible_moves.len() as usize].eval(self.phase);
			}
		}

		score
	}

	fn rook_files(&self) -> i32 {
		let mut score = 0;
		let our_pieces = self.board.colors(self.color);
		let all_pawns = self.board.pieces(Piece::Pawn);
		let our_pawns = our_pieces & all_pawns;
		let our_rooks = our_pieces & self.board.pieces(Piece::Rook);

		for rook in our_rooks {
			let rook_file = rook.file().bitboard();

			//check if there are no pawns on the line our rook is at. if so, it is on open file. if there are only enemy pawns on it, it is on a semi-opem file.
			if (all_pawns & rook_file).is_empty() {
				//it is on an open file
				score += ROOK_OPEN_FILE_BONUS.eval(self.phase);
			} else if (our_pawns & rook_file).is_empty() {
				//it is on a semi open file
				score += ROOK_SEMI_FILE_BONUS.eval(self.phase);
			}
		}

		score
	}

	fn passed_pawns(&self) -> i32 {
		let mut score = 0;
		let all_pawns = self.board.pieces(Piece::Pawn);
		let our_pawns = all_pawns & self.board.colors(self.color);
		let enemy_pawns = all_pawns & self.board.colors(!self.color);
		let promo_rank = Rank::Eighth.relative_to(self.color);

		for pawn in our_pawns {
			let mut pawn_goal = Square::new(pawn.file(), promo_rank);
			let mut checking_file = get_between_rays(pawn, pawn_goal);
			let mut block_mask = checking_file;

			//use this handy dandy attack function to add files to the right and left of pawn
			for attack_location in get_pawn_attacks(pawn, self.color) {
				pawn_goal = Square::new(attack_location.file(), promo_rank);
				checking_file = get_between_rays(attack_location, pawn_goal); //check from the pawn

				//add file to the BB block mask
				block_mask |= checking_file | attack_location.bitboard();
			}

			//check to see if these three BB files contain enemy pawns in them && and if this is not a pawn island
			let passed = (enemy_pawns & block_mask).is_empty() && (our_pawns & get_between_rays(pawn, Square::new(pawn.file(), promo_rank))).is_empty();
			if passed {
				score += PASSED_PAWN_BONUS.eval(self.phase);
			}
		}

		score
	}

	fn pawn_island(&self) -> i32 {
		let mut penalty = 0;
		let all_pawns = self.board.pieces(Piece::Pawn);
		let our_pawns = all_pawns & self.board.colors(self.color);
		let promo_rank = Rank::Eighth.relative_to(self.color);

		for pawn in our_pawns {
			let pawn_goal = Square::new(pawn.file(), promo_rank);
			let block_mask = get_between_rays(pawn, pawn_goal);

			//check if there are any of our pawns ahead of us, blocking the way
			let is_island = !(our_pawns & block_mask).is_empty();
			if is_island {
				penalty += PAWN_ISLAND_PENALTY.eval(self.phase);
			}
		}

		penalty
	}

	fn isolated_pawn(&self) -> i32 {
		let mut penalty = 0;
		let all_pawns = self.board.pieces(Piece::Pawn);
		let our_pawns = all_pawns & self.board.colors(self.color);
		let beginning_rank = Rank::First.relative_to(self.color);
		let promo_rank = Rank::Eighth.relative_to(self.color);

		for pawn in our_pawns {
			let mut block_mask = BitBoard::EMPTY;

			for attack_location in get_pawn_attacks(pawn, self.color) {
				let pawn_start = Square::new(attack_location.file(), beginning_rank);
				let pawn_goal = Square::new(attack_location.file(), promo_rank);
				let checking_file = get_between_rays(pawn_start, pawn_goal); //check from the pawn's file's base

				block_mask |= checking_file;
			}

			//check to see if we have any supporting pawns on neighbouring files
			let is_isolated = (our_pawns & block_mask).is_empty();
			if is_isolated {
				penalty += PAWN_ISOLATION_PENALTY.eval(self.phase);
			}
		}

		penalty
	}

	fn bishop_pair(&self) -> i32 {
		let mut score = 0;
		if (self.board.pieces(Piece::Bishop) & self.board.colors(self.color)).len() >= 2 {
			score += BISHOP_PAIR_BONUS.eval(self.phase);
		}

		score
	}

	fn square_index(&self, square: Square) -> usize {
		if self.color == Color::White {
			square as usize
		} else {
			//mirrors square
			square as usize ^ 0x38
		}
	}
}

impl Evaluator<'_> {
	const ORACLE_SCALE: i32 = 100;
}

pub fn evaluate(board: &Board) -> i32 {
	let mut eval = 0;

	let phase = calculate_phase(board);
	let white_eval = Evaluator::new(board, phase, Color::White);
	let black_eval = Evaluator::new(board, phase, Color::Black);

	eval += white_eval.eval();
	eval -= black_eval.eval();

	//load in extra calculations
	if board.side_to_move() == Color::White {
		eval += TEMPO.eval(phase);
	} else {
		eval -= TEMPO.eval(phase);
	}

	if oracle_lookup(board) {
		//scale eval down in the case of a known draw
		eval /= Evaluator::ORACLE_SCALE;
	}

	if board.side_to_move() == Color::White {
		eval
	} else {
		-eval
	}
}