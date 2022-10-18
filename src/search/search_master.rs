use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

use crate::search::search::*;
use crate::search::tt::*;
use crate::movegen::movesorter::*;
use crate::eval::score::*;
use crate::uci::castle_parse::*;

pub struct WorkerHandler<'a> {
	handler: &'a Arc<AtomicBool>
}

impl WorkerHandler<'_> {
	pub fn is_abort(&self) -> bool {
		self.handler.load(Ordering::Relaxed)
	}
}

pub struct SharedTables {
	pub tt: TT
}

#[derive(Clone, PartialEq, Debug)]
pub struct LocalTables {
	pub sorter: MoveSorter,
	pub my_past_positions: Vec<u64>
}

pub struct TimeControl {
	pub depth: i32,
	pub wtime: i64,
	pub btime: i64,
	pub winc: i64,
	pub binc: i64,
	pub movestogo: i64,
	pub handler: Arc<AtomicBool>,
	pub threads: usize
}

impl SharedTables {
	pub fn new() -> SharedTables {
		SharedTables {
			tt: TT::new()
		}
	}
}

impl LocalTables {
	pub fn new() -> LocalTables {
		LocalTables {
			sorter: MoveSorter::new(),
			my_past_positions: Vec::with_capacity(64)
		}
	}
}

impl TimeControl {
	pub fn new(stop_abort: Arc<AtomicBool>, threads: usize) -> TimeControl {
		TimeControl {
			depth: i32::MAX,
			wtime: i64::MAX,
			btime: i64::MAX,
			winc: 0,
			binc: 0,
			movestogo: i64::MAX,
			handler: stop_abort,
			threads: threads
		}
	}
}

pub struct Engine {
	pub board: Board,
	pub max_depth: i32,
	pub nodes: u64,
	pub searching_depth: i32,
	pub shared_tables: SharedTables,
	pub local_tables: LocalTables
}

impl Engine {
	pub fn new() -> Engine {
		Engine {
			board: Board::default(),
			max_depth: 0,
			searching_depth: 0,
			nodes: 0,
			shared_tables: SharedTables::new(),
			local_tables: LocalTables::new()
		}
	}

	pub fn go(&mut self, time_control: TimeControl) -> String {
		let now = Instant::now();

		let mut best_move = None;
		let mut best_eval = None;
		let mut time: f32;
		let mut timeinc: f32;

		self.max_depth = time_control.depth;

		self.nodes = 0;

		//set time
		match self.board.side_to_move() {
			Color::White => {
				time = time_control.wtime as f32;
				timeinc = time_control.winc as f32;
			},
			Color::Black => {
				time = time_control.btime as f32;
				timeinc = time_control.binc as f32;	
			}
		}

		let mut depth_index = 1;
		let mut search_data = (0..time_control.threads).map(|_| self.local_tables.clone()).collect::<Vec<_>>();

		while depth_index <= self.max_depth {
			let search_elapsed = now.elapsed().as_secs_f32() * 1000_f32;
			if search_elapsed < ((time + timeinc) / f32::min(40_f32, time_control.movestogo as f32)) {
				self.searching_depth = depth_index;

				let board = &mut self.board.clone();

				let abort = time_control.handler.clone();
				thread::spawn(move || {
					thread::sleep(Duration::from_millis(time as u64 / 32));
					abort.store(true, Ordering::Relaxed);
				});

				//LAZY SMP (MULTI-THREADING)
				let result: Option<(Option<Move>, Eval, u64)> = std::thread::scope(|scope| {
					//store workers
					let (_, worker_data) = search_data.split_first_mut().unwrap();

					let mut workers = Vec::with_capacity(worker_data.len());

					//node code
					let mut total_nodes = 0;

					for local_tables in worker_data {
						let mut handler = WorkerHandler {
							handler: &time_control.handler
						};
						let shared_tables = &self.shared_tables;
						let this_depth = self.searching_depth;
						let pos = &board;
						let this_best_eval = best_eval.clone();

						workers.push(scope.spawn(move || {
							Searcher::new(pos, shared_tables, local_tables, &mut handler, this_depth, this_best_eval)
						}));
					}

					let (best_mv, eval, nodes) = Searcher::new(board, &self.shared_tables, &mut self.local_tables, &mut WorkerHandler { handler: &time_control.handler }, self.searching_depth, best_eval.clone())?;

					total_nodes += nodes;

					for worker in workers {
						let worker_thread = worker.join().unwrap();

						if !worker_thread.is_none() {
							let (_, _, nodes) = worker_thread.unwrap();
							total_nodes += nodes;				
						}
					}


					Some((best_mv, eval, total_nodes))
				});

				if result != None {
					let (best_mv, eval, nodes) = result.unwrap();

					best_move = best_mv.clone();
					best_eval = Some(eval.clone());
					depth_index += 1;

					//UPDATE NODE COUNT TO MASTER
					self.nodes += nodes;

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
			} else {
				break;
			}
		}

		_960_to_regular_(best_move, &self.board)
	}

	//fish PV from TT
	fn get_pv(&self, board: &mut Board, depth: i32, ply: i32) -> String {
		if depth == 0 {
			return String::new();
		}

		//probe TT
		match self.shared_tables.tt.find(board, ply) {
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