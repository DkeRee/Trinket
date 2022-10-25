use cozy_chess::*;

use std::thread;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::search::searcher::*;
use crate::search::tt::*;
use crate::movegen::movegen::*;
use crate::uci::castle_parse::*;

#[derive(Debug)]
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
}

pub struct Engine<'a> {
	pub board: Board,
	pub my_past_positions: Vec<u64>,
	pub total_nodes: u64,
	thread_count: u32,
	handler: Option<Arc<AtomicBool>>,
	threads: Vec<EngineThread<'a>>,
	tt: TT
}

impl Engine<'_> {
	pub fn new(thread_count: u32) -> Engine<'static> {
		Engine {
			board: Board::default(),
			my_past_positions: Vec::with_capacity(64),
			total_nodes: 0,
			thread_count: thread_count,
			handler: None,
			threads: (0..thread_count).map(|_| EngineThread::new(None)).collect(),
			tt: TT::new()
		}
	}

	pub fn go(&mut self, time_control: TimeControl, handler: Arc<AtomicBool>) -> String {
		let shared_info = SharedInfo::new(&self.tt);

		thread::scope(|scope| {			
			self.handler = Some(handler.clone());
			self.total_nodes = 0;

			let mut worker_threads = Vec::new();

			for i in 0..self.thread_count {
				let thread_movegen = self.threads[i as usize].movegen.clone();
				let board = self.board.clone();
				let positions = self.my_past_positions.clone();
				let this_handler = &self.handler;
				let this_shared_info = &shared_info;

				worker_threads.push(scope.spawn(move || {
					Searcher::create(time_control.clone(), this_shared_info, thread_movegen, board, positions, this_handler.clone())
				}));
			}

			let mut index = 0;
			for worker in worker_threads {
				let (movegen, nodes) = worker.join().unwrap();
				self.threads[index].movegen = movegen.clone();
				self.total_nodes += nodes;
				index += 1;
			}

			let best_move = *(&shared_info).best_move.lock().unwrap();
			_960_to_regular_(best_move, &self.board)
		})
	}

	pub fn reset_threads(&mut self, thread_count: u32) {
		self.threads = (0..thread_count).map(|_| EngineThread::new(None)).collect();
	}
}