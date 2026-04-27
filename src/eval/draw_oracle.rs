use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white_only_king = (board.king(Color::White).bitboard() ^ board.colors(Color::White)).is_empty();
	let black_only_king = (board.king(Color::Black).bitboard() ^ board.colors(Color::Black)).is_empty();
	let mut draw = false;

	//one knight one bishop + one bishop one knight
	draw = one_knight_king(board, Color::White) && one_bishop_king(board, Color::Black);
	draw = one_knight_king(board, Color::Black) && one_bishop_king(board, Color::White);

	//one bishop + one bishop
	draw = one_bishop_king(board, Color::White) && one_bishop_king(board, Color::Black);

	//one knight + one knight
	draw = one_knight_king(board, Color::White) && one_knight_king(board, Color::Black);

	//two knights + lone king
	draw = two_knight_king(board, Color::White) && black_only_king;
	draw = two_knight_king(board, Color::Black) && white_only_king;

	//one rook + one rook
	draw = one_rook_king(board, Color::White) && one_rook_king(board, Color::Black);

	//one queen + one queen
	draw = one_queen_king(board, Color::White) && one_queen_king(board, Color::Black);

	draw
}

fn one_knight_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_one_knight = (my_pieces & board.pieces(Piece::Knight)).len() == 2 || (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_knights = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Knight))).is_empty();

	me_one_knight && me_only_knights
}

fn one_bishop_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_one_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	me_one_bishop && me_only_have_bishops
}

fn two_knight_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_two_knights = (my_pieces & board.pieces(Piece::Knight)).len() == 2;
	let me_only_knights = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Knight))).is_empty();

	me_two_knights && me_only_knights
}

fn two_bishop_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_two_bishops = (my_pieces & board.pieces(Piece::Bishop)).len() == 2;
	let me_only_have_bishops = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Bishop))).is_empty();

	me_two_bishops && me_only_have_bishops
}

fn one_rook_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_one_rook = (my_pieces & board.pieces(Piece::Rook)).len() == 1;
	let me_only_have_rooks = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Rook))).is_empty();

	me_one_rook && me_only_have_rooks
}

fn one_queen_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);

	let me_one_queen = (my_pieces & board.pieces(Piece::Queen)).len() == 1;
	let me_only_have_queens = ((board.king(color).bitboard() ^ my_pieces) ^ (my_pieces & board.pieces(Piece::Queen))).is_empty();

	me_one_queen && me_only_have_queens
}