use cozy_chess::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::search::search_master::*;
use crate::eval::evaluator::*;
use crate::eval::score::*;
use crate::search::tt::*;
use crate::movegen::movesorter::*;
use crate::movegen::movegen::*;

pub static mut LMR_TABLE: [[f32; 64]; 64] = [[0.0; 64]; 64];

pub struct Searcher<'a> {
	pub nodes: u64,
	searching_depth: i32,
	shared_tables: &'a SharedTables,
	local_tables: &'a mut LocalTables
}

impl Searcher<'_> {
	pub fn new(board: &Board, shared_tables: &SharedTables, local_tables: &mut LocalTables, handler: &Arc<AtomicBool>, depth: i32, eval: Option<Eval>) -> Option<(Option<Move>, Eval, u64)> {
		let mut instance = Searcher {
			nodes: 0,
			searching_depth: depth,
			shared_tables: shared_tables,
			local_tables: local_tables
		};

		let mut past_positions = instance.local_tables.my_past_positions.clone();

		//ASPIRATION WINDOWS ALPHA BETA
		let mut alpha = -i32::MAX;
		let mut beta = i32::MAX;

		//Adjust alpha and beta params based off of previous eval
		if !eval.is_none() {
			let score = eval.unwrap().score;

			alpha = score - Self::ASPIRATION_WINDOW;
			beta = score + Self::ASPIRATION_WINDOW;
		}

		loop {
			let result = instance.search(handler, board, depth, 0, alpha, beta, &mut past_positions);

			if !result.is_none() {
				let (mv, eval) = result.unwrap();

				//MANAGE ASPIRATION WINDOWS
				if eval.score >= beta {
					beta += Self::ASPIRATION_WINDOW * 4;
				} else if eval.score <= alpha {
					alpha -= Self::ASPIRATION_WINDOW * 4;
				} else {
					//search is within aspiration window, safe to return
					return Some((mv, eval, instance.nodes));
				}
			} else {
				//aborted due to outside context
				return None;
			}
		}
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

	fn get_lmr_reduction_amount(&self, mut depth: i32, mut moves_searched: i32) -> i32 {
		unsafe { 
			return LMR_TABLE[usize::min(depth as usize, 63)][usize::min(moves_searched as usize, 63)] as i32; 
		}
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

		let is_pv = beta > alpha + 1;
		let mut legal_moves: Vec<SortedMove>;

		//probe tt
		let table_find = match self.shared_tables.tt.find(board, ply) {
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

				legal_moves = move_gen(board);
				sort(&mut self.local_tables.sorter, &mut legal_moves, table_find.best_move, board, ply);

				Some(table_find)
			},
			None => {
				let mut iid_move = None;

				//Internal Iterative Deepening
				//We use the best move from a search with reduced depth to replace the hash move in move ordering if TT probe does not return a position

				//if sufficient depth
				//if PV node
				if depth >= Self::IID_DEPTH_MIN	&& is_pv {
					let iid_max_depth = depth / 4;
					let mut iid_depth = 1;

					while iid_depth <= iid_max_depth {
						let (best_mv, _) = self.search(&abort, board, iid_depth, ply, alpha, beta, past_positions)?;
						iid_move = best_mv;
						iid_depth += 1;
					}
				}

				legal_moves = move_gen(board);
				sort(&mut self.local_tables.sorter, &mut legal_moves, iid_move, board, ply);

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
			let r = if depth > 6 {
				3
			} else {
				2
			};

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
				//LMR can be applied
				//IF depth is above sufficient depth
				//IF the first X searched are searched
				//IF this move is QUIET
				if depth >= Self::LMR_DEPTH_LIMIT && moves_searched >= Self::LMR_FULL_SEARCHED_MOVE_LIMIT && sm.movetype == MoveType::Quiet {
					let reduction_amount = depth - self.get_lmr_reduction_amount(depth, moves_searched);
					let (_, mut child_eval) = self.search(&abort, &board_cache, reduction_amount - 1, ply + 1, -alpha - 1, -alpha, past_positions)?;
					child_eval.score *= -1;		

					value = child_eval;	
				} else {
					//make sure it searches at full depth in the next step
					value = Eval::new(alpha + 1, false);
				}

				//if a value ever surprises us in the future with a score that ACTUALLY changes the lowerbound...we have to search at full depth, for this move may possibly be good
				if value.score > alpha {
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
						self.shared_tables.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::LowerBound);
						self.local_tables.sorter.add_killer(&sm, ply, board);
						self.local_tables.sorter.add_history(&sm, depth);
						break;
					} else {
						self.shared_tables.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::Exact);
					}
				} else {
					self.shared_tables.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::UpperBound);
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

		let mut move_list = qmove_gen(board);
		sort(&mut self.local_tables.sorter, &mut move_list, None, board, ply);

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
						return Some((None, Eval::new(beta, false)));
					}
				}
			}
		}

		return Some((best_move, eval));
	}
}

impl Searcher<'_> {
	const ASPIRATION_WINDOW: i32 = 25;
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 100;
	const LMR_DEPTH_LIMIT: i32 = 3;
	const LMR_FULL_SEARCHED_MOVE_LIMIT: i32 = 4;
	const LMR_REDUCTION_BASE: f32 = 0.75;
	const LMR_MOVE_DIVIDER: f32 = 2.25;
	const IID_DEPTH_MIN: i32 = 6;
}

pub fn init_lmr_table() {
	for depth in 1..64 {
		for played_move in 1..64 {
			unsafe {
				LMR_TABLE[depth][played_move] = Searcher::LMR_REDUCTION_BASE + f32::ln(depth as f32) * f32::ln(played_move as f32) / Searcher::LMR_MOVE_DIVIDER;
			}
		}
	}
}