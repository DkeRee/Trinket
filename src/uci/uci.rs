use cozy_chess::*;

use std::sync::{Mutex, Arc, mpsc::channel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::search::search_master::*;

enum UCICmd {
	Uci,
	UciNewGame,
	IsReady,
	Go(i32, i64, i64, Arc<AtomicBool>),
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
	stop_abort: Option<Arc<AtomicBool>>,
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
			stop_abort: None,
			channel: get_channel()
		}
	}

	pub fn post(&mut self, cmd: &str) {
		let (sender, receiver) = &self.channel;

		let split = cmd.trim().split(" ");
		let cmd_vec: Vec<&str> = split.collect();

		match cmd_vec[0] {
			"uci" => {
				let thread_receiver = receiver.clone();

				self.engine_thread = Some(thread::spawn(move || {
					let mut engine = Engine::new();
					let mut playing = true;

					loop {
						if playing {
							match thread_receiver.lock().unwrap().recv().unwrap() {
								UCICmd::Uci => {
									println!("id name Trinket");
									println!("id author DkeRee");
									println!("uciok");
								},
								UCICmd::UciNewGame => {
									engine = Engine::new();
								},
								UCICmd::IsReady => {
									println!("readyok");
								},
								UCICmd::Go(depth, wtime, btime, stop_abort) => {
									let best_move = engine.go(depth, wtime, btime, stop_abort);
									println!("bestmove {}", best_move);
								},
								UCICmd::PositionFen(fen) => {
									engine.board = Board::from_fen(&*fen, false).unwrap();
									engine.my_past_positions.push(engine.board.hash());
								},
								UCICmd::PositionPgn(pgn_vec, default) => {
									if default {
										engine.board = Board::default();
									}
									
									engine.my_past_positions = Vec::with_capacity(64);

									for i in 0..pgn_vec.len() {
										let mv = &*pgn_vec[i];
									
										let from = mv.chars().nth(0).unwrap().to_string() + &mv.chars().nth(1).unwrap().to_string();
										let to = mv.chars().nth(2).unwrap().to_string() + &mv.chars().nth(3).unwrap().to_string();

										let square: Square = from.parse().unwrap();

										if from == "e1" && (to == "c1" || to == "g1") && engine.board.piece_on(square).unwrap() == Piece::King {
											if to == "c1" {
												engine.board.play("e1a1".parse().unwrap());
											} else {
												engine.board.play("e1h1".parse().unwrap());
											}
										} else if from == "e8" && (to == "c8" || to == "g8") && engine.board.piece_on(square).unwrap() == Piece::King {
											if to == "c8" {
												engine.board.play("e8a8".parse().unwrap());
											} else {
												engine.board.play("e8h8".parse().unwrap());
											}
										} else {
											engine.board.play(mv.parse().unwrap());
										}

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
				sender.send(UCICmd::Uci).unwrap();
			},
			"ucinewgame" => {
				sender.send(UCICmd::UciNewGame).unwrap();
			},
			"isready" => {
				sender.send(UCICmd::IsReady).unwrap();
			},
			"go" => {
				self.stop_abort = Some(Arc::new(AtomicBool::new(false)));

				let mut depth = i32::MAX;
				let mut wtime: i64 = 300000;
				let mut btime: i64 = 300000;

				if cmd_vec.len() > 1 {
					if cmd_vec[1] == "wtime" {
						//specified time
						wtime = cmd_vec[2].parse::<i64>().unwrap();
						btime = cmd_vec[4].parse::<i64>().unwrap();
					} else if cmd_vec[1] == "depth" {
						//specified depth without time
						depth = cmd_vec[2].parse::<i32>().unwrap();
					}

					if cmd_vec.len() > 7 && cmd_vec[7] == "depth" {
						//specified depth with time
						depth = cmd_vec[8].parse::<i32>().unwrap();
					}
				}

				sender.send(UCICmd::Go(depth, wtime, btime, self.stop_abort.clone().unwrap())).unwrap();
			},
			"position" => {
				if cmd_vec[1] == "startpos" {
					if cmd_vec.len() == 2 {
						sender.send(UCICmd::UciNewGame).unwrap();
					} else {
						let mut pgn_vec = Vec::with_capacity(64);
						for i in 3..cmd_vec.len() {
							pgn_vec.push(String::from(cmd_vec[i]));
						}
						sender.send(UCICmd::PositionPgn(pgn_vec, true)).unwrap();
					}
				} else if cmd_vec[1] == "fen" {
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
				}
			},
			"stop" => {
				self.stop_abort.as_ref().unwrap().store(true, Ordering::Relaxed);
			},
			"quit" => {
				sender.send(UCICmd::Quit).unwrap();
				self.playing = false;
			},
			_ => println!("Unknown Command: {}", cmd)
		}
	}
}