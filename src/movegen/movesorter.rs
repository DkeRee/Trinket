use cozy_chess::*;
use crate::movegen::movegen::*;
use crate::movegen::see::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MoveType {
	Loud,
	Quiet
}

pub struct MoveSorter {
	killer_table: [[[Option<Move>; 2]; 100]; 2],
	history_table: [[i32; 64]; 64],
	countermove_table: [[Option<Move>; 64]; 64],
	pawn_corrhist: [[f32; Self::CORRHIST_SIZE]; 2],
	see: See
}

impl MoveSorter {
	pub fn new () -> MoveSorter {
		MoveSorter {
			killer_table: [[[None; 2]; 100]; 2],
			history_table: [[0; 64]; 64],
			countermove_table: [[None; 64]; 64],
			pawn_corrhist: [[0.0; Self::CORRHIST_SIZE]; 2],
			see: See::new()
		}
	}

	pub fn sort(&mut self, move_list: &mut Vec<SortedMove>, tt_move: Option<Move>, board: &Board, ply: i32, last_move: Option<Move>) {
		for i in 0..move_list.len() {
			let mv_info = &mut move_list[i];

			if tt_move != None {
				if Some(mv_info.mv) == tt_move {
					mv_info.importance = Self::HASHMOVE_SCORE;
				}
			}

			let is_tt = mv_info.importance == Self::HASHMOVE_SCORE;

			if !is_tt {
				let is_promo = mv_info.mv.promotion != None;
				let mut base = 0;
				let mut increment = 0;

				if mv_info.movetype == MoveType::Loud {
					let capture_score = self.see.see(board, mv_info.mv);

					base = if capture_score > 0 {
						Self::WINNING_CAPTURE
					} else if capture_score == 0{
						Self::NEUTRAL_CAPTURE
					} else {
						Self::LOSING_CAPTURE
					};

					increment = capture_score;
				}
	
				if mv_info.movetype == MoveType::Quiet {
					base = Self::QUIET_MOVE;
	
					mv_info.is_killer = self.is_killer(mv_info.mv, board, ply);
					mv_info.is_countermove = self.is_countermove(mv_info.mv, last_move);
	
					if mv_info.is_killer || mv_info.is_countermove {
						base = 0;

						base += if mv_info.is_killer {
							Self::KILLER_QUIET
						} else {
							0
						};

						base += if mv_info.is_countermove {
							Self::COUNTER_QUIET
						} else {
							0
						};
					} else {
						let history = self.get_history(mv_info.mv);
						increment = history;
						mv_info.history = history;
					}
				}
	
				if is_promo {
					base = if mv_info.mv.promotion.unwrap() == Piece::Queen { 
						Self::PROMO
					} else { 
						Self::UNDER_PROMO
					};
				}

				mv_info.importance = base + increment;
			}
		}

		move_list.sort_by(|x, z| z.importance.cmp(&x.importance));
	}

	pub fn add_killer(&mut self, mv: Move, ply: i32, board: &Board) {
		if ply < 100 && !self.is_killer(mv, board, ply) {
			let color = board.side_to_move();
			let ply_slot = &mut self.killer_table[color as usize][ply as usize];

			ply_slot.rotate_right(1);
			ply_slot[0] = Some(mv);
		}
	}

	pub fn add_history(&mut self, mv: Move, depth: i32) {
		let history = self.history_table[mv.from as usize][mv.to as usize];
		let change = depth * depth + 10;

		if !change.checked_mul(history).is_none() {
			self.history_table[mv.from as usize][mv.to as usize] += change - change * history / Self::HISTORY_MAX; //add quiet score into history table based on from and to squares
		}
	}

	pub fn add_countermove(&mut self, mv: Move, last_move: Move) {
		self.countermove_table[last_move.from as usize][last_move.to as usize] = Some(mv);
	}

	pub fn decay_history(&mut self, mv: Move, depth: i32) {
		let history = self.history_table[mv.from as usize][mv.to as usize];
		let change = depth * depth;

		if !change.checked_mul(history).is_none() {
			self.history_table[mv.from as usize][mv.to as usize] -= change + change * history / Self::HISTORY_MAX; //decay quiet score into history table based on from and to squares
		}
	}

