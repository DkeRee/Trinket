#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

use cozy_chess::*;
use crate::eval::eval_info::*;

pub struct Evaluator {
	pub end_game: bool
}

impl Evaluator {
	pub fn new() -> Evaluator {
		Evaluator {
			end_game: false
		}
	}

	pub fn evaluate(&self, board: &Board) -> i32 {
		//our color
		let color = board.side_to_move();
		let our_pieces = board.colors(color);
		let their_pieces = board.colors(!color);
		let mut eval = 0;
		
		let mut mysum = 0;
		let mut theirsum = 0;

		for &piece in &Piece::ALL {
		   	let my_pieces = our_pieces & board.pieces(piece);
		   	let enemy_pieces = their_pieces & board.pieces(piece);

		    for square in my_pieces {
		    	let mut piece_sum = 0;
				match piece {
					Piece::Pawn => {
						let mut weight = 0;
						if self.end_game {
							weight = pawn_endgame;
						} else {
							weight = pawn_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += p[square as usize];
						} else {
							piece_sum += pr[square as usize];
						}
					},
					Piece::Knight => {
						let mut weight = 0;
						if self.end_game {
							weight = knight_endgame;
						} else {
							weight = knight_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += n[square as usize];
						} else {
							piece_sum += nr[square as usize];
						}
					},
					Piece::Bishop => {
						let mut weight = 0;
						if self.end_game {
							weight = bishop_endgame;
						} else {
							weight = bishop_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += b[square as usize];
						} else {
							piece_sum += br[square as usize];
						}
					},
					Piece::Rook => {
						let mut weight = 0;
						if self.end_game {
							weight = rook_endgame;
						} else {
							weight = rook_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += r[square as usize];
						} else {
							piece_sum += rr[square as usize];
						}
					},
					Piece::Queen => {
						let mut weight = 0;
						if self.end_game {
							weight = queen_endgame;
						} else {
							weight = queen_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += q[square as usize];
						} else {
							piece_sum += qr[square as usize];
						}
					},
					Piece::King => {
						let weight = 0;

						piece_sum = weight;

						if self.end_game {
							if color == Color::White {
								piece_sum += k_e[square as usize];
							} else {
								piece_sum += k_er[square as usize];
							}
						} else {
							if color == Color::White {
								piece_sum += k[square as usize];
							} else {
								piece_sum += kr[square as usize];
							}
						}
					},
					_ => unreachable!()
				};
				mysum += piece_sum;
		    }

		    for square in enemy_pieces {
		    	let mut piece_sum = 0;
				match piece {
					Piece::Pawn => {
						let mut weight = 0;
						if self.end_game {
							weight = pawn_endgame;
						} else {
							weight = pawn_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += pr[square as usize];
						} else {
							piece_sum += p[square as usize];
						}
					},
					Piece::Knight => {
						let mut weight = 0;
						if self.end_game {
							weight = knight_endgame;
						} else {
							weight = knight_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += nr[square as usize];
						} else {
							piece_sum += n[square as usize];
						}
					},
					Piece::Bishop => {
						let mut weight = 0;
						if self.end_game {
							weight = bishop_endgame;
						} else {
							weight = bishop_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += br[square as usize];
						} else {
							piece_sum += b[square as usize];
						}
					},
					Piece::Rook => {
						let mut weight = 0;
						if self.end_game {
							weight = rook_endgame;
						} else {
							weight = rook_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += rr[square as usize];
						} else {
							piece_sum += r[square as usize];
						}
					},
					Piece::Queen => {
						let mut weight = 0;
						if self.end_game {
							weight = queen_endgame;
						} else {
							weight = queen_normal;
						}

						piece_sum = weight;

						if color == Color::White {
							piece_sum += qr[square as usize];
						} else {
							piece_sum += q[square as usize];
						}
					},
					Piece::King => {
						let weight = 0;

						piece_sum = weight;

						if self.end_game {
							if color == Color::White {
								piece_sum += k_er[square as usize];
							} else {
								piece_sum += k_e[square as usize];
							}
						} else {
							if color == Color::White {
								piece_sum += kr[square as usize];
							} else {
								piece_sum += k[square as usize];
							}
						}
					},
					_ => unreachable!()
				};
				theirsum += piece_sum;
	    	}
	    }
		eval = mysum - theirsum;

	    eval
	}
}