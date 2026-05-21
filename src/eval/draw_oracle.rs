use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white_only_king = (board.king(Color::White).bitboard() ^ board.colors(Color::White)).is_empty();
	let black_only_king = (board.king(Color::Black).bitboard() ^ board.colors(Color::Black)).is_empty();

	((knight_lone_king(board, Color::White) || white_only_king) && (bishop_lone_king(board, Color::Black) || black_only_king))
	|| ((knight_lone_king(board, Color::Black) || black_only_king) && (bishop_lone_king(board, Color::White) || white_only_king))
	|| ((knight_lone_king(board, Color::White) || white_only_king) && (knight_lone_king(board, Color::Black) || black_only_king))
	|| bishops_same_color_only(board)
}

fn knight_lone_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_two_or_one_knights = (my_pieces & board.pieces(Piece::Knight)).len() == 2 || (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_knights = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Knight))).is_empty();

	me_two_or_one_knights && me_only_knights
}

fn bishop_lone_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_only_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	me_only_bishop && me_only_have_bishops
}

fn bishops_same_color_only(board: &Board) -> bool {
	let white_pieces = board.colors(Color::White);
	let black_pieces = board.colors(Color::Black);

	let white_only_bishops =
		((board.king(Color::White).bitboard() ^ white_pieces)
			^ (white_pieces & board.pieces(Piece::Bishop))).is_empty();

	let black_only_bishops =
		((board.king(Color::Black).bitboard() ^ black_pieces)
			^ (black_pieces & board.pieces(Piece::Bishop))).is_empty();

	if !white_only_bishops || !black_only_bishops {
		return false;
	}

	let bishops = board.pieces(Piece::Bishop);

	let mut seen_light = false;
	let mut seen_dark = false;

	for sq in bishops {
		let idx = sq as usize;

		let is_light = ((idx / 8) + (idx % 8)) % 2 == 1;

		if is_light {
			seen_light = true;
		} else {
			seen_dark = true;
		}

		if seen_light && seen_dark {
			return false;
		}
	}

	true
}