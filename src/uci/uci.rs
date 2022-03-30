use cozy_chess::*;
use crate::search::search_master::*;

//uci command parser
pub struct UCICmd {
	engine: Engine
}

impl UCICmd {
	pub fn new() -> UCICmd {
		UCICmd {
			engine: Engine::new()
		}
	}

	pub fn post(&mut self, cmd: &str) -> String {
		let result = self.post_await(cmd);
		let mut result_parsed = String::new();

		if result != None {
			result_parsed = result.unwrap();
		}
		return result_parsed;
	}

	fn post_await(&mut self, cmd: &str) -> Option<String> {
		let split = cmd.trim().split(" ");
		let cmd_vec: Vec<&str> = split.collect();
		if cmd_vec[0] == "ucinewgame" {
			self.engine = Engine::new();
		} else if cmd_vec[0] == "uci" {
			println!("id name Trinket");
			println!("id author DkeRee");
			return Some(String::from("uciok"));
		} else if cmd_vec[0] == "isready" {
			return Some(String::from("readyok"));
		} else if cmd.starts_with("go") {
			let mut depth = i32::MAX;

			if cmd_vec.len() > 1 {
				if cmd_vec[1] == "wtime" {
					//specified time
					self.engine.wtime = cmd_vec[2].parse::<u64>().unwrap();
					self.engine.btime = cmd_vec[4].parse::<u64>().unwrap();
				} else if cmd_vec[1] == "depth" {
					//specified depth without time
					depth = cmd_vec[2].parse::<i32>().unwrap();
				}

				if cmd_vec.len() > 7 && cmd_vec[7] == "depth" {
					//specified depth with time
					depth = cmd_vec[8].parse::<i32>().unwrap();
				}
			}

			self.engine.max_depth = depth;
			let best_move = self.engine.go();

			return Some(String::from("bestmove ") + &best_move);
		} else if cmd_vec[0] == "position" {
			if cmd_vec[1] == "startpos" {
				if cmd_vec.len() == 2 {
					self.engine.board = Board::default();
				} else {
					self.engine.board = Board::default();
					self.engine.my_past_positions = Vec::with_capacity(64);
					for i in 3..cmd_vec.len() {
						let mv = cmd_vec[i];
					
						let from = mv.chars().nth(0).unwrap().to_string() + &mv.chars().nth(1).unwrap().to_string();
						let to = mv.chars().nth(2).unwrap().to_string() + &mv.chars().nth(3).unwrap().to_string();

						let square: Square = from.parse().unwrap();

						if from == "e1" && (to == "c1" || to == "g1") && self.engine.board.piece_on(square).unwrap() == Piece::King {
							if to == "c1" {
								self.engine.board.play("e1a1".parse().unwrap());
							} else {
								self.engine.board.play("e1h1".parse().unwrap());
							}
						} else if from == "e8" && (to == "c8" || to == "g8") && self.engine.board.piece_on(square).unwrap() == Piece::King {
							if to == "c8" {
								self.engine.board.play("e8a8".parse().unwrap());
							} else {
								self.engine.board.play("e8h8".parse().unwrap());
							}
						} else {
							self.engine.board.play(mv.parse().unwrap());
						}

						self.engine.my_past_positions.push(self.engine.board.hash());
					}
				}
			} else if cmd_vec[1] == "fen" {
				let mut fen = String::new();
				for i in 2..cmd_vec.len() {
					let segment = String::from(cmd_vec[i]) + " ";
					fen += &*segment;
				}
				self.engine.board = Board::from_fen(&*fen, false).unwrap();
			}
		} else if cmd_vec[0] == "player" {
			//for debugging purposes/not actually part of UCI protocol
			if cmd_vec[1].len() == 4 {
				if self.engine.board.is_legal(cmd_vec[1].parse().unwrap()) {
					self.engine.board.play(cmd_vec[1].parse().unwrap());
					self.engine.my_past_positions.push(self.engine.board.hash());
					return Some(String::from("Move Made"));
				} else {
					return Some(String::from("Invalid Move"));
				}
			} else {
				return Some(String::from("Invalid Move"));
			}
		}
		return None;
	}
}