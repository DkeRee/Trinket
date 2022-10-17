use cozy_chess::*;

use std::sync::atomic::{Ordering, AtomicU64};
use bytemuck::{Pod, Zeroable};

use crate::eval::score::*;

/*
Special thanks to MinusKelvin from OpenBench!
https://www.chessprogramming.org/Transposition_Table
*/

const MB: usize = 16;

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
	mv_byte_one: u8,
	mv_byte_two: u8,
	eval: [u8; 4],
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
		let best_move_from = best_move.unwrap().from as u8;
		let best_move_to = best_move.unwrap().to as u8;
		let best_move_promotion = best_move.unwrap().promotion.map_or(6, |p| p as u8);

		let mv_byte_one = (best_move_from << 2) | (best_move_to >> 4);
		let mv_byte_two = (best_move_to << 4) | (best_move_promotion << 1);

		let data = bytemuck::cast(EncodedEntry {
			mv_byte_one: mv_byte_one,
			mv_byte_two: mv_byte_two,
			eval: eval_to_bytes(eval),
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
			Some(TTEntry {
				best_move: Some(Move {
					from: Square::index(((data.mv_byte_one & 252) >> 2) as usize),
					to: Square::index((((data.mv_byte_one & 3) << 4) | ((data.mv_byte_two & 240) >> 4)) as usize),
					promotion: Piece::try_index(((data.mv_byte_two & 14) >> 1) as usize)
				}),
				eval: add_mate_score(bytes_to_eval(data.eval) as i32, ply),
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

pub struct TT {
	pub table: Box<[TTSlot]>,
	length: u64
}

impl TT {
	pub fn new() -> TT {
		TT {
			table: (0..(MB * 1024 * 1024 / std::mem::size_of::<TTSlot>()) as u64).map(|_| TTSlot::empty()).collect(),
			length: (MB * 1024 * 1024 / std::mem::size_of::<TTSlot>()) as u64
		}
	}

	pub fn insert(&self, best_move: Option<Move>, eval: i32, position: u64, ply: i32, depth: i32, node_kind: NodeKind) {
		self.table[(position % self.length) as usize].store(best_move, remove_mate_score(eval, ply), position, depth, node_kind);
	}

	pub fn find(&self, board: &Board, ply: i32) -> Option<TTEntry> {
		self.table[(board.hash() % self.length) as usize].load(board, ply)
	}
}