#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::eval::evaluator::*;
use crate::search::tt::*;

pub struct Engine {
	pub board: Board,
	pub max_depth: i32,
	pub my_past_positions: Vec<u64>,
	pub wtime: u64,
	pub btime: u64,
	searching_depth: i32,
	nodes: u64,
	pv: [[Option<Move>; 100]; 100],
	evaluator: Evaluator,
	tt: TT
}

impl Engine {
	pub fn new() -> Engine {
		Engine {
			board: Board::default(),
			max_depth: 0,
			my_past_positions: Vec::with_capacity(64),
			wtime: 300000,
			btime: 300000,
			searching_depth: 0,
			nodes: 0,
			pv: [[None; 100]; 100],
			evaluator: Evaluator::new(),
			tt: TT::new()
		}
	}

	pub fn go(&mut self) -> String {
		let now = Instant::now();

		let mut best_move = None;
		let mut time: f32;

		if self.board.side_to_move() == Color::White {
			time = self.wtime as f32;
		} else {
			time = self.btime as f32;
		}

		for depth_index in 0..self.max_depth {
			let search_elapsed = now.elapsed().as_secs_f32() * 1000_f32;
			if search_elapsed < time / 50_f32 {
				self.nodes = 0;
				self.searching_depth = depth_index + 1;

				let search_time = Instant::now();
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

				let result = self.search(&search_abort, board, self.searching_depth, -i32::MAX, i32::MAX);

				if result != None {
					let (best_mv, eval) = result.unwrap();
					best_move = best_mv.clone();

					let elapsed = now.elapsed().as_secs_f32() * 1000_f32;

					let mut nps: u64;
					if elapsed == 0_f32 {
						nps = self.nodes;
					} else {
						nps = ((self.nodes as f32 * 1000_f32) / elapsed) as u64;
					}

					let mut pv = String::new();

					for i in 0..self.pv[0].len() {
						if self.pv[0][i] != None {
							pv += &(self.parse_to_uci(self.pv[0][i]) + " ");
						}
					}

					println!("{}", String::from("info depth ") + &self.searching_depth.to_string() + &String::from(" nodes ") + &self.nodes.to_string() + &String::from(" pv ") + &pv.trim().to_string() + &String::from(" score cp ") + &eval.to_string() + &String::from(" nps ") + &nps.to_string());
				} else {
					break;
				}
			} else {
				break;
			}
		}

		self.board.play_unchecked(best_move.unwrap());
		self.my_past_positions.push(self.board.hash());

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

		self.parse_to_uci(best_move)
	}

	fn parse_to_uci(&self, mv: Option<Move>) -> String {
		let mv_parsed = mv.unwrap();
		let mut uci_mv = String::new();

		uci_mv += &mv_parsed.from.to_string();
		uci_mv += &mv_parsed.to.to_string();

		if mv_parsed.promotion != None {
			uci_mv += &mv_parsed.promotion.unwrap().to_string();
		}

		uci_mv
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
		for piece in piece_type {
			piece_amount += 1;
		}
		piece_amount
	}

	fn qsearch(&mut self, abort: &AtomicBool, board: &Board, mut alpha: i32, beta: i32, mut ply: i32) -> Option<(Option<Move>, i32)> {
		//abort?
		if self.searching_depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;
		ply += 1;

		match board.status() {
			GameStatus::Won => return Some((None, -30000 + ply)),
			GameStatus::Drawn => return Some((None, 0)),
			GameStatus::Ongoing => {}
		}

		//check for three move repetition
		if self.my_past_positions.len() > 6 {
			let curr_hash = board.hash();
			if curr_hash == self.my_past_positions[self.my_past_positions.len() - 4] {
				return Some((None, 0));
			}
		}

		let stand_pat = self.evaluator.evaluate(board);

		//checking to see if the move the opponent makes is in our favor, if so, just return it no checking necessary
		if stand_pat >= beta {
			return Some((None, beta));
		}

		//new best move eval
		if alpha < stand_pat {
			alpha = stand_pat;
		}

		let move_list = self.evaluator.qmove_gen(board);

		//no more loud moves to be checked anymore, it can be returned safely
		if move_list.len() == 0 {
			return Some((None, stand_pat));
		}

		let mut best_move = None;
		let mut eval = i32::MIN;
		for sm in move_list {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			let (_, mut child_eval) = self.qsearch(&abort, &board_cache, -beta, -alpha, ply)?;
			child_eval *= -1;
			if child_eval > eval {
				eval = child_eval;
				best_move = Some(mv);
				if eval > alpha {
					self.update_pv(best_move, ply as usize);
					alpha = eval;
					if alpha >= beta {
						break;
					}
				}
			}
		}

		return Some((best_move, alpha));
	}

	fn add_to_front(&self, legal_moves: &mut Vec<SortedMove>, best_move: Option<Move>) {
		let sm = SortedMove::new(best_move.unwrap(), 1000);
		let index = legal_moves.iter().position(|x| x.mv == best_move.unwrap()).unwrap();
		legal_moves.remove(index);
		legal_moves.insert(0, sm);
	}

	fn search(&mut self, abort: &AtomicBool, board: &Board, depth: i32, mut alpha: i32, beta: i32) -> Option<(Option<Move>, i32)> {
		//abort?
		if self.searching_depth > 1 && abort.load(Ordering::Relaxed) {
			return None;
		}

		self.nodes += 1;
		let mut legal_moves = self.evaluator.move_gen(board);

		match board.status() {
			GameStatus::Won => return Some((None, -30000 + (self.searching_depth - depth))),
			GameStatus::Drawn => return Some((None, 0)),
			GameStatus::Ongoing => {}
		}

		//check for three move repetition
		if self.my_past_positions.len() > 6 {
			let curr_hash = board.hash();
			if curr_hash == self.my_past_positions[self.my_past_positions.len() - 4] {
				return Some((None, 0));
			}
		}

		//look up tt
		let table_find = self.tt.find(board.hash(), self.searching_depth, depth);
		if board.hash() == table_find.position {
			//if sufficient depth and NOT pv node
			if table_find.depth >= depth && alpha == beta - 1 {
				if table_find.node_kind == NodeKind::Exact {
					return Some((table_find.best_move, table_find.eval));
				} else if table_find.node_kind == NodeKind::UpperBound {
					if table_find.eval <= alpha {
						return Some((table_find.best_move, table_find.eval));	
					}
				} else if table_find.node_kind == NodeKind::LowerBound {
					if table_find.eval >= beta {
						return Some((table_find.best_move, table_find.eval));	
					}
				}
			}
			self.add_to_front(&mut legal_moves, table_find.best_move);
		}

		if depth == 0 {
			return self.qsearch(&abort, board, alpha, beta, self.searching_depth);
		}

		let mut best_move = None;
		let mut eval = i32::MIN;
		for sm in legal_moves {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);

			let (_, mut child_eval) = self.search(&abort, &board_cache, depth - 1, -beta, -alpha)?;
			child_eval *= -1;
			if child_eval > eval {
				eval = child_eval;
				best_move = Some(mv);
				if eval > alpha {
					self.update_pv(best_move, (self.searching_depth - depth) as usize);
					alpha = eval;
					if alpha >= beta {
						self.tt.insert(best_move, eval, board.hash(), self.searching_depth, depth, NodeKind::LowerBound);
						break;
					} else {
						self.tt.insert(best_move, eval, board.hash(), self.searching_depth, depth, NodeKind::Exact);
					}
				} else {
					self.tt.insert(best_move, eval, board.hash(), self.searching_depth, depth, NodeKind::UpperBound);
				}
			}
		}

		return Some((best_move, eval));
	}
}