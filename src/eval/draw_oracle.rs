use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	(knight_lone_king(board, Color::White) && bishop_lone_king(board, Color::Black))
	|| (knight_lone_king(board, Color::Black) && bishop_lone_king(board, Color::White))
}

fn knight_lone_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_only_king = (board.king(color).bitboard() ^ my_pieces).is_empty();
	let me_two_or_one_knights = (my_pieces & board.pieces(Piece::Knight)).len() == 2 || (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_knights = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Knight))).is_empty();

	(me_two_or_one_knights && me_only_knights) || me_only_king
}

fn bishop_lone_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_only_king = (board.king(color).bitboard() ^ my_pieces).is_empty();
	let me_only_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	(me_only_bishop && me_only_have_bishops) || me_only_king
}