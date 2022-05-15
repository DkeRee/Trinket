use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::eval::evaluator::*;
use crate::search::tt::*;
use crate::movegen::movegen::*;
use crate::uci::castle_parse::*;

pub struct Engine {
	pub board: Board,
	pub max_depth: i32,
	pub my_past_positions: Vec<u64>,
	searching_depth: i32,
	nodes: u64,
	pv: [[Option<Move>; 100]; 100],
	evaluator: Evaluator,
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
			pv: [[None; 100]; 100],
			evaluator: Evaluator::new(),
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

				//set up pv table
				self.pv = [[None; 100]; 100];

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

					//get pv
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

					let mut score_str = if eval.mate {
						let mut mate_score = if eval.score > 0 {
							(eval.mate_ply - (eval.mate_ply / 2)) as i32
						} else {
							(eval.mate_ply - (eval.mate_ply / 2)) as i32 * -1
						};

						format!("mate {}", mate_score)
					} else {
						format!("cp {}", eval.score)
					};

					println!("info depth {} nodes {} pv {} score {} nps {}", self.searching_depth, self.nodes, pv.trim(), score_str, nps);
				} else {
					break;
				}
			} else {
				break;
			}
		}

		//detect if endgame via tapered eval
		//source: https://www.chessprogramming.org/Tapered_Eval
		let pawn_phase = 0;
		let knight_phase = 1;
		let bishop_phase = 1;
		let rook_phase = 2;
		let queen_phase = 4;
		let total_phase = pawn_phase * 16 + knight_phase * 4 + bishop_phase * 4 + rook_phase * 4 + queen_phase * 2;

		let mut phase = total_phase;

		let pawns = self.board.pieces(Piece::Pawn);
		let knights = self.board.pieces(Piece::Knight);
		let bishops = self.board.pieces(Piece::Bishop);
		let rooks = self.board.pieces(Piece::Rook);
		let queens = self.board.pieces(Piece::Queen);

		phase -= self.get_piece_amount(pawns) * pawn_phase;
		phase -= self.get_piece_amount(knights) * knight_phase;
		phase -= self.get_piece_amount(bishops) * bishop_phase;
		phase -= self.get_piece_amount(rooks) * rook_phase;
		phase -= self.get_piece_amount(queens) * queen_phase;

		phase = (phase * 256 + (total_phase / 2)) / total_phase;
		
		if phase > 145 {
			self.evaluator.end_game = true;
		}

		_960_to_regular_(best_move, &self.board)
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

	fn update_pv(&mut self, mv: Option<Move>, ply: usize) {
		self.pv[ply][0] = mv;
		for i in 0..self.pv[ply + 1].len() {
			if i + 1 != self.pv[ply].len() {
				self.pv[ply][i + 1] = self.pv[ply + 1][i];
			}
		}
	}

	fn get_piece_amount(&self, piece_type: BitBoard) -> usize {
		let mut piece_amount = 0;
		for _piece in piece_type {
			piece_amount += 1;
		}
		piece_amount
	}

	fn qsearch(&mut self, abort: &AtomicBool, stop_abort: &AtomicBool, board: &Board, mut alpha: i32, beta: i32, mut ply: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && (abort.load(Ordering::Relaxed) || stop_abort.load(Ordering::Relaxed)) {
			return None;
		}

		self.nodes += 1;
		self.pv[ply as usize] = [None; 100];
		ply += 1;

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-30000 + ply, true, ply as usize))),
			GameStatus::Drawn => return Some((None, Eval::new(0, false, 0))),
			GameStatus::Ongoing => {}
		}

		//check for three move repetition
		if self.is_repetition(board, past_positions) {
			return Some((None, Eval::new(0, false, 0)));
		}

		let stand_pat = Eval::new(self.evaluator.evaluate(board), false, 0);

		//beta cutoff
		if stand_pat.score >= beta {
			return Some((None, Eval::new(beta, false, 0)));
		}

		if alpha < stand_pat.score {
			alpha = stand_pat.score;
		}

		let move_list = self.movegen.qmove_gen(board);

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
					self.update_pv(best_move, ply as usize);
					alpha = eval.score;
					if alpha >= beta {
						return Some((None, Eval::new(beta, false , 0)));
					}
				}
			}
		}

		return Some((best_move, eval));
	}

	fn search(&mut self, abort: &AtomicBool, stop_abort: &AtomicBool, board: &Board, depth: i32, mut alpha: i32, beta: i32, past_positions: &mut Vec<u64>) -> Option<(Option<Move>, Eval)> {
		//abort?
		if self.searching_depth > 1 && (abort.load(Ordering::Relaxed) || stop_abort.load(Ordering::Relaxed)) {
			return None;
		}

		self.nodes += 1;
		self.pv[(self.searching_depth - depth) as usize] = [None; 100];
		let mut legal_moves: Vec<SortedMove>;

		match board.status() {
			GameStatus::Won => return Some((None, Eval::new(-30000 + (self.searching_depth - depth), true, (self.searching_depth - depth) as usize))),
			GameStatus::Drawn => return Some((None, Eval::new(0, false, 0))),
			GameStatus::Ongoing => {}
		}

		//look up tt
		
		let table_find = self.tt.find(board.hash(), self.searching_depth, depth);
		if board.hash() == table_find.position {
			//if sufficient depth and NOT pv node
			if table_find.depth >= depth && alpha == beta - 1 {
				match table_find.node_kind {
					NodeKind::Exact => {
						return Some((table_find.best_move, Eval::new(table_find.eval, false, 0)));
					},
					NodeKind::UpperBound => {
						if table_find.eval <= alpha {
							return Some((table_find.best_move, Eval::new(table_find.eval, false, 0)));
						}	
					},
					NodeKind::LowerBound => {
						if table_find.eval >= beta {
							return Some((table_find.best_move, Eval::new(table_find.eval, false, 0)));
						}
					},
					NodeKind::Null => {}
				}
			}
			legal_moves = self.movegen.move_gen(board, table_find.best_move);
		} else {
			legal_moves = self.movegen.move_gen(board, None);
		}
		

		//reverse futility pruning
		/*
		// if depth isn't too deep
		// if NOT in check
		// if NON-PV node
		// if NOT a checkmate
		// THEN prune
		*/
		if depth <= Self::MAX_DEPTH_RFP && board.checkers() == BitBoard::EMPTY && alpha == beta - 1 {
			let eval = self.evaluator.evaluate(board);
			if eval - (Self::MULTIPLIER_RFP * depth) >= beta {
				return Some((None, Eval::new(eval, false, 0)));
			}
		}

		if depth == 0 {
			return self.qsearch(&abort, &stop_abort, board, alpha, beta, self.searching_depth, past_positions);
		}

		//check for three move repetition
		if self.is_repetition(board, past_positions) && self.searching_depth - depth > 0 {
			return Some((None, Eval::new(0, false, 0)));
		}

		let mut best_move = None;
		let mut eval = Eval::new(i32::MIN, false, 0);
		for sm in legal_moves {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			past_positions.push(board_cache.hash());

			let (_, mut child_eval) = self.search(&abort, &stop_abort, &board_cache, depth - 1, -beta, -alpha, past_positions)?;

			past_positions.pop();

			child_eval.score *= -1;

			if child_eval.score > eval.score {
				eval = child_eval;
				best_move = Some(mv);
				if eval.score > alpha {
					self.update_pv(best_move, (self.searching_depth - depth) as usize);
					alpha = eval.score;
					if alpha >= beta {
						self.tt.insert(best_move, eval.score, board.hash(), self.searching_depth, depth, NodeKind::LowerBound);
						break;
					} else {
						self.tt.insert(best_move, eval.score, board.hash(), self.searching_depth, depth, NodeKind::Exact);
					}
				} else {
					self.tt.insert(best_move, eval.score, board.hash(), self.searching_depth, depth, NodeKind::UpperBound);
				}
			}
		}

		return Some((best_move, eval));
	}
}

impl Engine {
	const MAX_DEPTH_RFP: i32 = 6;
	const MULTIPLIER_RFP: i32 = 100;
}