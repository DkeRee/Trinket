use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white_only_king = (board.king(Color::White).bitboard() ^ board.colors(Color::White)).is_empty();
	let black_only_king = (board.king(Color::Black).bitboard() ^ board.colors(Color::Black)).is_empty();

	((knight_lone_king(board, Color::White) || white_only_king) && (bishop_lone_king(board, Color::Black) || black_only_king))
	|| ((knight_lone_king(board, Color::Black) || black_only_king) && (bishop_lone_king(board, Color::White) || white_only_king))
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