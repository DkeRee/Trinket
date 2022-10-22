use cozy_chess::*;

/*
Special thanks to Malarksist and Pali from Openbench!
https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
*/

#[derive(Clone, Debug)]
pub struct See {
	gains: [i32; 16]
}

impl See {
	pub fn new() -> See {
		See {
			gains: [0_i32; 16]
		}
	}

	pub fn see(&mut self, board: &Board, mv: Move) -> i32 {
		self.gains = [0_i32; 16];

		let mut max_depth = 0;
		let mv_piece = board.piece_on(mv.from).unwrap();

		self.gains[0] = if let Some(piece) = board.piece_on(mv.to) {
			self.piece_pts(piece)
		} else {
			if mv_piece == Piece::King {
				return 0;
			}
			0
		};

		let mut color = !board.side_to_move();
		let mut blockers = board.occupied() & !mv.from.bitboard();
		let mut last_piece_pts = self.piece_pts(mv_piece);
	
		'outer: for i in 1..16 {
			self.gains[i] = last_piece_pts - self.gains[i - 1];

			let defenders = board.colors(color) & blockers;

			for &piece in &Piece::ALL {
				last_piece_pts = self.piece_pts(piece);

				let mut victim_square = match piece {
					Piece::Pawn => {
						cozy_chess::get_pawn_attacks(mv.to, !color)
					},
					Piece::Knight => {
						cozy_chess::get_knight_moves(mv.to)
					},
					Piece::Bishop => {
						cozy_chess::get_bishop_moves(mv.to, blockers)
					},
					Piece::Rook => {
						cozy_chess::get_rook_moves(mv.to, blockers)
					},
					Piece::Queen => {
						cozy_chess::get_rook_moves(mv.to, blockers)
							| cozy_chess::get_bishop_moves(mv.to, blockers)
					},
					Piece::King => {
						cozy_chess::get_king_moves(mv.to)
					}
				} & board.pieces(piece) & defenders;

				if victim_square != BitBoard::EMPTY {
					let attacker = victim_square.next_square().unwrap();
					blockers &= !attacker.bitboard();
					color = !color;
					continue 'outer;
				}
			}
			max_depth = i;
			break;
		}
		for depth in (1..max_depth).rev() {
			self.gains[depth - 1] = -i32::max(-self.gains[depth - 1], self.gains[depth]);
		}
		self.gains[0]
	}

	fn piece_pts(&self, piece: Piece) -> i32 {
		match piece {
			Piece::Pawn => 100,
			Piece::Knight => 375,
			Piece::Bishop => 375,
			Piece::Rook => 500,
			Piece::Queen => 1025,
			Piece::King => 10000
		}
	}
}