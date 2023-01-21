use cozy_chess::*;

use std::sync::{Mutex, Arc, mpsc::channel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::search::search_master::*;
use crate::search::search_options::*;
use crate::uci::bench::*;
use crate::uci::castle_parse::*;

const HASH_MIN: u32 = 0;
const HASH_MAX: u32 = 64000;

enum UCICmd {
	Uci,
	UciNewGame(u32),
	IsReady,
	SetOption(Options),
	Go(TimeControl),
	PositionFen(String),
	PositionPgn(Vec<String>, bool),
	Quit
}

fn get_channel() -> (Sender<UCICmd>, Arc<Mutex<Receiver<UCICmd>>>) {
	let (sender, receiver): (Sender<UCICmd>, Receiver<UCICmd>) = channel();
	return (sender, Arc::new(Mutex::new(receiver)));
}

//uci command parser
pub struct UCIMaster {
	pub playing: bool,
	hash: u32,
	engine_thread: Option<thread::JoinHandle<()>>,
	stop_abort: Arc<AtomicBool>,
	channel: (Sender<UCICmd>, Arc<Mutex<Receiver<UCICmd>>>)
}

impl UCIMaster {
	pub fn new() -> UCIMaster {
		println!("Trinket V{}", env!("CARGO_PKG_VERSION"));
		println!("{}", env!("CARGO_PKG_REPOSITORY"));
		println!("If manually operating, initialize engine before use by running the command 'uci' ONCE");
		println!("Engine runs on UCI protocol");
		println!("http://wbec-ridderkerk.nl/html/UCIProtocol.html");

		let mut continue_engine = true;

		//run bench if requested for OpenBench
		if std::env::args().nth(1).as_deref() == Some("bench") {
			bench();
			continue_engine = false;
		}

		UCIMaster {
			playing: continue_engine,
			hash: 16,
			engine_thread: None,
			stop_abort: Arc::new(AtomicBool::new(false)),
			channel: get_channel()
		}
	}

	pub fn post(&mut self, cmd: &str) {
		let (sender, receiver) = &self.channel;

		let split = cmd.trim().split(" ");
		let cmd_vec: Vec<&str> = split.collect();

		match cmd_vec[0] {
			"uci" => {
				//init engine
				if self.engine_thread.is_none() {
					let thread_receiver = receiver.clone();

					let init_hash_count = self.hash;
					self.engine_thread = Some(thread::spawn(move || {
						let mut engine = Engine::new(init_hash_count);
						let mut playing = true;

						loop {
							if playing {
								match thread_receiver.lock().unwrap().recv().unwrap() {
									UCICmd::Uci => {
										println!("id name Trinket {}", env!("CARGO_PKG_VERSION"));
										println!("id author DkeRee");
										println!("option name Hash type spin default 16 min 0 max 64000");
										println!("option name Threads type spin default 1 min 1 max 1");
										println!("option name AspirationWindow type spin default {} min 1 max 100", SearchOptions::ASPIRATION_WINDOW);
										println!("option name MaxDepthRfp type spin default {} min 1 max 15", SearchOptions::MAX_DEPTH_RFP);
										println!("option name MultiplierRfp type spin default {} min 10 max 200", SearchOptions::MULTIPLIER_RFP);
										println!("option name NmpReductionBase type spin default {} min 1 max 50", SearchOptions::NMP_REDUCTION_BASE);
										println!("option name NmpXShift type spin default {} min 1 max 30", SearchOptions::NMP_XSHIFT);
										println!("option name NmpYStretch type spin default {} min 1 max 40", SearchOptions::NMP_YSTRETCH);
										println!("option name LmrDepthLimit type spin default {} min 1 max 5", SearchOptions::LMR_DEPTH_LIMIT);
										println!("option name LmrFullSearchedMoveLimit type spin default {} min 1 max 5", SearchOptions::LMR_FULL_SEARCHED_MOVE_LIMIT);
										println!("option name IidDepthMin type spin default {} min 1 max 15", SearchOptions::IID_DEPTH_MIN);
										println!("option name LmpDepthMax type spin default {} min 1 max 10", SearchOptions::LMP_DEPTH_MAX);
										println!("option name LmpMultiplier type spin default {} min 5 max 30", SearchOptions::LMP_MULTIPLIER);
										println!("option name HistoryDepthMin type spin default {} min 1 max 10", SearchOptions::HISTORY_DEPTH_MIN);
										println!("option name HistoryPruneMoveLimit type spin default {} min 1 max 10", SearchOptions::HISTORY_PRUNE_MOVE_LIMIT);
										println!("option name HistoryThreshold type spin default {} min 1 max 200", SearchOptions::HISTORY_THRESHOLD);
										println!("option name HistoryReduction type spin default {} min 1 max 2", SearchOptions::HISTORY_REDUCTION);
										println!("uciok");
									},
									UCICmd::UciNewGame(hash_count) => {
										engine = Engine::new(hash_count);
									},
									UCICmd::IsReady => {
										println!("readyok");
									},
									UCICmd::SetOption(option) => {
										engine.options.change(option);
									}
									UCICmd::Go(time_control) => {
										let best_move = engine.go(time_control);
										println!("bestmove {}", best_move);
									},
									UCICmd::PositionFen(fen) => {
										engine.board = Board::from_fen(&*fen.trim(), false).unwrap();

										engine.my_past_positions = Vec::with_capacity(64);
										engine.my_past_positions.push(engine.board.hash());
									},
									UCICmd::PositionPgn(pgn_vec, default) => {
										if default {
											engine.board = Board::default();
											engine.my_past_positions = Vec::with_capacity(64);
										}

										for i in 0..pgn_vec.len() {
											engine.board.play_unchecked(_regular_to_960_(pgn_vec[i].clone(), &engine.board).parse().unwrap());
											engine.my_past_positions.push(engine.board.hash());
										}
									},
									UCICmd::Quit => {
										playing = false;
									}
								}
							} else {
								break;
							}
						}
					}));
				}

				sender.send(UCICmd::Uci).unwrap();
			},
			"setoption" => {
				match cmd_vec[1] {
					"name" => {
						match cmd_vec[2] {
							"Hash" => {
								let hash = cmd_vec[4].parse::<u32>().unwrap();

								if HASH_MIN <= hash && hash <= HASH_MAX {
									self.hash = hash;
									sender.send(UCICmd::UciNewGame(self.hash)).unwrap();
								} else {
									println!("Thread input is out of bounds.");
								}
							},
							_ => {
								let value = cmd_vec[4].parse::<i32>().unwrap();
								sender.send(UCICmd::SetOption(SearchOptions::get(cmd_vec[2], value))).unwrap();
							}
						}
					},
					_ => {}
				}
			},
			"ucinewgame" => {
				sender.send(UCICmd::UciNewGame(self.hash)).unwrap();
			},
			"isready" => {
				sender.send(UCICmd::IsReady).unwrap();
			},
			"go" => {
				self.stop_abort = Arc::new(AtomicBool::new(false));

				let mut time_control = TimeControl::new(self.stop_abort.clone());

				for i in 1..cmd_vec.len() {
					match cmd_vec[i] {
						"depth" => {
							time_control.depth = cmd_vec[i + 1].parse::<i32>().unwrap();
						},
						"movetime" => {
							time_control.movetime = Some(cmd_vec[i + 1].parse::<i64>().unwrap());
						},
						"wtime" => {
							time_control.wtime = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"btime" => {
							time_control.btime = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"winc" => {
							time_control.winc = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"binc" => {
							time_control.binc = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"movestogo" => {
							time_control.movestogo = Some(cmd_vec[i + 1].parse::<i64>().unwrap());
						},
						_ => {}
					}
				}

				sender.send(UCICmd::Go(time_control)).unwrap();
			},
			"position" => {
				if cmd_vec.len() > 1 {
					match cmd_vec[1] {
						"startpos" => {
							if cmd_vec.len() == 2 {
								sender.send(UCICmd::UciNewGame(self.hash)).unwrap();
							} else {
								let mut pgn_vec = Vec::with_capacity(64);
								for i in 3..cmd_vec.len() {
									pgn_vec.push(String::from(cmd_vec[i]));
								}
								sender.send(UCICmd::PositionPgn(pgn_vec, true)).unwrap();
							}
						},
						"fen" => {
							let mut fen = String::new();
							let mut pgn_index: Option<usize> = None;

							for i in 2..cmd_vec.len() {
								if cmd_vec[i] == "moves" {
									pgn_index = Some(i + 1);
									break;
								}
								let segment = String::from(cmd_vec[i]) + " ";
								fen += &segment;
							}
							sender.send(UCICmd::PositionFen(fen.clone())).unwrap();
						
							if pgn_index != None {
								let mut pgn_vec = Vec::with_capacity(64);
								for i in pgn_index.unwrap()..cmd_vec.len() {
									pgn_vec.push(String::from(cmd_vec[i]));
								}
								sender.send(UCICmd::PositionPgn(pgn_vec, false)).unwrap();
							}
						},
						_ => {}
					}
				} else {
					println!("You must provide commands startpos or fen for board initialization");
				}
			},
			"stop" => {
				self.stop_abort.as_ref().store(true, Ordering::Relaxed);
			},
			"quit" => {
				sender.send(UCICmd::Quit).unwrap();
				self.playing = false;
			},
			_ => println!("Unknown Command: {}", cmd)
		}
	}
}