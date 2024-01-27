use cozy_chess::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::search::lmr_table::*;
use crate::eval::evaluator::*;
use crate::eval::score::*;
use crate::search::tt::*;
use crate::movegen::movesorter::*;
use crate::movegen::movegen::*;

pub struct SearchInfo {
	pub board: Board,
	pub depth: i32,
	pub alpha: i32,
	pub beta: i32,
	pub past_positions: Vec<u64>
}

pub struct Searcher<'a> {
	tt: &'a TT,
	nodes: u64,
	seldepth: i32,
	movegen: &'a mut MoveGen,
	searching_depth: i32,
	evals: [i32; 250]
}

impl Searcher<'_> {
	pub fn new(tt: &TT, movegen: &mut MoveGen, abort: Arc<AtomicBool>, mut search_info: SearchInfo) -> Option<(Option<Move>, Eval, u64, i32)> {
		let mut searcher = Searcher {
			tt: tt,
			nodes: 0,
			seldepth: 0,
			movegen: movegen,
			searching_depth: search_info.depth,
			evals: [0; 250]
		};
		let (mv, eval) = searcher.search(&abort, &search_info.board, search_info.depth, 0, search_info.alpha, search_info.beta, &mut search_info.past_positions, None)?;
	
		return Some((mv, eval, searcher.nodes, searcher.seldepth));
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

	fn get_nmp_reduction_amount(&self, depth: i32, diff: i32) -> i32 {
		//calculate nmp reduction amount
		return 2 + (depth / 3) + (diff / 128);
	}

	fn get_lmr_reduction_amount(&self, mut depth: i32, mut moves_searched: i32) -> i32 {
		return LMR_TABLE[usize::min(depth as usize, 63)][usize::min(moves_searched as usize, 63)] as i32; 
	}

	pub fn search(&mut self, abort: &AtomicBool, board: &Board, mut depth: i32, mut ply: i32, mut alpha: i32, mut beta: i32, past_positions: &mut Vec<u64>, last_move: Option<Move>) -> Option<(Option<Move>, Eval)> {
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

		let mut extended = false;
		let in_check = !board.checkers().is_empty();
		let is_pv = beta > alpha + 1;

		//CHECK EXTENSION
		if in_check {
			// https://www.chessprogramming.org/Check_Extensions
			extended = true;
			depth += 1;
		}

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		if depth <= 0 {
			return self.qsearch(&abort, board, alpha, beta, ply); //proceed with qSearch to avoid horizon effect
		}

		//check for three move repetition
		if self.is_repetition(board, past_positions) && ply > 0 {
			return Some((None, Eval::new(Score::DRAW, false)));
		}

		let mut legal_moves: Vec<SortedMove> = Vec::with_capacity(64);

		//probe tt
		let (table_find_move, iid_find_move) = match self.tt.find(board, ply) {
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

				(Some(table_find), None)
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
						let (best_mv, _) = self.search(&abort, board, iid_depth, ply, alpha, beta, past_positions, last_move)?;
						iid_move = best_mv;
						iid_depth += 1;
					}
				}

				//Internal Iterative Reduction
				//IF sufficient depth
				//There is NO Hash Move
				if depth >= ply / 4 + 2 {
					depth -= depth / 10 + 1;
				}

				(None, iid_move)
			}
		};

		//static eval for tuning methods
		let static_eval = if table_find_move.as_ref().is_some() {
			table_find_move.as_ref().unwrap().eval
		} else {
			evaluate(board)
		};

		self.evals[ply as usize] = static_eval;
		let improving = ply > 1 && self.evals[ply as usize] > self.evals[ply as usize - 2];

		//Reverse Futility Pruning
		/*
		// if depth isn't too deep
		// if NOT in check
		// THEN prune
		*/

		if depth <= Self::MAX_DEPTH_RFP && !in_check {
			if static_eval - (Self::MULTIPLIER_RFP * depth) - (!improving as i32 * 30) >= beta {
				return Some((None, Eval::new(static_eval, false)));
			}
		}

		//Null Move Pruning
		/*
		// if NOT root node
		// if last move is NOT null
		// if NOT in check
		// if board has non pawn material
		// if board can produce a beta cutoff
		// THEN prune
		*/

		let our_pieces = board.colors(board.side_to_move());
		let sliding_pieces = board.pieces(Piece::Rook) | board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
		let improving_nmp_check = if ply > 1 {
			self.evals[ply as usize] - self.evals[ply as usize - 2] > -100
		} else {
			true
		};

		if ply > 0 && !in_check && !(our_pieces & sliding_pieces).is_empty() && static_eval >= beta && improving_nmp_check {
			let r = self.get_nmp_reduction_amount(depth, static_eval - beta + (!improving as i32) * 30);

			let nulled_board = board.clone().null_move().unwrap();
			let (_, mut null_score) = self.search(&abort, &nulled_board, depth - r, ply + 1, -beta, -beta + 1, past_positions, None)?; //perform a ZW search

			null_score.score *= -1;
		
			if null_score.score >= beta {
				return Some((None, Eval::new(beta, false))); //return the lower bound produced by the fail high for this node since doing nothing in this position is insanely good
			}
		}

		let mut moves_searched = 0;
		let mut best_move = None;
		let mut eval = Eval::new(i32::MIN, false);

		//STAGED MOVEGEN
		//Check if TT moves produce a cutoff before generating moves to same time
		if table_find_move.is_some() || iid_find_move.is_some() {
			moves_searched += 1;

			let mv = if table_find_move.is_some() {
				table_find_move.clone().unwrap().best_move.unwrap()
			} else {
				iid_find_move.clone().unwrap()
			};

			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			past_positions.push(board_cache.hash());

			let (_, mut child_eval) = self.search(&abort, &board_cache, depth - 1, ply + 1, -beta, -alpha, past_positions, Some(mv))?;
			child_eval.score *= -1;

			past_positions.pop();

			eval = child_eval;
			best_move = Some(mv);

			let movetype = if (mv.to.bitboard() & board.colors(!board.side_to_move())).is_empty() {
				MoveType::Quiet
			} else {
				MoveType::Loud
			};
			let mut sm = SortedMove::new(mv, 0, movetype);

			if eval.score > alpha {
				alpha = eval.score;
				if alpha >= beta {
					self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::LowerBound);
					sm.insert_killer(&mut self.movegen.sorter, ply, board);
					sm.insert_history(&mut self.movegen.sorter, depth);
					return Some((best_move, eval));
				} else {
					self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::Exact);
				}
			} else {
				self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::UpperBound);
			}

			sm.decay_history(&mut self.movegen.sorter, depth);

			legal_moves = self.movegen.move_gen(board, Some(mv), ply, true, last_move);
		} else {
			legal_moves = self.movegen.move_gen(board, None, ply, false, last_move);
		}

		for mut sm in legal_moves {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			let move_is_check = !board_cache.checkers().is_empty();

			past_positions.push(board_cache.hash());

			let mut value: Eval;
			let mut new_depth = depth - 1;

			//Extensions

			//King Pawn Endgame Extension
			let non_pawns = board.pieces(Piece::Rook) | board.pieces(Piece::Bishop) | board.pieces(Piece::Queen) | board.pieces(Piece::Knight);
			if !(board.occupied() & non_pawns).is_empty() && (board_cache.occupied() & non_pawns).is_empty() {
				new_depth += 1;
			}

			if moves_searched == 0 {
				let (_, mut child_eval) = self.search(&abort, &board_cache, new_depth, ply + 1, -beta, -alpha, past_positions, Some(mv))?;
				child_eval.score *= -1;

				value = child_eval;
			} else {
				//Pruning

				//LMP
				//We can skip specific quiet moves that are very late in a node
				//IF isn't PV
				//IF low depth
				//IF move is quiet
				//IF alpha is NOT a losing mate
				//IF IS late move
				//IF is NOT a check
				if !is_pv && depth <= Self::LMP_DEPTH_MAX && sm.movetype == MoveType::Quiet && alpha > -Score::CHECKMATE_DEFINITE && moves_searched > (Self::LMP_MULTIPLIER * depth) - (!improving as i32 * 3) && !in_check {
					past_positions.pop();
					break;
				}

				//History Pruning
				if depth >= Self::HISTORY_DEPTH_MIN && sm.history < -500 * depth {
					past_positions.pop();
					continue;
				}

				//get initial value with reduction and pv-search null window
				let mut reduction = 0;

				//History Leaf Reduction
				reduction -= sm.history / 1500;

				//LMR can be applied
				//IF depth is above sufficient depth
				//IF the first X searched are searched
				if depth >= Self::LMR_DEPTH_LIMIT 
				&& moves_searched >= 2 
				&& (!is_pv || sm.movetype == MoveType::Quiet || !move_is_check) {
					reduction += self.get_lmr_reduction_amount(depth, moves_searched);
				}

				//Reduce less if PV node
				reduction -= is_pv as i32;

				//Underpromo Reduction
				if !mv.promotion.is_none() {
					if mv.promotion.unwrap() != Piece::Queen && depth >= Self::UNDERPROMO_REDUC_DEPTH {
						reduction += 1;
					}
				}

				//Passed Pawn Reduction
				let all_pawns = board.pieces(Piece::Pawn);
				let my_pawns = all_pawns & board.colors(board.side_to_move());
				let enemy_pawns = all_pawns & board.colors(!board.side_to_move());
				let ranks = Rank::Seventh.relative_to(board.side_to_move()).bitboard() | Rank::Sixth.relative_to(board.side_to_move()).bitboard();
				let pawn_on_ranks = my_pawns & ranks;
				let exists = !(mv.from.bitboard() & pawn_on_ranks).is_empty();
				if exists && is_pv {
					//pawn exists, check if it's a passer
					let promo_rank = Rank::Eighth.relative_to(board.side_to_move());
					let mut pawn_goal = Square::new(mv.from.file(), promo_rank);
					let mut checking_file = get_between_rays(mv.from, pawn_goal);
					let mut block_mask = checking_file;

					//use this handy dandy attack function to add files to the right and left of pawn
					for attack_location in get_pawn_attacks(mv.from, board.side_to_move()) {
						pawn_goal = Square::new(attack_location.file(), promo_rank);
						checking_file = get_between_rays(attack_location, pawn_goal); //check from the pawn

						//add file to the BB block mask
						block_mask |= checking_file | attack_location.bitboard();
					}

					//check to see if these three BB files contain enemy pawns in them && and if this is not a pawn island
					let passed = (enemy_pawns & block_mask).is_empty() && (my_pawns & get_between_rays(mv.from, Square::new(mv.from.file(), promo_rank))).is_empty();
					if passed {
						reduction -= 1;
					} else {
						reduction += 1;
					}
				}

				if reduction < 0 || in_check || sm.is_killer || sm.is_countermove {
					reduction = 0;
				}

				let (_, mut child_eval) = self.search(&abort, &board_cache, new_depth - reduction, ply + 1, -alpha - 1, -alpha, past_positions, Some(mv))?;
				child_eval.score *= -1;

				value = child_eval;

				//check if reductions should be removed
				//search with full depth and null window
				if value.score > alpha && reduction > 0 {
					let (_, mut child_eval) = self.search(&abort, &board_cache, new_depth, ply + 1, -alpha - 1, -alpha, past_positions, Some(mv))?;
					child_eval.score *= -1;

					value = child_eval;	
				}

				//if PV
				//search with full depth and full window
				if value.score > alpha && value.score < beta {
					let (_, mut child_eval) = self.search(&abort, &board_cache, new_depth, ply + 1, -beta, -alpha, past_positions, Some(mv))?;
					child_eval.score *= -1;		

					value = child_eval;	
				}
			}

			past_positions.pop();

			let mut do_spp = false;

			if value.score > eval.score {
				eval = value;
				best_move = Some(mv);
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::LowerBound);
						sm.insert_killer(&mut self.movegen.sorter, ply, board);
						sm.insert_history(&mut self.movegen.sorter, depth);
						sm.insert_countermove(&mut self.movegen.sorter, last_move);
						break;
					} else {
						self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::Exact);
					}
				} else {
					//SPP
					//IF is NOT PV
					//IF is of reasonable depth
					//IF move does NOT give check
					//IF is quiet move
					do_spp = !is_pv && depth <= Self::SPP_DEPTH_CAP && !move_is_check && sm.movetype == MoveType::Quiet;
					self.tt.insert(best_move, eval.score, board.hash(), ply, depth, NodeKind::UpperBound);
				}
			}

			sm.decay_history(&mut self.movegen.sorter, depth);

			if do_spp {
				break;
			}

			moves_searched += 1;
		}

		return Some((best_move, eval));
	}

	fn qsearch(&mut self, abort: &AtomicBool, board: &Board, mut alpha: i32, beta: i32, mut ply: i32) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.seldepth = i32::max(self.seldepth, ply);
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

			let (_, mut child_eval) = self.qsearch(&abort, &board_cache, -beta, -alpha, ply + 1)?;

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

impl Searcher<'_> {
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 80;
	const LMR_DEPTH_LIMIT: i32 = 2;
	const HISTORY_DEPTH_MIN: i32 = 5;
	const IID_DEPTH_MIN: i32 = 6;
	const LMP_DEPTH_MAX: i32 = 3;
	const LMP_MULTIPLIER: i32 = 5;
	const SPP_DEPTH_CAP: i32 = 3;
	const UNDERPROMO_REDUC_DEPTH: i32 = 4;
}