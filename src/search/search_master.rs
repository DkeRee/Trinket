use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::eval::evaluator::*;
use crate::eval::score::*;
use crate::search::tt::*;
use crate::movegen::movesorter::*;
use crate::movegen::movegen::*;
use crate::uci::castle_parse::*;

pub struct Engine {
	pub board: Board,
	pub max_depth: i32,
	pub my_past_positions: Vec<u64>,
	pub nodes: u64,
	pub searching_depth: i32,
	movegen: MoveGen,
	tt: TT
}

impl Engine {
	pub fn new() -> Engine {
		Engine {
			board: Board::default(),
			max_depth: 0,
			my_past_positions: Vec::with_capacity(64),
			searching_depth: 0,
			nodes: 0,
			movegen: MoveGen::new(),
			tt: TT::new()
		}
	}

	pub fn go(&mut self, max_depth: i32, wtime: i64, btime: i64, winc: i64, binc: i64, movestogo: i64, stop_abort: Arc<AtomicBool>) -> String {
		let now = Instant::now();

		let mut best_move = None;
		let mut time: f32;
		let mut timeinc: f32;

		self.max_depth = max_depth;

		self.nodes = 0;

		//set time
		match self.board.side_to_move() {
			Color::White => {
				time = wtime as f32;
				timeinc = winc as f32;
			},
			Color::Black => {
				time = btime as f32;
				timeinc = binc as f32;	
			}
		}

		for depth_index in 0..self.max_depth {
			let search_elapsed = now.elapsed().as_secs_f32() * 1000_f32;
			if search_elapsed < ((time + timeinc) / f32::min(40_f32, movestogo as f32)) {
				self.searching_depth = depth_index + 1;

				let board = &mut self.board.clone();

				//set up multithread for search abort
				let search_abort = Arc::new(AtomicBool::new(false));
				let counter_abort = search_abort.clone();
				thread::spawn(move || {
					thread::sleep(Duration::from_millis(time as u64 / 32));
					counter_abort.store(true, Ordering::Relaxed);
				});

				let mut past_positions = self.my_past_positions.clone();

				let result = self.search(&search_abort, &stop_abort, board, self.searching_depth, -i32::MAX, i32::MAX, &mut past_positions);

				if result != None {
					let (best_mv, eval) = result.unwrap();
					best_move = best_mv.clone();

					let elapsed = now.elapsed().as_secs_f32() * 1000_f32;

					//get nps
					let mut nps: u64;
					if elapsed == 0_f32 {
						nps = self.nodes;
					} else {
						nps = ((self.nodes as f32 * 1000_f32) / elapsed) as u64;
					}

					//get pv/
					/*
					let mut pv = String::new();
					let pv_board = &mut self.board.clone();

					for i in 0..self.pv[0].len() {
						if self.pv[0][i] != None {
							let pv_parsed = _960_to_regular_(self.pv[0][i], pv_board);

							pv += &(pv_parsed.clone() + " ");

							let mut uci_mv = String::new();
							let pv_mv = self.pv[0][i].unwrap();

							let from = pv_mv.from.to_string();
							let to = pv_mv.to.to_string();

							uci_mv += &from;
							uci_mv += &to;

							if pv_mv.promotion != None {
								uci_mv += &pv_mv.promotion.unwrap().to_string();
							}

							pv_board.play(uci_mv.parse().unwrap());
						}
					}
					*/

					let mut score_str = if eval.mate {
						let mut mate_score = if eval.score > 0 {
							(((Score::CHECKMATE_BASE - eval.score + 1) / 2) as f32).ceil()
						} else {
							((-(eval.score + Score::CHECKMATE_BASE) / 2) as f32).ceil()
						};

						format!("mate {}", mate_score)
					} else {
						format!("cp {}", eval.score)
					};

					println!("info depth {} time {} score {} nodes {} nps {} pv {}", self.searching_depth, elapsed as u64, score_str, self.nodes, nps, self.get_pv(board, self.searching_depth));
				} else {
					break;
				}
			} else {
				break;
			}
		}

		_960_to_regular_(best_move, &self.board)
	}

