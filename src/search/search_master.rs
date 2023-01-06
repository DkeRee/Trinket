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
 use crate::search::lmr_table::*;

pub struct TimeControl {
	pub depth: i32,
	pub wtime: i64,
	pub btime: i64,
	pub winc: i64,
	pub binc: i64,
	pub movetime: Option<i64>,
	pub movestogo: Option<i64>,
	pub handler: Arc<AtomicBool>
}

impl TimeControl {
	pub fn new(stop_abort: Arc<AtomicBool>) -> TimeControl {
		TimeControl {
			depth: i32::MAX,
			wtime: i64::MAX,
			btime: i64::MAX,
			winc: 0,
			binc: 0,
			movetime: None,
			movestogo: None,
			handler: stop_abort
		}
	}
}

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
	pub fn new(hash: u32) -> Engine {
		Engine {
			board: Board::default(),
			max_depth: 0,
			my_past_positions: Vec::with_capacity(64),
			searching_depth: 0,
			nodes: 0,
			movegen: MoveGen::new(),
			tt: TT::new(hash)
		}
	}

	pub fn go(&mut self, time_control: TimeControl) -> String {
		let now = Instant::now();

		let mut best_move = None;

		//set time
		let movetime = time_control.movetime;
		let movestogo = time_control.movestogo;
		let mut time: u64;
		let mut timeinc: u64;

		self.max_depth = time_control.depth;

		self.nodes = 0;

		match self.board.side_to_move() {
			Color::White => {
				time = time_control.wtime as u64;
				timeinc = time_control.winc as u64;
			},
			Color::Black => {
				time = time_control.btime as u64;
				timeinc = time_control.binc as u64;	
			}
		}

		//set up multithread for search abort
		let abort = time_control.handler.clone();
		if time != u64::MAX {
			thread::spawn(move || {
				let search_time = if movetime.is_none() {
					(time + timeinc / 2) / movestogo.unwrap_or(40) as u64
				} else {
					movetime.unwrap() as u64
				};

				thread::sleep(Duration::from_millis(search_time));
				abort.store(true, Ordering::Relaxed);
			});
		}

		//ASPIRATION WINDOWS ALPHA BETA
		let mut alpha = -i32::MAX;
		let mut beta = i32::MAX;

		let mut depth_index = 0;

		while depth_index < self.max_depth {
			self.searching_depth = depth_index + 1;
			let board = &mut self.board.clone();
			let mut past_positions = self.my_past_positions.clone();

			let result = self.search(&time_control.handler, board, self.searching_depth, 0, alpha, beta, &mut past_positions);
			if result != None {
				let (best_mv, eval) = result.unwrap();

				//MANAGE ASPIRATION WINDOWS
				if eval.score >= beta {
					beta += Self::ASPIRATION_WINDOW * 4;
					continue;						
				} else if eval.score <= alpha {
					alpha -= Self::ASPIRATION_WINDOW * 4;
					continue;						
				} else {
					alpha = eval.score - Self::ASPIRATION_WINDOW;
					beta = eval.score + Self::ASPIRATION_WINDOW;
					best_move = best_mv.clone();
					depth_index += 1;
				}

				let elapsed = now.elapsed().as_secs_f32() * 1000_f32;

				//get nps
				let mut nps: u64;
				if elapsed == 0_f32 {
					nps = self.nodes;
				} else {
					nps = ((self.nodes as f32 * 1000_f32) / elapsed) as u64;
				}

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

				println!("info depth {} time {} score {} nodes {} nps {} pv {}", self.searching_depth, elapsed as u64, score_str, self.nodes, nps, self.get_pv(board, self.searching_depth, 0));
			} else {
				break;
			}
		}

		_960_to_regular_(best_move, &self.board)
	}

	//fish PV from TT
	fn get_pv(&self, board: &mut Board, depth: i32, ply: i32) -> String {
		if depth == 0 || ply > 50 {
			return String::new();
		}

		//probe TT
		match self.tt.find(board, ply) {
			Some(table_find) => {
				let mut pv = String::new();
				if board.is_legal(table_find.best_move.unwrap()) {
					board.play_unchecked(table_find.best_move.unwrap());
					pv = format!("{} {}", table_find.best_move.unwrap(), self.get_pv(board, depth - 1, ply + 1));
				}

				return pv;
			},
			None => {}
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

	fn get_nmp_reduction_amount(&self, depth: i32) -> i32 {
		//calculate nmp reduction amount
		//x = depth
		//y = reduction
		//y = base + (x - a) / b
		return Self::NMP_REDUCTION_BASE + (depth - Self::NMP_XSHIFT) / Self::NMP_YSTRETCH;
	}

	fn get_lmr_reduction_amount(&self, mut depth: i32, mut moves_searched: i32) -> i32 {
		return LMR_TABLE[usize::min(depth as usize, 63)][usize::min(moves_searched as usize, 63)] as i32; 
	}

	pub fn search(&mut self, abort: &AtomicBool, board: &Board, mut depth: i32, mut ply: i32, mut alpha: i32, mut beta: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;

		//MATE DISTANCE PRUNING
		//make sure that alpha is not defaulted to negative infinity
		if alpha != -i32::MAX && Score::CHECKMATE_BASE - ply <= alpha {
			return Some((None, Eval::new(Score::CHECKMATE_BASE - ply, true)));
		}

		let in_check = !board.checkers().is_empty();
		let is_pv = beta > alpha + 1;

		//CHECK EXTENSION
		if in_check {
			// https://www.chessprogramming.org/Check_Extensions
			depth += 1;
		}

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		if depth <= 0 {
			return self.qsearch(&abort, board, alpha, beta, ply, past_positions); //proceed with qSearch to avoid horizon effect
		}

		//check for three move repetition
		if self.is_repetition(board, past_positions) && ply > 0 {
			return Some((None, Eval::new(Score::DRAW, false)));
		}

		let mut legal_moves: Vec<SortedMove>;

		//probe tt
		let table_find = match self.tt.find(board, ply) {
			Some(table_find) => {
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

				Some(table_find)
			},
			None => {
				let mut iid_move = None;

				//Internal Iterative Deepening
				//We use the best move from a search with reduced depth to replace the hash move in move ordering if TT probe does not return a position

				//if sufficient depth
				//if PV node
				if depth >= Self::IID_DEPTH_MIN	&& beta > alpha + 1 {
					let iid_max_depth = depth / 4;
					let mut iid_depth = 1;

					while iid_depth <= iid_max_depth {
						let (best_mv, _) = self.search(&abort, board, iid_depth, ply, alpha, beta, past_positions)?;
						iid_move = best_mv;
						iid_depth += 1;
					}
				}

				legal_moves = self.movegen.move_gen(board, iid_move, ply);

				None
			}
		};

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
			let r = self.get_nmp_reduction_amount(depth);

			let nulled_board = board.clone().null_move().unwrap();
			let (_, mut null_score) = self.search(&abort, &nulled_board, depth - r - 1, ply + 1, -beta, -beta + 1, past_positions)?; //perform a ZW search

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
				let (_, mut child_eval) = self.search(&abort, &board_cache, depth - 1, ply + 1, -beta, -alpha, past_positions)?;
				child_eval.score *= -1;

				value = child_eval;
			} else {
				//LMP
				//We can skip specific quiet moves that are very late in a node
				//IF isn't PV
				//IF low depth
				//IF move is quiet
				//IF alpha is NOT a losing mate
				//IF IS late move
				//IF is NOT a check
				if !is_pv && depth <= Self::LMP_DEPTH_MAX && sm.movetype == MoveType::Quiet && alpha > -Score::CHECKMATE_DEFINITE && moves_searched > Self::LMP_MULTIPLIER * depth && !in_check {
					past_positions.pop();
					continue;
				}

				//See Pruning
				//IF move is loud (we do not do SEE with quiets)
				//IF depth is low
				//IF SEE is below 0
				//IF moves searched is above a limit
				//IF is NOT a check
				//IF is NOT a PV
				//IF is NOT a promotion
				if sm.movetype == MoveType::Loud && depth <= Self::SEE_PRUNE_DEPTH && sm.see_score < Self::SEE_PRUNE_THRESHOLD * depth && moves_searched > Self::SEE_PRUNE_MOVE_MULTIPLIER * depth && !in_check && !is_pv && sm.mv.promotion.is_none() {
					past_positions.pop();
					continue;
				}

				//LMR can be applied
				//IF depth is above sufficient depth
				//IF the first X searched are searched
				let apply_lmr = depth >= Self::LMR_DEPTH_LIMIT && moves_searched >= Self::LMR_FULL_SEARCHED_MOVE_LIMIT;

				//get initial value with reduction and pv-search null window
				let mut new_depth = depth;

				//LMR
				//reduce only if ISNT in check and ISNT a killer move
				if !in_check && !sm.is_killer && apply_lmr {
					new_depth = depth - self.get_lmr_reduction_amount(depth, moves_searched);
				}

				let (_, mut child_eval) = self.search(&abort, &board_cache, new_depth - 1, ply + 1, -alpha - 1, -alpha, past_positions)?;
				child_eval.score *= -1;

				value = child_eval;

				//check if lmr should be removed
				//search with full depth and null window
				if value.score > alpha && apply_lmr {
					let (_, mut child_eval) = self.search(&abort, &board_cache, depth - 1, ply + 1, -alpha - 1, -alpha, past_positions)?;
					child_eval.score *= -1;

					value = child_eval;	
				}

				//if PV
				//search with full depth and full window
				if value.score > alpha && value.score < beta {
					let (_, mut child_eval) = self.search(&abort, &board_cache, depth - 1, ply + 1, -beta, -alpha, past_positions)?;
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

	fn qsearch(&mut self, abort: &AtomicBool, board: &Board, mut alpha: i32, beta: i32, mut ply: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;

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

		let mut move_list: Vec<SortedMove>;

		//probe TT
		let table_find = match self.tt.find(board, ply) {
			Some(table_find) => {
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

				move_list = self.movegen.qmove_gen(board, table_find.best_move, ply);

				Some(table_find)
			},
			None => {
				move_list = self.movegen.qmove_gen(board, None, ply);

				None
			}
		};

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

			let (_, mut child_eval) = self.qsearch(&abort, &board_cache, -beta, -alpha, ply + 1, past_positions)?;

			past_positions.pop();

			child_eval.score *= -1;

			if child_eval.score > eval.score {
				eval = child_eval;
				best_move = Some(mv);
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						self.tt.insert(best_move, eval.score, board.hash(), ply, 0, NodeKind::LowerBound);
						return Some((None, Eval::new(beta, false)));
					} else {
						self.tt.insert(best_move, eval.score, board.hash(), ply, 0, NodeKind::Exact);
					}
				} else {
					self.tt.insert(best_move, eval.score, board.hash(), ply, 0, NodeKind::UpperBound);
				}
			}
		}

		return Some((best_move, eval));
	}
}

impl Engine {
	const ASPIRATION_WINDOW: i32 = 25;
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 100;
	const NMP_REDUCTION_BASE: i32 = 3;
	const NMP_XSHIFT: i32 = 2;
	const NMP_YSTRETCH: i32 = 4;
	const LMR_DEPTH_LIMIT: i32 = 2;
	const LMR_FULL_SEARCHED_MOVE_LIMIT: i32 = 3;
	const IID_DEPTH_MIN: i32 = 6;
	const LMP_DEPTH_MAX: i32 = 3;
	const LMP_MULTIPLIER: i32 = 10;
	const SEE_PRUNE_DEPTH: i32 = 3;
	const SEE_PRUNE_MOVE_MULTIPLIER: i32 = 5;
	const SEE_PRUNE_THRESHOLD: i32 = -600;
}