#![allow(unused_mut)]
#![allow(unused_must_use)]

mod uci;
mod search;
mod eval;
mod movegen;

use crate::uci::uci::*;
use std::io;

fn main() {
	let mut uci = UCIMaster::new();

	loop {
		if uci.playing {
				let mut results = String::new();
				io::stdin().read_line(&mut results);
			
				uci.post(&*results);
		} else {
			break;
		}
	}
}