use cozy_chess::*;

use std::sync::atomic::{Ordering, AtomicU64};
use bytemuck::{Pod, Zeroable};

use crate::eval::score::*;

/*
Special thanks to MinusKelvin from OpenBench!
https://www.chessprogramming.org/Transposition_Table
*/

const MB: usize = 16;
const TT_LENGTH: u64 = (MB * 1024 * 1024 / std::mem::size_of::<TTSlot>()) as u64;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeKind {
	Exact,
	UpperBound,
	LowerBound,
	Null
}

#[derive(Debug)]
pub struct TTEntry {
	pub best_move: Option<Move>,
	pub eval: i32,
	pub depth: i32,
	pub node_kind: NodeKind
}

#[derive(Debug)]
pub struct TTSlot {
	position_hash: AtomicU64,
	data: AtomicU64
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct EncodedEntry {
	eval: i32,
	mv_byte: u16,
	depth: u8,
	node_kind: u8
}

//adjust tricky mate scores to make valid eval
fn add_mate_score(eval: i32, ply: i32) -> i32 {
	if eval < -Score::CHECKMATE_DEFINITE {
		eval + ply
	} else if eval > Score::CHECKMATE_DEFINITE {
		eval - ply
	} else {
		eval
	}
}

fn remove_mate_score(eval: i32, ply: i32) -> i32 {
	if eval < -Score::CHECKMATE_DEFINITE {
		eval - ply
	} else if eval > Score::CHECKMATE_DEFINITE {
		eval + ply
	} else {
		eval
	}
}

impl TTSlot {
	fn empty() -> TTSlot {
		TTSlot {
			position_hash: AtomicU64::new(0),
			data: AtomicU64::new(0)
		}
	}

	fn store(&self, best_move: Option<Move>, eval: i32, position: u64, depth: i32, node_kind: NodeKind) {	
		let mut move_bits = 0u16;
		move_bits = (move_bits << 6) | best_move.unwrap().from as u16;
		move_bits = (move_bits << 6) | best_move.unwrap().to as u16;
		move_bits = (move_bits << 4) | best_move.unwrap().promotion.map_or(0b1111, |p| p as u16);

		let data = bytemuck::cast(EncodedEntry {
			eval: eval,
			mv_byte: move_bits,
			depth: depth as u8,
			node_kind: node_kind as u8
		});

		self.position_hash.store(position ^ data, Ordering::Relaxed);
		self.data.store(data, Ordering::Relaxed);
	}

	fn load(&self, board: &Board, ply: i32) -> Option<TTEntry> {
		let position_hash_data = self.position_hash.load(Ordering::Relaxed);
		let data = self.data.load(Ordering::Relaxed);

		//Invalid Probe
		if data == 0 || position_hash_data ^ data != board.hash() {
			return None;
		} else {
			let data: EncodedEntry = bytemuck::cast(data);

			let mut move_bits = data.mv_byte;
			let promotion = move_bits & 0b1111;
			move_bits >>= 4;
			let to = move_bits & 0b111111;
			move_bits >>= 6;
			let from = move_bits & 0b111111;
			move_bits >>= 6;

			Some(TTEntry {
				best_move: Some(Move {
					from: Square::index(from as usize),
					to: Square::index(to as usize),
					promotion: Piece::try_index(promotion as usize)
				}),
				eval: add_mate_score(data.eval, ply),
				depth: data.depth as i32,
				node_kind: match data.node_kind {
					0 => NodeKind::Exact,
					1 => NodeKind::UpperBound,
					2 => NodeKind::LowerBound,
					_ => NodeKind::Null
				}
			})
		}
	}
}


#[derive(Debug)]
pub struct TT {
	pub table: Box<[TTSlot]>,
	length: u64
}

impl TT {
	pub fn new() -> TT {
		TT {
			table: (0..TT_LENGTH).map(|_| TTSlot::empty()).collect(),
			length: TT_LENGTH
		}
	}

	pub fn insert(&self, best_move: Option<Move>, eval: i32, position: u64, ply: i32, depth: i32, node_kind: NodeKind) {
		self.table[(position % self.length) as usize].store(best_move, remove_mate_score(eval, ply), position, depth, node_kind);
	}

	pub fn find(&self, board: &Board, ply: i32) -> Option<TTEntry> {
		self.table[(board.hash() % self.length) as usize].load(board, ply)
	}
}