	fn is_killer(&self, mv: Move, board: &Board, ply: i32) -> bool {
		if ply < 100 {
			let color = board.side_to_move();
			let ply_slot = self.killer_table[color as usize][ply as usize];

			for i in 0..ply_slot.len() {
				if !ply_slot[i].is_none() {
					if ply_slot[i].unwrap() == mv {
						return true;
					}
				}
			}
		}

		return false;
	}

	pub fn add_pawn_corrhist(&mut self, board: &Board, depth: i32, best_alpha: i32, static_eval: i32) {
		let idx = (self.pawn_hash(board) % Self::CORRHIST_SIZE as u64) as usize;
		let side = board.side_to_move() as usize;
	
		let entry = &mut self.pawn_corrhist[side][idx];
	
		let weight = f32::min(depth as f32 * depth as f32 + 2.0, 62.0) / 596.0;
		*entry = *entry * (1.0 - weight) + ((best_alpha - static_eval) as f32).clamp(-81.0, 81.0) * 280.0 * weight;
	}

	pub fn read_pawn_corrhist(&mut self, board: &Board) -> f32 {
		let pawn_hist = self.pawn_corrhist[board.side_to_move() as usize][(self.pawn_hash(board) % Self::CORRHIST_SIZE as u64) as usize];
		pawn_hist / 198.0
	}

	fn pawn_hash(&self, board: &Board) -> u64 {
		let mut hash = 0u64;
	
		for square in board.colored_pieces(Color::White, Piece::Pawn) {
			hash ^= Self::KEYS[0][square as usize];
		}
	
		for square in board.colored_pieces(Color::Black, Piece::Pawn) {
			hash ^= Self::KEYS[1][square as usize];
		}
	
		hash
	}

	fn is_countermove(&self, mv: Move, last_move: Option<Move>) -> bool {
		if last_move.is_none() {
			return false;
		}

		return self.countermove_table[last_move.unwrap().from as usize][last_move.unwrap().to as usize] == Some(mv);
	}

	fn get_history(&self, mv: Move) -> i32 {
		return self.history_table[mv.from as usize][mv.to as usize];
	}
}

impl MoveSorter {
	const HASHMOVE_SCORE: i32 = 1000000;

	const PROMO: i32 = 50000;
	const WINNING_CAPTURE: i32 = 50000;

	const NEUTRAL_CAPTURE: i32 = 30000;

	const KILLER_QUIET: i32 = 15000;
	const COUNTER_QUIET: i32 = 10000;
	const QUIET_MOVE: i32 = 0;

	const LOSING_CAPTURE: i32 = -50000;
	const UNDER_PROMO: i32 = -50000;

	const HISTORY_MAX: i32 = 2000;
	const CORRHIST_SIZE: usize = 16384;

