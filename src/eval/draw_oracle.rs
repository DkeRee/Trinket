use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white_only_king = (board.king(Color::White).bitboard() ^ board.colors(Color::White)).is_empty();
	let black_only_king = (board.king(Color::Black).bitboard() ^ board.colors(Color::Black)).is_empty();

	((knight_lone_king(board, Color::White) || white_only_king) && (bishop_lone_king(board, Color::Black) || black_only_king))
	|| ((knight_lone_king(board, Color::Black) || black_only_king) && (bishop_lone_king(board, Color::White) || white_only_king))
	|| (knight_lone_king(board, Color::White) && knight_lone_king(board, Color::Black))
	|| (bishop_pair_lone_king(board, Color::White) && bishop_lone_king(board, Color::Black))
	|| (bishop_pair_lone_king(board, Color::Black) && bishop_lone_king(board, Color::White))
	|| (minor_piece_king(board, Color::White) && knight_lone_king(board, Color::Black))
	|| (minor_piece_king(board, Color::Black) && knight_lone_king(board, Color::White))
	|| (minor_piece_king(board, Color::White) && bishop_lone_king(board, Color::Black))
	|| (minor_piece_king(board, Color::Black) && bishop_lone_king(board, Color::White))
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