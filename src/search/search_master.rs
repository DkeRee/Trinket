#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

use cozy_chess::*;
use std::time::Instant;
use crate::eval::evaluator::*;

pub struct Engine {
	pub board: Board,
	pub searching_depth: i32,
	pub my_past_positions: Vec<u64>,
	nodes: u64,
	pv: [[Option<Move>; 100]; 100],
	evaluator: Evaluator
}

impl Engine {
	pub fn new() -> Engine {
		Engine {
			board: Board::default(),
			searching_depth: 0,
			my_past_positions: Vec::with_capacity(64),
			nodes: 0,
			pv: [[None; 100]; 100],
			evaluator: Evaluator::new()
		}
	}

	pub fn go(&mut self) -> (String, String, i32, u64, u64) {
		self.nodes = 0;

		let now = Instant::now();

		let board = &self.board.clone();

		//set up pv table
		self.pv = [[None; 100]; 100];

		let (best_move, eval) = self.search(board, self.searching_depth, -i32::MAX, i32::MAX);
		
		let elapsed = now.elapsed().as_secs() * 1000;

		let mut nps: u64;
		if elapsed == 0 {
			nps = self.nodes;
		} else {
			nps = (self.nodes * 1000) / elapsed;
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

		let pawns = board.pieces(Piece::Pawn);
		let knights = board.pieces(Piece::Knight);
		let bishops = board.pieces(Piece::Bishop);
		let rooks = board.pieces(Piece::Rook);
		let queens = board.pieces(Piece::Queen);

		phase -= self.get_piece_amount(pawns) * pawn_phase;
		phase -= self.get_piece_amount(knights) * knight_phase;
		phase -= self.get_piece_amount(bishops) * bishop_phase;
		phase -= self.get_piece_amount(rooks) * rook_phase;
		phase -= self.get_piece_amount(queens) * queen_phase;

		phase = (phase * 256 + (total_phase / 2)) / total_phase;
		
		if phase > 145 {
			self.evaluator.end_game = true;
		}

		let mut pv = String::new();

		for i in 0..self.pv[0].len() {
			if self.pv[0][i] != None {
				pv += &(self.parse_to_uci(self.pv[0][i]) + " ");
			}
		}

		(self.parse_to_uci(best_move), pv.trim().to_string(), eval, self.nodes, nps)
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

	fn qsearch(&mut self, board: &Board, mut alpha: i32, beta: i32, mut ply: i32) -> (Option<Move>, i32) {
		self.nodes += 1;
		ply += 1;

		match board.status() {
			GameStatus::Won => return (None, -30000 + ply),
			GameStatus::Drawn => return (None, 0),
			GameStatus::Ongoing => {}
		}

		//check for three move repetition
		if self.my_past_positions.len() > 6 {
			let curr_hash = board.hash();
			if curr_hash == self.my_past_positions[self.my_past_positions.len() - 4] {
				return (None, 0);
			}
		}

		let stand_pat = self.evaluator.evaluate(board);

		//checking to see if the move the opponent makes is in our favor, if so, just return it no checking necessary
		if stand_pat >= beta {
			return (None, beta);
		}

		//new best move eval
		if alpha < stand_pat {
			alpha = stand_pat;
		}

		let move_list = self.evaluator.qmove_gen(board);

		//no more loud moves to be checked anymore, it can be returned safely
		if move_list.len() == 0 {
			return (None, stand_pat);
		}

		let mut best_move = None;
		let mut eval = i32::MIN;
		for sm in move_list {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);
			let (_, mut child_eval) = self.qsearch(&board_cache, -beta, -alpha, ply);
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

		(best_move, alpha)
	}

	fn search(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32) -> (Option<Move>, i32) {
		self.nodes += 1;

		match board.status() {
			GameStatus::Won => return (None, -30000 + (self.searching_depth - depth)),
			GameStatus::Drawn => return (None, 0),
			GameStatus::Ongoing => {}
		}

		//check for three move repetition
		if self.my_past_positions.len() > 6 {
			let curr_hash = board.hash();
			if curr_hash == self.my_past_positions[self.my_past_positions.len() - 4] {
				return (None, 0);
			}
		}

		if depth == 0 {
			return self.qsearch(board, alpha, beta, self.searching_depth);
		}

		let mut best_move = None;
		let mut eval = i32::MIN;
		for sm in self.evaluator.move_gen(board) {
			let mv = sm.mv;
			let mut board_cache = board.clone();
			board_cache.play_unchecked(mv);
			let (_, mut child_eval) = self.search(&board_cache, depth - 1, -beta, -alpha);
			child_eval *= -1;
			if child_eval > eval {
				eval = child_eval;
				best_move = Some(mv);
				if eval > alpha {
					self.update_pv(best_move, (self.searching_depth - depth) as usize);
					alpha = eval;
					if alpha >= beta {
						break;
					}
				}
			}
		}
		(best_move, eval)
	}
}