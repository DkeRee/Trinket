#![allow(unused_mut)]
#![allow(unused_must_use)]

mod uci;
mod search;
mod eval;

use crate::uci::uci::*;
use std::io;

fn main() {
	let mut uci = UCICmd::new();
	let mut playing = true;

	while playing {
		let mut results = String::new();
		io::stdin().read_line(&mut results);

		let cmd_result = uci.post(&*results);
		println!("{}", cmd_result);
	}
}