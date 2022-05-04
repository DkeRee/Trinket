use cozy_chess::*;

pub fn _regular_to_960_(mv_string: String, board: &Board) -> String {
	let mv = &*mv_string;
										
	let from = mv.chars().nth(0).unwrap().to_string() + &mv.chars().nth(1).unwrap().to_string();
	let to = mv.chars().nth(2).unwrap().to_string() + &mv.chars().nth(3).unwrap().to_string();

	let square: Square = from.parse().unwrap();

	if from == "e1" && (to == "c1" || to == "g1") && board.piece_on(square).unwrap() == Piece::King {
		if to == "c1" {
			return String::from("e1a1");
		} else {
			return String::from("e1h1");
		}
	} else if from == "e8" && (to == "c8" || to == "g8") && board.piece_on(square).unwrap() == Piece::King {
		if to == "c8" {
			return String::from("e8a8");
		} else {
			return String::from("e8h8");
		}
	} else {
		return mv_string;
	}
}

pub fn _960_to_regular_(mv: Option<Move>, board: &Board) -> String {
	let mv_parsed = mv.unwrap();

	let from = mv_parsed.from.to_string();
	let to = mv_parsed.to.to_string();

	let square: Square = from.parse().unwrap();

	if from == "e1" && (to == "a1" || to == "h1") && board.piece_on(square).unwrap() == Piece::King {
		if to == "a1" {
			return String::from("e1c1");
		} else {
			return String::from("e1g1");
		}
	} else if from == "e8" && (to == "a8" || to == "h8") && board.piece_on(square).unwrap() == Piece::King {
		if to == "a8" {
			return String::from("e8c8");
		} else {
			return String::from("e8g8");
		}
	} else {
		let mut uci_mv = String::new();

		uci_mv += &from;
		uci_mv += &to;

		if mv_parsed.promotion != None {
			uci_mv += &mv_parsed.promotion.unwrap().to_string();
		}

		return uci_mv;
	}
}