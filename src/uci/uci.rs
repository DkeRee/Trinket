use cozy_chess::*;

use std::sync::{Mutex, Arc, mpsc::channel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::search::search_master::*;
use crate::search::search::*;
use crate::uci::bench::*;
use crate::uci::castle_parse::*;

const THREAD_MAX: i32 = 1024;
const THREAD_MIN: i32 = 1;

enum UCICmd {
	Uci,
	UciNewGame,
	IsReady,
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

		//init lmr table
		init_lmr_table();

		UCIMaster {
			playing: continue_engine,
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

					self.engine_thread = Some(thread::spawn(move || {
						let mut engine = Engine::new();
						let mut playing = true;

						loop {
							if playing {
								match thread_receiver.lock().unwrap().recv().unwrap() {
									UCICmd::Uci => {
										println!("id name Trinket {}", env!("CARGO_PKG_VERSION"));
										println!("id author DkeRee");
										println!("option name Hash type spin default 16 min 0 max 64000");
										println!("option name Threads type spin default 1 min {} max {}", THREAD_MIN, THREAD_MAX);
										println!("uciok");
									},
									UCICmd::UciNewGame => {
										engine = Engine::new();
									},
									UCICmd::IsReady => {
										println!("readyok");
									},
									UCICmd::Go(time_control) => {
										let best_move = engine.go(time_control);
										println!("bestmove {}", best_move);
									},
									UCICmd::PositionFen(fen) => {
										engine.board = Board::from_fen(&*fen.trim(), false).unwrap();

										engine.local_tables.my_past_positions = Vec::with_capacity(64);
										engine.local_tables.my_past_positions.push(engine.board.hash());
									},
									UCICmd::PositionPgn(pgn_vec, default) => {
										if default {
											engine.board = Board::default();
											engine.local_tables.my_past_positions = Vec::with_capacity(64);
										}

										for i in 0..pgn_vec.len() {
											engine.board.play_unchecked(_regular_to_960_(pgn_vec[i].clone(), &engine.board).parse().unwrap());
											engine.local_tables.my_past_positions.push(engine.board.hash());
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
				unsafe {
					for i in 1..cmd_vec.len() {
						match cmd_vec[i] {
							"name" => {
								for o in (i + 1)..cmd_vec.len() {
									match cmd_vec[o] {
										"Threads" => {
											let new_thread = cmd_vec[o + 1].parse::<i32>().unwrap();

											if THREAD_MIN <= new_thread && new_thread <= THREAD_MAX {
												THREADS = new_thread;
											} else {
												println!("The thread count you provided is out of bounds");
											}
											break;
										},
										_ => println!("Unknown Field")
									}
								}
								break;
							},
							_ => println!("Invalid Arguments")
						}
					}
				}
			}
			"ucinewgame" => {
				sender.send(UCICmd::UciNewGame).unwrap();
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
							time_control.wtime = cmd_vec[i + 1].parse::<i64>().unwrap();
							time_control.btime = cmd_vec[i + 1].parse::<i64>().unwrap();
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
							time_control.movestogo = cmd_vec[i + 1].parse::<i64>().unwrap();
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
								sender.send(UCICmd::UciNewGame).unwrap();
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