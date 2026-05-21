use cozy_chess::*;

pub fn oracle_lookup(board: &Board) -> bool {
	let white = Color::White;
	let black = Color::Black;

	let white_num_queens = board.colored_pieces(white, Piece::Queen).len();
	let white_num_rooks = board.colored_pieces(white, Piece::Rook).len();
	let white_num_bishops = board.colored_pieces(white, Piece::Bishop).len();
	let white_num_knights = board.colored_pieces(white, Piece::Knight).len();
	let white_num_pawns = board.colored_pieces(white, Piece::Pawn).len();

	let black_num_queens = board.colored_pieces(black, Piece::Queen).len();
	let black_num_rooks = board.colored_pieces(black, Piece::Rook).len();
	let black_num_bishops = board.colored_pieces(black, Piece::Bishop).len();
	let black_num_knights = board.colored_pieces(black, Piece::Knight).len();
	let black_num_pawns = board.colored_pieces(black, Piece::Pawn).len();

	let total_pieces = white_num_queens
	+ white_num_rooks
	+ white_num_bishops
	+ white_num_knights
	+ white_num_pawns
	+ black_num_queens
	+ black_num_rooks
	+ black_num_knights
	+ black_num_pawns;

	let mut draw = false;

    if total_pieces == 2 {
		draw = true;
    }

    if total_pieces == 3 && (white_num_knights != 0 || white_num_bishops != 0
        || black_num_knights != 0 || black_num_bishops != 0)
    {
		draw = true;
    }

    if total_pieces == 4
        && (white_num_knights != 0 || white_num_bishops != 0)
        && (black_num_knights != 0 || black_num_bishops != 0)
    {
		draw = true;
    }

    if total_pieces == 4 && (white_num_knights == 2 || black_num_knights == 2) {
		draw = true;
    }

    if total_pieces == 5 && white_num_knights == 2 && (black_num_knights != 0 || black_num_bishops != 0) {
		draw = true;
    }

    if total_pieces == 5 && black_num_knights == 2 && (white_num_knights != 0 || white_num_bishops != 0) {
		draw = true;
    }

    if total_pieces == 5 && white_num_bishops == 2 && black_num_bishops != 0 {
		draw = true;
    }

    if total_pieces == 5 && black_num_bishops == 2 && white_num_bishops != 0 {
		draw = true;
    }

    if total_pieces == 4 && white_num_rooks != 0 && black_num_rooks != 0 {
		draw = true;
    }

    if total_pieces == 4 && white_num_queens != 0 && black_num_queens != 0 {
		draw = true;
    }

	draw
}