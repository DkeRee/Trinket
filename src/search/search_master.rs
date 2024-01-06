use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::search::tt::*;
use crate::eval::score::*;
use crate::search::searcher::*;
use crate::movegen::movegen::*;
use crate::uci::castle_parse::*;

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
	seldepth: i32,
	movegen: MoveGen,
	tt: TT
}

impl Engine {
	pub fn new(hash: u32) -> Engine {
		Engine {
			board: Board::default(),
			max_depth: 0,
			my_past_positions: Vec::with_capacity(64),
			nodes: 0,
			seldepth: 0,
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
		let mut soft_timeout = None;
		if time != u64::MAX {
			thread::spawn(move || {
				let hard_timeout = if movetime.is_none() {
					let mut hard_timeout_div = 2;
					if let Some(movestogo) = movestogo {
						hard_timeout_div /= movestogo / 10;
					}

					(time + timeinc) / (hard_timeout_div) as u64
				} else {
					movetime.unwrap() as u64
				};

				thread::sleep(Duration::from_millis(hard_timeout));
				abort.store(true, Ordering::Relaxed);
			});

			let mut soft_timeout_div = 25;
			if let Some(movestogo) = movestogo {
				soft_timeout_div /= movestogo / 10;
			}

			soft_timeout = Some((time + timeinc) / (soft_timeout_div) as u64);
		}

		//ASPIRATION WINDOWS ALPHA BETA
		let mut alpha = -i32::MAX;
		let mut beta = i32::MAX;

		let mut depth_index = 0;

		while depth_index < self.max_depth && depth_index < 250 {
			self.seldepth = 0;
			let board = &mut self.board.clone();
			let mut past_positions = self.my_past_positions.clone();

			let result = Searcher::new(&self.tt, &mut self.movegen, time_control.handler.clone(), SearchInfo {
				board: board.clone(),
				depth: depth_index + 1,
				alpha,
				beta,
				past_positions
			});

			if result != None {
				let (best_mv, eval, nodes, seldepth) = result.unwrap();

				self.nodes += nodes;
				self.seldepth += seldepth;

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

				println!("info depth {} seldepth {} time {} score {} nodes {} nps {} pv {}", depth_index, self.seldepth, elapsed as u64, score_str, self.nodes, nps, self.get_pv(board, depth_index, 0));

				if movetime.is_none() && !soft_timeout.is_none() {
					if elapsed as u64 > soft_timeout.unwrap() {
						break;
					}
				}
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
}

impl Engine {
	const ASPIRATION_WINDOW: i32 = 15;
}