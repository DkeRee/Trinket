use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white_pieces = board.colors(Color::White);
	let black_pieces = board.colors(Color::Black);

	let white_only_king = (board.king(Color::White).bitboard() ^ white_pieces).is_empty();
	let black_only_king = (board.king(Color::Black).bitboard() ^ black_pieces).is_empty();

	let white_pawns = white_pieces & board.pieces(Piece::Pawn);
	let black_pawns = black_pieces & board.pieces(Piece::Pawn);

	let white_pawn_remove = if !white_pawns.is_empty() && white_pawns.len() <= 3 && (white_pawns & (Rank::Eighth.relative_to(Color::White).bitboard() | Rank::Seventh.relative_to(Color::White).bitboard())).is_empty() {
		Some(white_pieces ^ (white_pieces & board.pieces(Piece::Pawn)))
	} else {
		None
	};

	let black_pawn_remove = if !black_pawns.is_empty() && black_pawns.len() <= 3 && (black_pawns & (Rank::Eighth.relative_to(Color::Black).bitboard() | Rank::Seventh.relative_to(Color::Black).bitboard())).is_empty() {
		Some(black_pieces ^ (black_pieces & board.pieces(Piece::Pawn)))
	} else {
		None
	};

	((knight_lone_king(board, Color::White, None) || white_only_king) && (bishop_lone_king(board, Color::Black, None) || black_only_king))
	|| ((knight_lone_king(board, Color::Black, None) || black_only_king) && (bishop_lone_king(board, Color::White, None) || white_only_king))
	|| ((knight_lone_king(board, Color::White, None) || white_only_king) && (knight_lone_king(board, Color::Black, None) || black_only_king))
	|| ((bishop_pair_lone_king(board, Color::White) || white_only_king) && bishop_lone_king(board, Color::Black, black_pawn_remove))
	|| ((bishop_pair_lone_king(board, Color::Black) || black_only_king) && bishop_lone_king(board, Color::White, white_pawn_remove))
	|| (minor_piece_king(board, Color::White) && knight_lone_king(board, Color::Black, black_pawn_remove))
	|| (minor_piece_king(board, Color::Black) && knight_lone_king(board, Color::White, white_pawn_remove))
	|| (minor_piece_king(board, Color::White) && bishop_lone_king(board, Color::Black, black_pawn_remove))
	|| (minor_piece_king(board, Color::Black) && bishop_lone_king(board, Color::White, white_pawn_remove))
}

fn knight_lone_king(board: &Board, color: Color, custom_pieces: Option<BitBoard>) -> bool {
	let mut my_pieces = board.colors(color);

	if !custom_pieces.is_none() {
		my_pieces = custom_pieces.unwrap();
	}

	let me_two_or_one_knights = (my_pieces & board.pieces(Piece::Knight)).len() == 2 || (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_knights = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Knight))).is_empty();

	me_two_or_one_knights && me_only_knights
}

fn bishop_lone_king(board: &Board, color: Color, custom_pieces: Option<BitBoard>) -> bool {
	let mut my_pieces = board.colors(color);

	if !custom_pieces.is_none() {
		my_pieces = custom_pieces.unwrap();
	}

	let me_only_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	me_only_bishop && me_only_have_bishops
}

fn bishop_pair_lone_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_only_have_2_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 2;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	me_only_have_2_bishop && me_only_have_bishops
}

fn minor_piece_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);
	let opponent_pieces = board.colors(!color);

	let me_only_one_knight = (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_one_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;	
    let me_only_minor = ((my_pieces & (board.pieces(Piece::Bishop) | board.pieces(Piece::Knight))) ^ (board.king(color).bitboard() ^ my_pieces)).is_empty();

	me_only_one_knight && me_only_one_bishop && me_only_minor
}