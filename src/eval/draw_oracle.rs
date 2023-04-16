use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	(knight_lone_king(board, Color::White) && bishop_lone_king(board, Color::Black))
	|| (knight_lone_king(board, Color::Black) && bishop_lone_king(board, Color::White))
	|| minor_piece_king(board, Color::White)
	|| minor_piece_king(board, Color::Black)
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

fn minor_piece_king(board: &Board, color: Color) -> bool {
	let my_pieces = board.colors(color);
	let opponent_pieces = board.colors(!color);

	let me_only_one_knight = (my_pieces & board.pieces(Piece::Knight)).len() == 1;
	let me_only_one_bishop = (my_pieces & board.pieces(Piece::Bishop)).len() == 1;	
    let me_only_minor = ((my_pieces & (board.pieces(Piece::Bishop) | board.pieces(Piece::Knight))) ^ (board.king(color).bitboard() ^ my_pieces)).is_empty();

	let opponent_num_knight = (opponent_pieces & board.pieces(Piece::Knight)).len();
	let opponent_num_bishop = (opponent_pieces & board.pieces(Piece::Bishop)).len();

	let opponent_only_knight = ((opponent_pieces & board.pieces(Piece::Knight)) ^ (board.king(!color).bitboard() ^ opponent_pieces)).is_empty();
	let opponent_only_bishop = ((opponent_pieces & board.pieces(Piece::Bishop)) ^ (board.king(!color).bitboard() ^ opponent_pieces)).is_empty();
	
	me_only_one_knight && me_only_one_bishop && me_only_minor && ((opponent_num_knight == 1 && opponent_only_knight) || (opponent_num_bishop == 1 && opponent_only_bishop))
}