	//fish PV from TT
	fn get_pv(&self, board: &mut Board, depth: i32) -> String {
		if depth == 0 {
			return String::new();
		}

		//probe TT
		let table_find = self.tt.find(board.hash(), self.searching_depth - depth);
		if board.hash() == table_find.position {
			let mut pv = String::new();

			if board.is_legal(table_find.best_move.unwrap()) {
				board.play_unchecked(table_find.best_move.unwrap());
				pv = format!("{} {}", table_find.best_move.unwrap(), self.get_pv(board, depth - 1));
			}

			return pv;
		}

		String::new()
	}

	fn is_repetition(&self, board: &Board, past_positions: &mut Vec<u64>) -> bool {
		if past_positions.len() > 0 {
			for i in 0..past_positions.len() - 1 {
				if past_positions[i] == board.hash() {
					return true;
				}
			}
		}
		return false;
	}

	pub fn search(&mut self, abort: &AtomicBool, stop_abort: &AtomicBool, board: &Board, mut depth: i32, mut alpha: i32, beta: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && (abort.load(Ordering::Relaxed) || stop_abort.load(Ordering::Relaxed)) {
			return None;
		}

		let in_check = !board.checkers().is_empty();

		//search a little deeper if we are in check!
		if in_check {
			// https://www.chessprogramming.org/Check_Extensions
			depth += 1;
		}

		let ply = self.searching_depth - depth;

		self.nodes += 1;

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		if depth <= 0 {
			return self.qsearch(&abort, &stop_abort, board, alpha, beta, self.searching_depth, past_positions); //proceed with qSearch to avoid horizon effect
		}

		//check for three move repetition
		if self.is_repetition(board, past_positions) && ply > 0 {
			return Some((None, Eval::new(Score::DRAW, false)));
		}

		let mut legal_moves: Vec<SortedMove>;

		//look up tt
		let table_find = self.tt.find(board.hash(), ply);
		if board.hash() == table_find.position {
			//if sufficient depth
			if table_find.depth >= depth {
				//check if position from TT is a mate
				let mut is_checkmate = if table_find.eval < -Score::CHECKMATE_DEFINITE || table_find.eval > Score::CHECKMATE_DEFINITE {
					true
				} else {
					false
				};

				match table_find.node_kind {
					NodeKind::Exact => {
						return Some((table_find.best_move, Eval::new(table_find.eval, is_checkmate)));
					},
					NodeKind::UpperBound => {
						if table_find.eval <= alpha {
							return Some((table_find.best_move, Eval::new(table_find.eval, is_checkmate)));
						}	
					},
					NodeKind::LowerBound => {
						if table_find.eval >= beta {
							return Some((table_find.best_move, Eval::new(table_find.eval, is_checkmate)));
						}
					},
					NodeKind::Null => {}
				}
			}
			legal_moves = self.movegen.move_gen(board, table_find.best_move, ply);
		} else {
			legal_moves = self.movegen.move_gen(board, None, ply);
		}
		
		//static eval for tuning methods
		let static_eval = evaluate(board);

		//Reverse Futility Pruning
		/*
		// if depth isn't too deep
		// if NOT in check
		// THEN prune
		*/

		if depth <= Self::MAX_DEPTH_RFP && !in_check {
			if static_eval - (Self::MULTIPLIER_RFP * depth) >= beta {
				return Some((None, Eval::new(static_eval, false)));
			}
		}

		//Null Move Pruning
		/*
		// if NOT root node
		// if NOT in check
		// if board has non pawn material
		// if board can produce a beta cutoff
		// THEN prune
		*/

		let our_pieces = board.colors(board.side_to_move());
		let sliding_pieces = board.pieces(Piece::Rook) | board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
		if ply > 0 && !in_check && !(our_pieces & sliding_pieces).is_empty() && static_eval >= beta {
			let r = if depth > 6 {
				3
			} else {
				2
			};

			let nulled_board = board.clone().null_move().unwrap();
			let (_, mut null_score) = self.search(&abort, &stop_abort, &nulled_board, depth - r - 1, -beta, -beta + 1, past_positions)?; //perform a ZW search

			null_score.score *= -1;
		
			if null_score.score >= beta {
				return Some((None, Eval::new(beta, false))); //return the lower bound produced by the fail high for this node since doing nothing in this position is insanely good
			}
		}

		let mut moves_searched = 0;
		let mut best_move = None;
		let mut eval = Eval::new(i32::MIN, false);
		for mut sm in legal_moves {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			past_positions.push(board_cache.hash());

			let mut value: Eval;

			if moves_searched == 0 {
				let (_, mut child_eval) = self.search(&abort, &stop_abort, &board_cache, depth - 1, -beta, -alpha, past_positions)?;
				child_eval.score *= -1;

				value = child_eval;
			} else {
				//LMR can be applied
				//IF depth is above sufficient depth
				//IF the first X searched are searched
				//IF this move is QUIET
				if depth >= Self::LMR_DEPTH_LIMIT && moves_searched >= Self::LMR_FULL_SEARCHED_MOVE_LIMIT && sm.movetype == MoveType::Quiet {
					let (_, mut child_eval) = self.search(&abort, &stop_abort, &board_cache, depth - 2, -alpha - 1, -alpha, past_positions)?;
					child_eval.score *= -1;		

					value = child_eval;	
				} else {
					//hack to make sure it searches at full depth in the next step
					value = Eval::new(alpha + 1, false);
				}

				//if a value ever surprises us in the future with a score that ACTUALLY changes the lowerbound...we have to search at full depth, for this move may possibly be good
				if value.score > alpha {
					let (_, mut child_eval) = self.search(&abort, &stop_abort, &board_cache, depth - 1, -beta, -alpha, past_positions)?;
					child_eval.score *= -1;		

					value = child_eval;	
				}
			}

			past_positions.pop();

			if value.score > eval.score {
				eval = value;
				best_move = Some(mv);
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::LowerBound);
						sm.insert_killer(&mut self.movegen.sorter, ply, board);
						sm.insert_history(&mut self.movegen.sorter, depth);
						break;
					} else {
						self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::Exact);
					}
				} else {
					self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::UpperBound);
				}
			}

			moves_searched += 1;
		}

		return Some((best_move, eval));
	}

	fn qsearch(&mut self, abort: &AtomicBool, stop_abort: &AtomicBool, board: &Board, mut alpha: i32, beta: i32, mut ply: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && (abort.load(Ordering::Relaxed) || stop_abort.load(Ordering::Relaxed)) {
			return None;
		}

		self.nodes += 1;

		ply += 1;

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		let stand_pat = Eval::new(evaluate(board), false);

		//beta cutoff
		if stand_pat.score >= beta {
			return Some((None, Eval::new(beta, false)));
		}

		if alpha < stand_pat.score {
			alpha = stand_pat.score;
		}

		let move_list = self.movegen.qmove_gen(board, ply);

		//no more loud moves to be checked anymore, it can be returned safely
		if move_list.len() == 0 {
			return Some((None, stand_pat));
		}

		let mut best_move = None;
		let mut eval = stand_pat;

		for sm in move_list {

			//prune losing captures found through SEE swap algorithm
			if sm.importance < 0 {
				break;
			}

			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			past_positions.push(board_cache.hash());

			let (_, mut child_eval) = self.qsearch(&abort, &stop_abort, &board_cache, -beta, -alpha, ply, past_positions)?;

			past_positions.pop();

			child_eval.score *= -1;

			if child_eval.score > eval.score {
				eval = child_eval;
				best_move = Some(mv);
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						return Some((None, Eval::new(beta, false)));
					}
				}
			}
		}

		return Some((best_move, eval));
	}
}

impl Engine {
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 100;
	const LMR_DEPTH_LIMIT: i32 = 3;
	const LMR_FULL_SEARCHED_MOVE_LIMIT: i32 = 4;
}