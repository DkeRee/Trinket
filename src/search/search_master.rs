use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

use crate::search::tt::*;
use crate::eval::score::*;
use crate::search::searcher::*;
use crate::movegen::boardwrapper::*;
use crate::movegen::movegen::*;
use crate::movegen::boardwrapper::*;
use crate::uci::castle_parse::*;

pub struct SharedInfo<'a> {
	pub tt: &'a TT,
	pub best_move: Arc<Mutex<Option<Move>>>,
	pub best_depth: Arc<Mutex<i32>>
}

impl SharedInfo<'_> {
	pub fn new(tt: &TT) -> SharedInfo {
		SharedInfo {
			tt: tt,
			best_move: Arc::new(Mutex::new(None)),
			best_depth: Arc::new(Mutex::new(0))
		}
	}
}

pub struct EngineThread<'a> {
	pub shared_info: Option<&'a SharedInfo<'a>>,
	movegen: MoveGen
}

impl EngineThread<'_> {
	pub fn new<'a>(shared_info: Option<&'a SharedInfo>) -> EngineThread<'a> {
		EngineThread {
			shared_info: shared_info,
			movegen: MoveGen::new()
		}
	}
}

#[derive(Clone, Copy)]
pub struct TimeControl {
	pub depth: i32,
	pub wtime: i64,
	pub btime: i64,
	pub winc: i64,
	pub binc: i64,
	pub movetime: Option<i64>,
	pub movestogo: Option<i64>,
}

impl TimeControl {
	pub fn new() -> TimeControl {
		TimeControl {
			depth: i32::MAX,
			wtime: i64::MAX,
			btime: i64::MAX,
			winc: 0,
			binc: 0,
			movetime: None,
			movestogo: None,
		}
	}
}

pub struct Engine<'a> {
	pub boardwrapper: BoardWrapper,
	pub my_past_positions: Vec<u64>,
	pub nodes: u64,
	thread_count: u32,
	threads: Vec<EngineThread<'a>>,
	handler: Option<Arc<AtomicBool>>,
	tt: TT
}

impl Engine<'_> {
	pub fn new(hash: u32, thread_count: u32) -> Engine<'static> {
		Engine {
			boardwrapper: BoardWrapper::new(),
			my_past_positions: Vec::with_capacity(64),
			nodes: 0,
			thread_count: thread_count,
			threads: (0..thread_count).map(|_| EngineThread::new(None)).collect(),
			handler: None,
			tt: TT::new(hash)
		}
	}

	pub fn go(&mut self, time_control: TimeControl, handler: Arc<AtomicBool>) -> String {
		let shared_info = SharedInfo::new(&self.tt);

		thread::scope(|scope| {
			self.handler = Some(handler.clone());

			let mut worker_threads = Vec::new();

			for i in 0..self.thread_count {
				let thread_movegen = self.threads[i as usize].movegen.clone();
				let boardwrapper = self.boardwrapper.clone();
				let positions = self.my_past_positions.clone();
				let this_handler = &self.handler;
				let this_shared_info = &shared_info;

				worker_threads.push(scope.spawn(move || {
					Searcher::create(time_control.clone(), this_shared_info, thread_movegen, boardwrapper, positions, this_handler.clone())
				}));
			}

			//manage time
			let mut time: u64;
			let mut timeinc: u64;

			let abort = handler.clone();

			let movetime = time_control.movetime;
			let movestogo = time_control.movestogo;

			//set time
			match self.boardwrapper.board.side_to_move() {
				Color::White => {
					time = time_control.wtime as u64;
					timeinc = time_control.winc as u64;
				},
				Color::Black => {
					time = time_control.btime as u64;
					timeinc = time_control.binc as u64;	
				}
			}

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
			}

			let mut index = 0;
			for worker in worker_threads {
				let (movegen, nodes) = worker.join().unwrap();
				self.threads[index].movegen = movegen.clone();
				self.nodes += nodes;
				index += 1;
			}

			let best_move = *(&shared_info).best_move.lock().unwrap();
			_960_to_regular_(best_move, &self.boardwrapper.board)
		})
	}
}