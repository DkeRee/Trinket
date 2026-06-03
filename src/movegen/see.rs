use cozy_chess::*;

/*
Special thanks to Malarksist and Pali from Openbench!
https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
*/


#[derive(Clone)]
pub struct See {
	gains: [i32; 32]
}

impl See {
	pub fn new() -> See {
		See {
			gains: [0_i32; 32]
		}
	}

	pub fn see(&mut self, board: &Board, mv: Move) -> i32 {
		self.gains = [0_i32; 32];

		let mv_piece = board.piece_on(mv.from).unwrap();

		// cozy_chess stores EP as a File, not a Square.
		// EP capture: pawn moves to the EP file on the correct destination rank.
		let ep_file = board.en_passant();
		let is_ep = mv_piece == Piece::Pawn
			&& ep_file == Some(mv.to.file())
			&& mv.to.rank() == Rank::Sixth.relative_to(board.side_to_move());

		// FIX #1: Handle en passant — mv.to is empty but a pawn is captured
		self.gains[0] = if is_ep {
			self.piece_pts(Piece::Pawn)
		} else if let Some(piece) = board.piece_on(mv.to) {
			self.piece_pts(piece)
		} else {
			0
		};

		let mut color = !board.side_to_move();

		// FIX #2: Also remove the EP-captured pawn from blockers, not just mv.from
		let mut blockers = board.occupied() & !mv.from.bitboard();
		if is_ep {
			// The captured pawn sits one rank behind mv.to relative to the moving side
			let ep_pawn_sq = match board.side_to_move() {
				Color::White => Square::new(mv.to.file(), Rank::Fifth),
				Color::Black => Square::new(mv.to.file(), Rank::Fourth),
			};
			blockers &= !ep_pawn_sq.bitboard();
		}

		let mut last_piece_pts = self.piece_pts(mv_piece);
		let mut max_depth = 0;

		'outer: for i in 1..32 {
			self.gains[i] = last_piece_pts - self.gains[i - 1];

			let defenders = board.colors(color) & blockers;

			for &piece in &Piece::ALL {
				last_piece_pts = self.piece_pts(piece);

				let attackers = match piece {
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

				if attackers != BitBoard::EMPTY {
					let attacker = attackers.next_square().unwrap();
					blockers &= !attacker.bitboard();
					color = !color;
					max_depth = i;
					continue 'outer;
				}
			}
			// No attacker found — exchange ends here
			break;
		}

		// FIX #3: Rollback uses inclusive range (..=max_depth) so gains[max_depth] is included
		for depth in (1..=max_depth).rev() {
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