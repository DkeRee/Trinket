use cozy_chess::*;

use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::search::searcher::*;
use crate::search::tt::*;
use crate::movegen::movegen::*;
use crate::uci::castle_parse::*;

pub struct SharedInfo<'a> {
	pub tt: &'a TT,
	pub best_move: Arc<Mutex<Option<Move>>>,
	pub best_depth: Arc<Mutex<i32>>,
	pub nodes: Arc<Mutex<u64>>,
}

impl SharedInfo<'_> {
	pub fn new(tt: &TT) -> SharedInfo {
		SharedInfo {
			tt: tt,
			best_move: Arc::new(Mutex::new(None)),
			best_depth: Arc::new(Mutex::new(0)),
			nodes: Arc::new(Mutex::new(0))
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct TimeControl {
	pub depth: i32,
	pub wtime: i64,
	pub btime: i64,
	pub winc: i64,
	pub binc: i64,
	pub movestogo: i64
}

impl TimeControl {
	pub fn new() -> TimeControl {
		TimeControl {
			depth: i32::MAX,
			wtime: i64::MAX,
			btime: i64::MAX,
			winc: 0,
			binc: 0,
			movestogo: i64::MAX
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
/*
	pub fn go(&mut self, time_control: &TimeControl, board: Board, my_past_positions: Vec<u64>) {
		let thread_movegen = self.movegen.clone();
		let this_time_control = time_control;
		let this_shared_info = &self.shared_info.unwrap();

		self.thread = Some(thread::spawn(move || {
			Searcher::create(this_time_control, this_shared_info, thread_movegen, board, my_past_positions)
		}));
	}
*/
}

pub struct Engine<'a> {
	pub board: Board,
	pub my_past_positions: Vec<u64>,
	pub total_nodes: u64,
	handler: Option<Arc<AtomicBool>>,
	threads: Vec<EngineThread<'a>>,
	tt: TT
}

impl Engine<'_> {
	pub fn new(thread_count: usize) -> Engine<'static> {
		Engine {
			board: Board::default(),
			my_past_positions: Vec::with_capacity(64),
			total_nodes: 0,
			handler: None,
			threads: (0..thread_count).map(|_| EngineThread::new(None)).collect(),
			tt: TT::new()
		}
	}

	pub fn go(&mut self, time_control: TimeControl, handler: Arc<AtomicBool>) -> String {
		let shared_info = SharedInfo::new(&self.tt);

		self.handler = Some(handler.clone());
		self.total_nodes = 0;

		let mut worker_threads = Vec::with_capacity(self.threads.len());

		//start search on all threads
		for i in 0..worker_threads.len() {
			//self.threads[i].shared_info = Some(&shared_info);

			//SEARCH
			let thread_movegen = self.threads[i].movegen.clone();
			let board = self.board.clone();
			let positions = self.my_past_positions.clone();
			let this_handler = &self.handler;

			worker_threads.push(thread::spawn(move || {
				Searcher::create(time_control.clone(), &shared_info, thread_movegen, board, positions, this_handler.clone())
			}));


			//.go(&time_control, self.board.clone(), self.my_past_positions.clone());
		}

		//wait for all workers to finish their tasks
		let mut i = 0;
		for worker in worker_threads {
			//fish out updated movegen tables for individual local use
			self.threads[i].movegen = worker.join().unwrap();
			i += 1;
		}

		self.total_nodes = *shared_info.nodes.lock().unwrap();

		_960_to_regular_(*shared_info.best_move.lock().unwrap(), &self.board)
	}

	pub fn reset_threads(&mut self, thread_count: usize) {
		self.threads = (0..thread_count).map(|_| EngineThread::new(None)).collect();
	}
}