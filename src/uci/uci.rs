use cozy_chess::*;

use std::sync::{Mutex, Arc, mpsc::channel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::search::search_master::*;
use crate::uci::castle_parse::*;

enum UCICmd {
	Uci,
	UciNewGame,
	IsReady,
	Go(i32, i64, i64, i64, i64, i64, Arc<AtomicBool>),
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

		UCIMaster {
			playing: true,
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
										println!("id name Trinket 2.0.0");
										println!("id author DkeRee");
										println!("uciok");
									},
									UCICmd::UciNewGame => {
										engine = Engine::new();
									},
									UCICmd::IsReady => {
										println!("readyok");
									},
									UCICmd::Go(depth, wtime, btime, winc, binc, movestogo, stop_abort) => {
										let best_move = engine.go(depth, wtime, btime, winc, binc, movestogo, stop_abort);
										println!("bestmove {}", best_move);
									},
									UCICmd::PositionFen(fen) => {
										engine.board = Board::from_fen(&*fen, false).unwrap();

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
			"ucinewgame" => {
				sender.send(UCICmd::UciNewGame).unwrap();
			},
			"isready" => {
				sender.send(UCICmd::IsReady).unwrap();
			},
			"go" => {
				self.stop_abort = Arc::new(AtomicBool::new(false));

				let mut depth = i32::MAX;
				let mut wtime: i64 = i64::MAX;
				let mut btime: i64 = i64::MAX;
				let mut winc: i64 = 0;
				let mut binc: i64 = 0;
				let mut movestogo: i64 = i64::MAX;

				for i in 1..cmd_vec.len() {
					match cmd_vec[i] {
						"depth" => {
							depth = cmd_vec[i + 1].parse::<i32>().unwrap();
						},
						"movetime" => {
							wtime = cmd_vec[i + 1].parse::<i64>().unwrap();
							btime = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"wtime" => {
							wtime = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"btime" => {
							btime = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"winc" => {
							winc = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"binc" => {
							binc = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						"movestogo" => {
							movestogo = cmd_vec[i + 1].parse::<i64>().unwrap();
						},
						_ => {}
					}
				}

				sender.send(UCICmd::Go(depth, wtime, btime, winc, binc, movestogo, self.stop_abort.clone())).unwrap();
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