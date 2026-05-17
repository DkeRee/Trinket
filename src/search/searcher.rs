use cozy_chess::*;

use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::search::lmr_table::*;
use crate::eval::evaluator::*;
use crate::eval::score::*;
use crate::search::tt::*;
use crate::search::search_master::*;
use crate::movegen::movesorter::*;
use crate::movegen::movegen::*;
use crate::movegen::boardwrapper::*;

pub struct SearchInfo {
	pub boardwrapper: BoardWrapper,
	pub depth: i32,
	pub alpha: i32,
	pub beta: i32,
	pub past_positions: Vec<u64>
}

pub struct Searcher<'a> {
	pub time_control: TimeControl,
	pub shared_info: &'a SharedInfo<'a>,
	pub movegen: MoveGen,
	total_thread_count: u32,
	nodes: u64,
	boardwrapper: BoardWrapper,
	my_past_positions: Vec<u64>,
	evals: [i32; 300]
}

impl Searcher<'_> {
	pub fn create(time_control: TimeControl, shared_info: &SharedInfo, movegen: MoveGen, boardwrapper: BoardWrapper, my_past_positions: Vec<u64>, handler: Option<Arc<AtomicBool>>, total_thread_count: u32) -> (MoveGen, u64) {
		let mut instance = Searcher {
			time_control: time_control,
			shared_info: shared_info,
			movegen: movegen,
			total_thread_count: total_thread_count,
			nodes: 0,
			boardwrapper: boardwrapper,
			my_past_positions: my_past_positions,
			evals: [0; 300]
		};

		instance.go(handler.unwrap());
		(instance.movegen, instance.nodes)
	}

	pub fn go(&mut self, handler: Arc<AtomicBool>) {
		let now = Instant::now();

		let mut last_result = 0;
		let mut depth_index = 0;
		let mut window = 10;

		while depth_index < self.time_control.depth && depth_index < 250 {
			let boardwrapper = &mut self.boardwrapper.clone();
			let mut past_positions = self.my_past_positions.clone();

			let new_alpha = if depth_index + 1 > 3 {
				last_result - window
			} else {
				-i32::MAX
			};

			let new_beta = if depth_index + 1 > 3 {
				last_result + window
			} else {
				i32::MAX
			};

			let search_handler: Arc<AtomicBool> = handler.clone();

			let result = self.search(&search_handler, boardwrapper, depth_index + 1, 0, new_alpha, new_beta, &mut past_positions, None);

			if result != None {
				let (best_mv, eval) = result.unwrap();

				if eval.score <= last_result - window || eval.score >= last_result + window {
					window *= 2;
					continue;
				}

				window = 10;
				last_result = eval.score;

				let mut best_move = self.shared_info.best_move.lock().unwrap();
				let mut best_depth = self.shared_info.best_depth.lock().unwrap();
				let mut best_eval = self.shared_info.best_eval.lock().unwrap();

				depth_index += 1;

				let is_main_thread = depth_index > *best_depth || (depth_index == *best_depth && eval.score > *best_eval) || self.total_thread_count == 1;

				if is_main_thread {
					*best_move = best_mv.clone();
					*best_depth = depth_index;
					*best_eval = eval.score;
				}
				
				let mut time: u64;
				let mut timeinc: u64;

				let movetime = self.time_control.movetime;
				let movestogo = self.time_control.movestogo;
				
				//set time
				match self.boardwrapper.board.side_to_move() {
					Color::White => {
						time = self.time_control.wtime as u64;
						timeinc = self.time_control.winc as u64;
					},
					Color::Black => {
						time = self.time_control.btime as u64;
						timeinc = self.time_control.binc as u64;	
					}
				}

				let elapsed: f32 = now.elapsed().as_secs_f32() * 1000_f32;

				if time != u64::MAX {
					let mut soft_timeout = None;

					let mut soft_timeout_div = 25;
					if let Some(movestogo) = movestogo {
						soft_timeout_div /= movestogo / 10;
					}

					soft_timeout = Some((time + timeinc) / (soft_timeout_div) as u64);

					if movetime.is_none() && !soft_timeout.is_none() {
						if elapsed as u64 > soft_timeout.unwrap() {
							break;
						}
					}
				}

				if !is_main_thread {
					continue;
				}

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

				println!("info depth {} time {} score {} nodes {} nps {} pv {}", depth_index, elapsed as u64, score_str, self.nodes, nps, self.get_pv(&mut boardwrapper.board, depth_index, 0));
			} else {
				break;
			}
		}
	}

	//fish PV from TT
	fn get_pv(&self, board: &mut Board, depth: i32, ply: i32) -> String {
		if depth == 0 || ply > 50 {
			return String::new();
		}

		//probe TT
		match self.shared_info.tt.find(board, ply) {
			Some(table_find) => {
				let mut pv = String::new();
				if table_find.best_move.is_some() {
					if board.is_legal(table_find.best_move.unwrap()) {
						board.play_unchecked(table_find.best_move.unwrap());
						pv = format!("{} {}", table_find.best_move.unwrap(), self.get_pv(board, depth - 1, ply + 1));
					}
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

	fn get_nmp_reduction_amount(&self, depth: i32, diff: i32) -> i32 {
		//calculate nmp reduction amount
		return 2 + (depth / 3) + (diff / 128);
	}

	fn get_lmr_reduction_amount(&self, mut depth: i32, mut moves_searched: i32) -> i32 {
		return LMR_TABLE[usize::min(depth as usize, 63)][usize::min(moves_searched as usize, 63)] as i32; 
	}

	pub fn search(&mut self, abort: &AtomicBool, boardwrapper: &BoardWrapper, mut depth: i32, mut ply: i32, mut alpha: i32, mut beta: i32, past_positions: &mut Vec<u64>, last_move: Option<Move>) -> Option<(Option<Move>, Eval)> {		
		//abort?
		if self.time_control.depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;

		//MATE DISTANCE PRUNING
		//make sure that alpha is not defaulted to negative infinity
		if alpha != -i32::MAX && Score::CHECKMATE_BASE - ply <= alpha {
			return Some((None, Eval::new(Score::CHECKMATE_BASE - ply, true)));
		}

		let mut globally_extended = false;
		let in_check = !boardwrapper.board.checkers().is_empty();
		let is_pv = beta > alpha + 1;

		//CHECK EXTENSION
		if in_check {
			// https://www.chessprogramming.org/Check_Extensions
			globally_extended = true;
			depth += 1;
		}

		match boardwrapper.board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		if depth <= 0 {
			return self.qsearch(&abort, boardwrapper, alpha, beta, ply); //proceed with qSearch to avoid horizon effect
		}

		//check for three move repetition
		if self.is_repetition(&boardwrapper.board, past_positions) && ply > 0 {
			return Some((None, Eval::new(Score::DRAW, false)));
		}

		let mut legal_moves: Vec<SortedMove> = Vec::with_capacity(64);

		//probe tt
		let (tt_hit, iid) = match self.shared_info.tt.find(&boardwrapper.board, ply) {
			Some(table_find) => {
				//if sufficient depth
				if table_find.depth >= depth {
					//check if position from TT is a mate
					let mut is_checkmate = if table_find.eval < -Score::CHECKMATE_BASE || table_find.eval > Score::CHECKMATE_BASE {
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
					let (best_mv, _) = self.search(&abort, boardwrapper, depth - 10, ply, alpha, beta, past_positions, last_move)?;
					iid_move = best_mv;
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
		let static_eval = if tt_hit.as_ref().is_some() {
			tt_hit.as_ref().unwrap().eval
		} else {
			let base_eval = evaluate(&boardwrapper.board) as f32;
			let pawn_corrhist = self.movegen.sorter.read_pawn_corrhist(boardwrapper);
			let non_pawn_corrhist = self.movegen.sorter.read_non_pawn_corrhist(boardwrapper);
			let material_corrhist = self.movegen.sorter.read_material_corrhist(boardwrapper);

			(base_eval 
				+ pawn_corrhist 
				+ non_pawn_corrhist
				+ material_corrhist) as i32
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

		let our_pieces = boardwrapper.board.colors(boardwrapper.board.side_to_move());
		let sliding_pieces = boardwrapper.board.pieces(Piece::Rook) | boardwrapper.board.pieces(Piece::Bishop) | boardwrapper.board.pieces(Piece::Queen);
		let improving_nmp_check = if ply > 1 {
			self.evals[ply as usize] - self.evals[ply as usize - 2] > -100
		} else {
			true
		};

		if ply > 0 && !in_check && !(our_pieces & sliding_pieces).is_empty() && static_eval >= beta && improving_nmp_check {
			let r = self.get_nmp_reduction_amount(depth, static_eval - beta + (!improving as i32) * 30);

			let nulled_board = &boardwrapper.clone().null_move();
			
			let (_, mut null_score) = self.search(&abort, nulled_board, depth - r, ply + 1, -beta, -beta + 1, past_positions, None)?; //perform a ZW search

			null_score.score *= -1;
		
			if null_score.score >= beta {
				return Some((None, Eval::new(beta, false))); //return the lower bound produced by the fail high for this node since doing nothing in this position is insanely good
			}
		}

		let mut best_move = None;
		let mut best_move_type = None;
		let mut eval = Eval::new(i32::MIN, false);

		//STAGED MOVEGEN
		//Check if TT moves produce a cutoff before generating moves to same time
		let mut staged_movegen = tt_hit.is_some();
		if staged_movegen {
			let top_move = if tt_hit.is_some() {
				tt_hit.clone().unwrap().best_move
			} else if iid.is_some() {
				iid.clone()
			} else {
				None
			};

			if top_move.is_some() {
				let movetype = if (top_move.unwrap().to.bitboard() & boardwrapper.board.colors(!boardwrapper.board.side_to_move())).is_empty() {
					MoveType::Quiet
				} else {
					MoveType::Loud
				};
				let mut sm = SortedMove::new(top_move.unwrap(), 0, movetype);
	
				legal_moves.push(sm);
			} else {
				staged_movegen = false;
				legal_moves = self.movegen.move_gen(&boardwrapper.board, None, ply, false, last_move);
			}
		} else {
			legal_moves = self.movegen.move_gen(&boardwrapper.board, None, ply, false, last_move);
		}

		let mut moves_searched = 0;
		let mut legal_index = 0;
		let mut tt_nodetype = NodeKind::UpperBound;

		while legal_index < legal_moves.len() {
			let mut mvlen = legal_moves.len() as i32;
			let mut sm = &mut legal_moves[legal_index];
			let mv = sm.mv;
			let mut board_wrapper_cache = boardwrapper.clone();
				
			board_wrapper_cache.play_unchecked(sm);

			let move_is_check = !board_wrapper_cache.board.checkers().is_empty();

			past_positions.push(board_wrapper_cache.board.hash());

			let mut value: Eval;
			let mut new_depth = depth - 1;

			//Extensions

			//King Pawn Endgame Extension
			let non_pawns = boardwrapper.board.pieces(Piece::Rook) | boardwrapper.board.pieces(Piece::Bishop) | boardwrapper.board.pieces(Piece::Queen) | boardwrapper.board.pieces(Piece::Knight);
			if !(boardwrapper.board.occupied() & non_pawns).is_empty() && (board_wrapper_cache.board.occupied() & non_pawns).is_empty() && !globally_extended && !staged_movegen {
				new_depth += 1;
			}

			if moves_searched == 0 {
				let (_, mut child_eval) = self.search(&abort, &board_wrapper_cache, new_depth, ply + 1, -beta, -alpha, past_positions, Some(mv))?;
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
				if !is_pv && depth <= Self::LMP_DEPTH_MAX 
				&& sm.movetype == MoveType::Quiet 
				&& alpha > -Score::CHECKMATE_BASE 
				&& moves_searched > ((mvlen / 6) * depth) - (!improving as i32 * 3)
				&& !in_check {
					past_positions.pop();
					break;
				}

				//History Pruning
				if depth >= Self::HISTORY_DEPTH_MIN && sm.history < -500 * depth {
					past_positions.pop();
					legal_index += 1;
					continue;
				}

				//get initial value with reduction and pv-search null window
				let mut reduction = 0;

				//History Leaf Reduction
				reduction -= sm.history / 1500;

				//LMR can be applied
				//IF depth is above sufficient depth
				//IF the first X searched are searched
				if moves_searched >= 2 
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
				let all_pawns = boardwrapper.board.pieces(Piece::Pawn);
				let my_pawns = all_pawns & boardwrapper.board.colors(boardwrapper.board.side_to_move());
				let enemy_pawns = all_pawns & boardwrapper.board.colors(!boardwrapper.board.side_to_move());
				let ranks = Rank::Seventh.relative_to(boardwrapper.board.side_to_move()).bitboard() | Rank::Sixth.relative_to(boardwrapper.board.side_to_move()).bitboard();
				let pawn_on_ranks = my_pawns & ranks;
				let exists = !(mv.from.bitboard() & pawn_on_ranks).is_empty();
				if exists && is_pv {
					//pawn exists, check if it's a passer
					let promo_rank = Rank::Eighth.relative_to(boardwrapper.board.side_to_move());
					let mut pawn_goal = Square::new(mv.from.file(), promo_rank);
					let mut checking_file = get_between_rays(mv.from, pawn_goal);
					let mut block_mask = checking_file;

					//use this handy dandy attack function to add files to the right and left of pawn
					for attack_location in get_pawn_attacks(mv.from, boardwrapper.board.side_to_move()) {
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

				let (_, mut child_eval) = self.search(&abort, &board_wrapper_cache, new_depth - reduction, ply + 1, -alpha - 1, -alpha, past_positions, Some(mv))?;
				child_eval.score *= -1;

				value = child_eval;

				//check if reductions should be removed
				//search with full depth and null window
				if value.score > alpha && reduction > 0 {
					let (_, mut child_eval) = self.search(&abort, &board_wrapper_cache, new_depth, ply + 1, -alpha - 1, -alpha, past_positions, Some(mv))?;
					child_eval.score *= -1;

					value = child_eval;	
				}

				//if PV
				//search with full depth and full window
				if value.score > alpha && value.score < beta {
					let (_, mut child_eval) = self.search(&abort, &board_wrapper_cache, new_depth, ply + 1, -beta, -alpha, past_positions, Some(mv))?;
					child_eval.score *= -1;		

					value = child_eval;	
				}
			}

			past_positions.pop();

			let mut do_spp = false;

			if value.score > eval.score {
				eval = value;
				best_move = Some(mv);
				best_move_type = Some(sm.movetype.clone());
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						tt_nodetype = NodeKind::LowerBound;
						sm.insert_killer(&mut self.movegen.sorter, ply, &boardwrapper.board);
						sm.insert_history(&mut self.movegen.sorter, depth);
						sm.insert_countermove(&mut self.movegen.sorter, last_move);
						break;
					} else {
						tt_nodetype = NodeKind::Exact;
					}
				} else {
					//SPP
					do_spp = !is_pv 
					&& depth <= Self::SPP_DEPTH_CAP 
					&& !move_is_check 
					&& !sm.is_killer
					&& !sm.is_countermove
					&& sm.movetype == MoveType::Quiet
					&& !staged_movegen;
				}
			}

			sm.decay_history(&mut self.movegen.sorter, depth);

			if do_spp {
				break;
			}

			moves_searched += 1;
			legal_index += 1;

			if staged_movegen && legal_index >= legal_moves.len() {
				staged_movegen = false;
				legal_index = 0; 
				legal_moves = self.movegen.move_gen(&boardwrapper.board, Some(mv), ply, true, last_move);
			}
		}

		self.shared_info.tt.insert(best_move, eval.score, boardwrapper.board.hash(), ply, depth, tt_nodetype);

		if best_move_type.is_some() {
			if best_move_type.unwrap() == MoveType::Quiet
			&& ( (tt_nodetype == NodeKind::UpperBound && eval.score < static_eval) || (tt_nodetype == NodeKind::LowerBound && eval.score > static_eval) ) {
				self.movegen.sorter.add_pawn_corrhist(boardwrapper, depth, eval.score, static_eval);
				self.movegen.sorter.add_non_pawn_corrhist(boardwrapper, depth, eval.score, static_eval);
				self.movegen.sorter.add_material_corrhist(boardwrapper, depth, eval.score, static_eval);
			}
		}

		return Some((best_move, eval));
	}

	fn qsearch(&mut self, abort: &AtomicBool, boardwrapper: &BoardWrapper, mut alpha: i32, beta: i32, mut ply: i32) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.time_control.depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;

		match boardwrapper.board.status() {
			GameStatus::Won => return Some((None, Eval::new(-Score::CHECKMATE_BASE + ply, true))),
			GameStatus::Drawn => return Some((None, Eval::new(Score::DRAW, false))),
			GameStatus::Ongoing => {}
		}

		let base_eval = evaluate(&boardwrapper.board) as f32;
		let pawn_corrhist = self.movegen.sorter.read_pawn_corrhist(boardwrapper);
		let non_pawn_corrhist = self.movegen.sorter.read_non_pawn_corrhist(boardwrapper);
		let material_corrhist = self.movegen.sorter.read_material_corrhist(boardwrapper);
		let stand_pat = Eval::new((
			base_eval + pawn_corrhist + non_pawn_corrhist + material_corrhist
		) as i32, false);

		//beta cutoff
		if stand_pat.score >= beta {
			return Some((None, Eval::new(beta, false)));
		}

		if alpha < stand_pat.score {
			alpha = stand_pat.score;
		}

		let mut move_list: Vec<SortedMove>;

		//probe TT
		let table_find = match self.shared_info.tt.find(&boardwrapper.board, ply) {
			Some(table_find) => {
				//check if position from TT is a mate
				let mut is_checkmate = if table_find.eval < -Score::CHECKMATE_BASE || table_find.eval > Score::CHECKMATE_BASE {
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

				move_list = self.movegen.qmove_gen(&boardwrapper.board, table_find.best_move, ply);

				Some(table_find)
			},
			None => {
				move_list = self.movegen.qmove_gen(&boardwrapper.board, None, ply);

				None
			}
		};

		//no more loud moves to be checked anymore, it can be returned safely
		if move_list.len() == 0 {
			return Some((None, stand_pat));
		}

		let mut best_move = None;
		let mut eval = stand_pat;
		let mut tt_nodetype = NodeKind::UpperBound;

		for mut sm in move_list {

			//prune losing captures found through SEE swap algorithm
			if sm.importance < 0 {
				break;
			}

			let mv = sm.mv;
			let mut board_wrapper_cache = boardwrapper.clone();
			board_wrapper_cache.play_unchecked(&mut sm);

			let (_, mut child_eval) = self.qsearch(&abort, &board_wrapper_cache, -beta, -alpha, ply + 1)?;

			child_eval.score *= -1;

			let mut v_score = child_eval.score;
			if v_score > eval.score {
				eval = child_eval;
				best_move = Some(mv);
				if eval.score > alpha {
					alpha = eval.score;
					if alpha >= beta {
						tt_nodetype = NodeKind::LowerBound;
						break;
					} else {
						tt_nodetype = NodeKind::Exact
					}
				} else {
					tt_nodetype = NodeKind::UpperBound;
				}
			}
		}

		self.shared_info.tt.insert(best_move, eval.score, boardwrapper.board.hash(), ply, 0, tt_nodetype);

		return Some((best_move, eval));
	}
}

impl Searcher<'_> {
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 80;
	const HISTORY_DEPTH_MIN: i32 = 5;
	const IID_DEPTH_MIN: i32 = 6;
	const LMP_DEPTH_MAX: i32 = 3;
	const SPP_DEPTH_CAP: i32 = 3;
	const UNDERPROMO_REDUC_DEPTH: i32 = 4;
}