	pub const KEYS: [[u64; 64]; 2] = [
		[
			0x9D39247E33776D41, 0x2AF7398005AAA5C7, 0x44DB015024623547, 0x9C15F73E62A76AE2,
			0x75834465489C0C89, 0x3290AC3A203001BF, 0x0FBBAD1F61042279, 0xE83A908FF2FB60CA,
			0x0D7E765D58755C10, 0x1A083822CEAFE02D, 0x9605D5F0E25EC3B0, 0xD021FF5CD13A2ED5,
			0x40BDF15D4A672E32, 0x011355146FD56395, 0x5DB4832046F3D9E5, 0x239F8B2D7FF719CC,
			0x05D1A1AE85B49AA1, 0x679F848F6E8FC971, 0x7449BBFF801FED0B, 0x7D11CDB1C3B7ADF0,
			0x82C7709E781EB7CC, 0xF3218F1C9510786C, 0x331478F3AF51BBE6, 0x4BB38DE5E7219443,
			0xAA649C6EBCFD50FC, 0x8DBD98A352AFD40B, 0x87D2074B81D79217, 0x19F3C751D3E92AE1,
			0xB4AB30F062B19ABF, 0x7B0500AC42047AC4, 0xC9452CA81A09D85D, 0x24AA6C514DA27500,
			0x4C9F34427501B447, 0x14A68FD73C910841, 0xA71B9B83461CBD93, 0x03488B95B0F1850F,
			0x637B2B34FF93C040, 0x09D1BC9A3DD90A94, 0x3575668334A1DD3B, 0x735E2B97A4C45A23,
			0x18727070F1BD400B, 0x1FCBACD259BF02E7, 0xD310A7C2CE9B6555, 0xBF983FE0FE5D8244,
			0x9F74D14F7454A824, 0x51EBDC4AB9BA3035, 0x5C82C505DB9AB0FA, 0xFCF7FE8A3430B241,
			0x3253A729B9BA3DDE, 0x8C74C368081B3075, 0xB9BC6C87167C33E7, 0x7EF48F2B83024E20,
			0x11D505D4C351BD7F, 0x6568FCA92C76A243, 0x4DE0B0F40F32A7B8, 0x96D693460CC37E5D,
			0x42E240CB63689F2F, 0x6D2BDCDAE2919661, 0x42880B0236E4D951, 0x5F0F4A5898171BB6,
			0x39F890F579F92F88, 0x93C5B5F47356388B, 0x63DC359D8D231B78, 0xEC16CA8AEA98AD76,
		],
		[
			0x5355F900C2A82DC7, 0x07FB9F855A997142, 0x5093417AA8A7ED5E, 0x7BCBC38DA25A7F3C,
			0x19FC8A768CF4B6D4, 0x637A7780DECFC0D9, 0x8249A47AEE0E41F7, 0x79AD695501E7D1E8,
			0x14ACBAF4777D5776, 0xF145B6BECCDEA195, 0xDABF2AC8201752FC, 0x24C3C94DF9C8D3F6,
			0xBB6E2924F03912EA, 0x0CE26C0B95C980D9, 0xA49CD132BFBF7CC4, 0xE99D662AF4243939,
			0x27E6AD7891165C3F, 0x8535F040B9744FF1, 0x54B3F4FA5F40D873, 0x72B12C32127FED2B,
			0xEE954D3C7B411F47, 0x9A85AC909A24EAA1, 0x70AC4CD9F04F21F5, 0xF9B89D3E99A075C2,
			0x87B3E2B2B5C907B1, 0xA366E5B8C54F48B8, 0xAE4A9346CC3F7CF2, 0x1920C04D47267BBD,
			0x87BF02C6B49E2AE9, 0x092237AC237F3859, 0xFF07F64EF8ED14D0, 0x8DE8DCA9F03CC54E,
			0x9C1633264DB49C89, 0xB3F22C3D0B0B38ED, 0x390E5FB44D01144B, 0x5BFEA5B4712768E9,
			0x1E1032911FA78984, 0x9A74ACB964E78CB3, 0x4F80F7A035DAFB04, 0x6304D09A0B3738C4,
			0x2171E64683023A08, 0x5B9B63EB9CEFF80C, 0x506AACF489889342, 0x1881AFC9A3A701D6,
			0x6503080440750644, 0xDFD395339CDBF4A7, 0xEF927DBCF00C20F2, 0x7B32F7D1E03680EC,
			0xB9FD7620E7316243, 0x05A7E8A57DB91B77, 0xB5889C6E15630A75, 0x4A750A09CE9573F7,
			0xCF464CEC899A2F8A, 0xF538639CE705B824, 0x3C79A0FF5580EF7F, 0xEDE6C87F8477609D,
			0x799E81F05BC93F31, 0x86536B8CF3428A8C, 0x97D7374C60087B73, 0xA246637CFF328532,
			0x043FCAE60CC0EBA0, 0x920E449535DD359E, 0x70EB093B15B290CC, 0x73A1921916591CBD,
		],
	];
}
//Ranking: TT, Promo, Good Loud Moves (further specifity by SEE), Best Quiets (further specifity by history), Quiets (furhter specifity by history), Bad Loud Moves = Underpromo
//TT will have no specifity, Promos have no specifity