use cozy_chess::*;

use crate::movegen::movegen::*;
use crate::movegen::movesorter::*;

fn init_pawn_hash(board: Board) -> u64 {
	let mut hash = 0u64;
	
	for square in board.colored_pieces(Color::White, Piece::Pawn) {
		hash ^= BoardWrapper::BOARD_BY_PIECE_KEYS[Piece::Pawn as usize][square as usize];
	}
	
	for square in board.colored_pieces(Color::Black, Piece::Pawn) {
		hash ^= BoardWrapper::BOARD_BY_PIECE_KEYS[Piece::Pawn as usize][square as usize];
	}
	
	hash
}

fn init_non_pawn_hash(board: Board) -> [u64; 2] {
    let mut hash = [0u64; 2];

    let pieces = [
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King
    ];

    for color in [Color::White, Color::Black] {
        for piece in pieces {
            for square in board.colored_pieces(color, piece) {
                hash[color as usize] ^= BoardWrapper::BOARD_BY_PIECE_KEYS[piece as usize][square as usize];
            }
        }
    }

    hash
}

fn init_material_hash(board: Board) -> u64 {
    let mut hash = 0u64;

    let pieces = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen
    ];

    for color in [Color::White, Color::Black] {
        for piece in pieces {
            let bb_count = board.colored_pieces(color, piece).len() as usize;
            hash ^= BoardWrapper::COUNT_BY_SIDE_KEYS[color as usize][piece as usize][bb_count];
        }
    }

    hash
}

pub struct BoardWrapper {
    pub board: Board,
    pub pawn_hash: u64,
    pub non_pawn_hash: [u64; 2],
    pub material_hash: u64
}

impl BoardWrapper {
    pub fn new() -> BoardWrapper {
        let new_board = Board::default();

        BoardWrapper {
            board: new_board.clone(),
            pawn_hash: init_pawn_hash(new_board.clone()),
            non_pawn_hash: init_non_pawn_hash(new_board.clone()),
            material_hash: init_material_hash(new_board.clone())
        }
    }

    fn new_set(&self, board: Board) -> BoardWrapper {
        BoardWrapper {
            board: board,
            pawn_hash: self.pawn_hash,
            non_pawn_hash: self.non_pawn_hash,
            material_hash: self.material_hash
        }
    }

    pub fn clone(&self) -> BoardWrapper {
        BoardWrapper {
            board: self.board.clone(),
            pawn_hash: self.pawn_hash,
            non_pawn_hash: self.non_pawn_hash,
            material_hash: self.material_hash
        }
    }

    pub fn update_fen(&mut self, fen: String) {
		self.board = Board::from_fen(&*fen.trim(), false).unwrap();
        self.pawn_hash = init_pawn_hash(self.board.clone());
        self.non_pawn_hash = init_non_pawn_hash(self.board.clone());
        self.material_hash = init_material_hash(self.board.clone());
    }

    pub fn null_move(&self) -> BoardWrapper {
        let null_board = self.board.null_move().unwrap();

        BoardWrapper::new_set(&self, null_board)
    }

    pub fn play_unchecked(&mut self, sm: &mut SortedMove) {
        let mv = sm.mv;
        let us = self.board.side_to_move();
        let enemy = !us;

        //update material history for single capture
        if sm.movetype == MoveType::Loud {
            let captured_piece = self.board.piece_on(mv.to);
            if captured_piece.is_some() {
                let captured_piece_count = self.board.colored_pieces(enemy, captured_piece.unwrap()).len() as usize;

                //remove old state before capture
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[enemy as usize][captured_piece.unwrap() as usize][captured_piece_count];

                //new state after capture
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[enemy as usize][captured_piece.unwrap() as usize][captured_piece_count - 1];
                if captured_piece.unwrap() != Piece::Pawn {
                    self.non_pawn_hash[enemy as usize] ^= Self::BOARD_BY_PIECE_KEYS[captured_piece.unwrap() as usize][mv.to as usize];
                }
            }
        }

        //update pawn hash
        let piece_from = self.board.piece_on(mv.from);
        if piece_from == Some(Piece::Pawn) {
            //remove pawn from source square
            self.pawn_hash ^= Self::BOARD_BY_PIECE_KEYS[Piece::Pawn as usize][mv.from as usize];

            //add pawn to target square
            if mv.promotion.is_none() {
                self.pawn_hash ^= Self::BOARD_BY_PIECE_KEYS[Piece::Pawn as usize][mv.to as usize];
            } else {
                //update material hash for promotions
                let promotion_piece = mv.promotion.unwrap();

                let pawn_piece_count = self.board.colored_pieces(us, Piece::Pawn).len() as usize;
                let promotion_piece_count = self.board.colored_pieces(us, promotion_piece).len() as usize;

                //remove pawn from old state
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[us as usize][Piece::Pawn as usize][pawn_piece_count];
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[us as usize][Piece::Pawn as usize][pawn_piece_count - 1];

                //add promotion in new state
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[us as usize][promotion_piece as usize][promotion_piece_count];
                self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[us as usize][promotion_piece as usize][promotion_piece_count + 1];
                self.non_pawn_hash[us as usize] ^= Self::BOARD_BY_PIECE_KEYS[promotion_piece as usize][mv.to as usize];
            }

            //remove pawn if en passant
            if let Some(ep_file) = self.board.en_passant() {
                if mv.to.file() == ep_file {
                    let captured_rank = match enemy {
                        Color::White => Rank::Fourth,
                        Color::Black => Rank::Fifth
                    };

                    let captured_sq = Square::new(ep_file, captured_rank);
                    let captured_piece = self.board.piece_on(captured_sq).unwrap();
                    let captured_piece_count = self.board.colored_pieces(enemy, captured_piece).len() as usize;
                    
                    //handle en passant for pawn hash
                    if captured_piece == Piece::Pawn {
                        self.pawn_hash ^= Self::BOARD_BY_PIECE_KEYS[Piece::Pawn as usize][captured_sq as usize];
                    }

                    //handle en passant for non pawn captured pieces
                    self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[enemy as usize][captured_piece as usize][captured_piece_count];
                    self.material_hash ^= Self::COUNT_BY_SIDE_KEYS[enemy as usize][captured_piece as usize][captured_piece_count - 1];

                    if captured_piece != Piece::Pawn {
                        self.non_pawn_hash[enemy as usize] ^= Self::BOARD_BY_PIECE_KEYS[captured_piece as usize][captured_sq as usize];
                    }
                }
            }
        } else if piece_from.is_some() {
            //remove piece from source square for nonpawn hash
            self.non_pawn_hash[us as usize] ^= Self::BOARD_BY_PIECE_KEYS[piece_from.unwrap() as usize][mv.from as usize];

            //add piece to target square for nonpawn hash
            self.non_pawn_hash[us as usize] ^= Self::BOARD_BY_PIECE_KEYS[piece_from.unwrap() as usize][mv.to as usize];

            //handle castling for nonpawn corrhist
            if piece_from.unwrap() == Piece::King {
                let from_file = mv.from.file() as i8;
                let to_file = mv.to.file() as i8;

                //is castling move
                if (from_file - to_file).abs() == 2 {
                    let rank = mv.from.rank();

                    let (rook_from, rook_to) = if to_file > from_file {
                        //kingside
                        (
                            Square::new(File::H, rank),
                            Square::new(File::F, rank)
                        )
                    } else {
                        //queenside
                        (
                            Square::new(File::A, rank),
                            Square::new(File::D, rank)
                        )
                    };

                    self.non_pawn_hash[us as usize] ^= Self::BOARD_BY_PIECE_KEYS[Piece::Rook as usize][rook_from as usize];
                    self.non_pawn_hash[us as usize] ^= Self::BOARD_BY_PIECE_KEYS[Piece::Rook as usize][rook_to as usize];
                }
            }
        }

        self.board.play_unchecked(mv);
    }
}

impl BoardWrapper {
	pub const BOARD_BY_PIECE_KEYS: [[u64; 64]; 6] = [
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
        [
            0xA3F1C2D4B5E69807, 0x1C2D3E4F50617283, 0x9F8E7D6C5B4A3928, 0x0123456789ABCDEF,
            0xFEDCBA9876543210, 0x0F1E2D3C4B5A6978, 0x89ABCDEF01234567, 0x76543210FEDCBA98,
            0xC3D2E1F0A9B8C7D6, 0x5A4B3C2D1E0F1020, 0xFFEEDDCCBBAA9988, 0x1122334455667788,
            0x99AABBCCDDEEFF00, 0x0A0B0C0D0E0F1011, 0x1234432112344321, 0xDEAD10CCB16B00B5,
            0xBADC0FFEE0DDF00D, 0xC0FFEE123456789A, 0x13579BDF2468ACE0, 0x2468ACE013579BDF,
            0xFACEB00C0FF1CE00, 0x0D15EA5EDEADBEEF, 0xCAFEBABEDEADBEEF, 0xDEADBEEFCAFEBABE,
            0xABCDEFABCDEFABCD, 0x1234567812345678, 0x8765432187654321, 0x0FEDCBA987654321,
            0x0011223344556677, 0x8899AABBCCDDEEFF, 0xFFEEDDCCBBAA9988, 0x7766554433221100,
            0xA1A2A3A4A5A6A7A8, 0xB1B2B3B4B5B6B7B8, 0xC1C2C3C4C5C6C7C8, 0xD1D2D3D4D5D6D7D8,
            0xE1E2E3E4E5E6E7E8, 0xF1F2F3F4F5F6F7F8, 0x0F0E0D0C0B0A0908, 0x08090A0B0C0D0E0F,
            0xAAAAAAAAFFFFFFFF, 0x5555555500000000, 0x0F0F0F0F0F0F0F0F, 0xF0F0F0F0F0F0F0F0,
            0x33333333CCCCCCCC, 0xCCCCCCCC33333333, 0xAAAAAAAAAAAAAAAA, 0x5555555555555555,
            0xCAFED00DDEADBEEF, 0xFACEFEEDBADC0FFE, 0xDEADC0DE0BADF00D, 0xB16B00B5FACEB00C,
            0xC001D00DC0FFEE00, 0x0BADC0DE0BADC0DE, 0xFEEDFACECAFEBEEF, 0xDEAD10CCDEADBEEF,
            0x13572468ACE0BDF0, 0x2468ACE013572468, 0x89ABCDEF76543210, 0xFEDCBA0987654321,
            0x1020304050607080, 0x8090A0B0C0D0E0F0, 0x0F1F2F3F4F5F6F7F, 0xF0E0D0C0B0A09080,
        ],
        [
            0xD43A1F9E8B7C6D5A, 0x3E2F1D0C4B5A6978, 0x8A7B6C5D4E3F2011, 0x1122334455667788,
            0x99AABBCCDDEEFF00, 0x0F1E2D3C4B5A6978, 0xABCDEF0123456789, 0xFEDCBA9876543210,
            0xCAFEBABEDEADBEEF, 0xDEADBEEFCAFEBABE, 0x0BADF00DDEADC0DE, 0xFACEB00C0FF1CE00,
            0xB16B00B5DEAD10CC, 0xC001D00DDEADBEEF, 0xFEEDFACECAFEBEEF, 0x0D15EA5EDEADBEEF,
            0x13579BDF2468ACE0, 0x2468ACE013579BDF, 0x89ABCDEF01234567, 0x76543210FEDCBA98,
            0x0011223344556677, 0x8899AABBCCDDEEFF, 0xFFEEDDCCBBAA9988, 0x7766554433221100,
            0xA1B2C3D4E5F60718, 0x1827364554637281, 0x1029384756ABCD90, 0x90ABCDEF10293847,
            0x0A0B0C0D0E0F1011, 0x11100F0E0D0C0B0A, 0x1234432112344321, 0xDEADC0DEBAADF00D,
            0xAAAAAAAA55555555, 0x55555555AAAAAAAA, 0x0F0F0F0FF0F0F0F0, 0xF0F0F0F00F0F0F0F,
            0x33333333CCCCCCCC, 0xCCCCCCCC33333333, 0xCAFED00DDEADBEEF, 0xFACEFEEDBADC0FFE,
            0xBADC0FFEE0DDF00D, 0xC0FFEE123456789A, 0xDEAD10CCB16B00B5, 0x0BADC0DE0BADC0DE,
            0xFEEDFACECAFEBEEF, 0xDEADBEEFCAFEBABE, 0xABCDEFABCDEFABCD, 0x1234567812345678,
            0xFEDCBA9876543210, 0x0F1E2D3C4B5A6978, 0x89ABCDEF01234567, 0x76543210FEDCBA98,
            0xA5A5A5A5A5A5A5A5, 0x5A5A5A5A5A5A5A5A, 0xFFFFFFFF00000000, 0x00000000FFFFFFFF,
            0x13572468ACE0BDF0, 0x2468ACE013572468, 0xCAFEBABEDEADBEEF, 0xDEADBEEFCAFEBABE,
            0x667B377985EBCA77, 0x37794EB4C2B2AE3D, 0x4EB4C15F5667B19E, 0xC15F79B9AE3D27D4,
        ],
        [
            0xD6E8FEB86659FD93, 0xA6D5A5C1B7F3D2E1, 0xF3B5C7D9E1A2B4C6, 0x9C4E3F2A1B0D5C7E,
            0x1D2C3B4A59687766, 0x8877665544332211, 0x1122334455667788, 0x99AABBCCDDEEFF00,
            0xFFEEDDCCBBAA0099, 0x0F1E2D3C4B5A6978, 0x1020304050607080, 0x8090A0B0C0D0E0F0,
            0x0A0B0C0D0E0F1011, 0x1A2B3C4D5E6F7081, 0x2B3C4D5E6F708192, 0x3C4D5E6F708192A3,
            0x4D5E6F708192A3B4, 0x5E6F708192A3B4C5, 0x6F708192A3B4C5D6, 0x708192A3B4C5D6E7,
            0x8192A3B4C5D6E7F8, 0x92A3B4C5D6E7F809, 0xA3B4C5D6E7F8091A, 0xB4C5D6E7F8091A2B,
            0xC5D6E7F8091A2B3C, 0xD6E7F8091A2B3C4D, 0xE7F8091A2B3C4D5E, 0xF8091A2B3C4D5E6F,
            0x091A2B3C4D5E6F70, 0x1B2C3D4E5F607182, 0x2C3D4E5F60718293, 0x3D4E5F60718293A4,
            0x4E5F60718293A4B5, 0x5F60718293A4B5C6, 0x60718293A4B5C6D7, 0x718293A4B5C6D7E8,
            0x8293A4B5C6D7E8F9, 0x93A4B5C6D7E8F90A, 0xA4B5C6D7E8F90A1B, 0xB5C6D7E8F90A1B2C,
            0xC6D7E8F90A1B2C3D, 0xD7E8F90A1B2C3D4E, 0xE8F90A1B2C3D4E5F, 0xF90A1B2C3D4E5F60,
            0x0A1B2C3D4E5F6071, 0x1C2D3E4F50617283, 0x2D3E4F5061728394, 0x3E4F5061728394A5,
            0x4F5061728394A5B6, 0x5061728394A5B6C7, 0x61728394A5B6C7D8, 0x728394A5B6C7D8E9,
            0x8394A5B6C7D8E9FA, 0x94A5B6C7D8E9FA0B, 0xA5B6C7D8E9FA0B1C, 0xB6C7D8E9FA0B1C2D,
            0xC7D8E9FA0B1C2D3E, 0xD8E9FA0B1C2D3E4F, 0xE9FA0B1C2D3E4F50, 0xFA0B1C2D3E4F5061,
            0x0B1C2D3E4F506172, 0x1D2E3F5061728394, 0x452821E638D01377, 0xBE5466CF34E90C6C,
        ],
        [
            0xF8D626AAAF278509, 0x0E3FEE3F4A4F8C12, 0x4C8A1B27D5E9F633, 0x7B942ACD183E60AF,
            0xD4F3E9A21C7B5088, 0x6AE2BC5D9F013477, 0x913A6F8D42CEB155, 0x28C5D1E7FA934622,
            0xB1F26D947AC83E10, 0x59A7C31E2D6BF488, 0xEC4A8019F5372BCD, 0x3D62FEA418B97C50,
            0x87BDE214CA5069F1, 0xF21C5893DE467A2E, 0x1458AF3B9C02E677, 0xA63FD5E1709BC248,
            0xC90E347ABF651D33, 0x718DCA0F2E49B856, 0x2AF4B9816D30CE77, 0xE5C2374DA81FB290,
            0x94B80DF6317AE54C, 0x38EFA572C4D019AB, 0xDB14C6F83A275E91, 0x6F90A1BD45EC3372,
            0x1A7E3CD864B2F550, 0xB84F9217DE60AC39, 0xC35D0AFBE1947682, 0x7D21E8436ABF905E,
            0xF6A90D31C7284BE4, 0x0BC57A9EF463D128, 0x5E13DF8A20CB6749, 0xA9F42C71D85E036B,
            0x2748B3EC9D16FA40, 0xD18E640A53BC29F7, 0x68CB2F9E714D8055, 0xFE05A71BD342C69A,
            0x31D9EC8F607A1544, 0x8C24B7A5FD91E263, 0x476AE19D3CB8507F, 0xBAF50326E4D17C18,
            0x03E4C89AF1276DB2, 0xD762A40C5E9B3184, 0x94F0BE6138CA5727, 0x2D81E75AB469F03C,
            0xC658149F72DE8A90, 0x71BC3E5D0AF24768, 0xE92067A3D58CF141, 0x56DA18FC309BE27D,
            0x1FC37AB542D69088, 0xA4835DE71F2CB649, 0xD7E01A8C96F45320, 0x689C42BDF715AE13,
            0x32F7D9840ECB561A, 0xBC159E6AF43027D5, 0xF047C2D91B8EA364, 0x0D8AE57163FC294B,
            0x95B34CF8D1207E6F, 0x47E6A193BC5D8422, 0xEA2F580D716CB937, 0x2B90D4EFC8631578,
            0xC7143AB85D09FE41, 0x6D58E2F130AC7B96, 0x18AF79C4E26D5033, 0xF39C046B8D71A2EC,
        ]
	];


    pub const COUNT_BY_SIDE_KEYS: [[[u64; 16]; 6]; 2] = [
        [
            [
                0xA9D06DB327E439F5, 0x620DD663B377BB12, 0x86242FF22B53D02B, 0x7AB53E341F07EC90,
                0x84B209B5D290055B, 0xA30FEBCE2F02A082, 0x3E0A813BD37B2E18, 0xB8D3AF23681C2889,
                0xE59D7D731F8B2D4A, 0xD3C45A1986AE7F20, 0x489EF1B7B9DFA4C7, 0xCE179A04F63891AB,
                0x12A5B4D80E3FC6DD, 0x6FBB8E9C243D7A61, 0x92DE5C0A81F33E17, 0xFD1047BA55A68CC2
            ],
            [
                0x5AB9F86D7104B2E3, 0x0CE4DAD3E91A6278, 0x79D3180B24CF55A4, 0xEE640C72FBB14DA1,
                0xC8427FA17A2D34F9, 0xA1B4C9076FDD8843, 0x39E03BD52CA7A5D8, 0xB5FA2A8E1D9C43E2,
                0x7C21E45F03B6D99F, 0xF4E09B7C16A25344, 0x685D0EAF42F91C73, 0xD941B2C8EE370A1D,
                0x2E7C4A50B68FDC29, 0x81A6D9F137E2BB05, 0xC30F7BEA948651E7, 0x1DBA32C4F570AA8C
            ],
            [
                0x9F1B6D4A32C0E7F8, 0xB47E0AC9135D62EE, 0x25A4FDC7E1B98241, 0xD80C5BAA6F4371D3,
                0x6A2E91BF04CCF56A, 0xF1D07E283A9B164D, 0x48BC3F9155EEA720, 0xC5E24A76B9D8319E,
                0x7E41A8D2037F4CBB, 0x14F0B7CE8DAA60F2, 0xA2C5D93F7B0165D4, 0xE7A98B0243F2D8C1,
                0x33DE4C618A7BBFE0, 0x8BF62A509C14377A, 0x57C13E9A2D5F4B06, 0xFC7A84E1369DB290
            ],
            [
                0x04F3A9B5D827CE61, 0xBD60C4F218AE5937, 0x6E2D7109F45BC38A, 0xD174AFB86E03E2C5,
                0x92C58E370A6D4F19, 0x3B9AEF1427D8C6E4, 0xF8D214CB605E9173, 0x58C47A2D91B3EF00,
                0xAE1935F04CDA6288, 0x1C8BF7E36952A4D1, 0xC27E50ABF4D1B63E, 0x74D8A91C20EF573A,
                0xE4C70D3B196A8F25, 0x29A5FEC8461370BD, 0xB13748D92CE5AA4F, 0x83F20B5A7DE46C92
            ],
            [
                0xFA103CB5D26E9847, 0x61A9EF42C0D3748A, 0xC53B7D0A1EF68231, 0x09E7C1B5FA483D62,
                0xD65A0F91B3CC7E19, 0x34FEC208A9D76144, 0xB0C41E7F2D9358AB, 0x72D95ABF10EC64F3,
                0x1A7B04E5C39FD2D8, 0x8DFA6731B540AE06, 0x47C8D2AF6E1B395C, 0xF31E9B7084CD25A1,
                0x2CB6E541D89AF730, 0x98A347F1C50D6BE5, 0xE05D28BC7134CFA9, 0x56F18D049AE27C13
            ],
            [
                0x8E4C20A7D9F36BB1, 0x15D3BF4E6809A254, 0xC9F271B5A34ED07A, 0x6BF8402D17C59CE3,
                0xF7A10D9B2E4386C0, 0x3A5CE87140DFBA26, 0xA413F09E75B84D1C, 0xDB6205A9CF17E873,
                0x20ED4B3681FA5C9D, 0x9ACF7502D14B63E8, 0xE6B1D38F4C2A9715, 0x53A8CE907D6F20B4,
                0xBC47E2F135A0D669, 0x0F92AB64E3D81C57, 0x7D6143B9FA25EE02, 0xD2E84C7016BF498A
            ]
        ],
        [
            [
                0xB6D3F18A24EC7095, 0x43A7C20D9185BEF1, 0xD90B64F7EA3102AC, 0x27CE9A583D46F8B3,
                0xF102E74BA5C9136D, 0x6D85CB1F407EA924, 0x9B47A6D0F2C58E11, 0x3CFA28B5619DE047,
                0xE14C970AB83F5D62, 0x52B0ED3C7649A18F, 0xAC7F01D943E6B250, 0x0D98B5A2CF7E3C79,
                0x74E1D02B958AF4CE, 0xC3287BF651D40AA3, 0x18FA4E6D3B72C591, 0xF69D1308AE4CB7E8
            ],
            [
                0x39A4C5E270DB816F, 0x8F16E2D04BA93C25, 0xD4E93AF81560B742, 0x21B7C40DAE6F58C9,
                0xA80D7F3492E1BC50, 0x5F3EC28176A409DD, 0xEBC7904F3A62561B, 0x0A51DE98C74F32E4,
                0xC7D420A63E9B85A8, 0x61AF9E14508D7C33, 0x97C2F805DB134E6A, 0x34E68A1BC5F72091,
                0xF3B109DE82764D7E, 0x4CD8A57B19E0F2C0, 0xB24FCE361D8A9B15, 0x1583D6FA40C75EE9
            ],
            [
                0xE2D641A70BC59F48, 0x56B7C0D91EAF3421, 0x93FC15A4D8276BE0, 0x2B408E6F7159CD37,
                0xCA19D3B584F2A670, 0x7E05AFC241DB9832, 0x148D72EB50C4F11D, 0xB9F630AC8D1E75A4,
                0x04AB91F672CDE388, 0xDFA72C1459B0AE6F, 0x6C51E3B8F2047A59, 0x80D49FA37E16BC22,
                0x37C5B019D4FA684E, 0xF0A86E52BC731D94, 0x5D2147C6E89FAB03, 0xA71BC4D350E265DA
            ],
            [
                0x11C5FAE78034BD6E, 0x8AE1D9534CB07F29, 0xC63F40A9DE278154, 0x3D728BF105EC96C3,
                0xF89A2476B31DC05F, 0x65DE30BC948AF7E1, 0x9C170A4D52F36BA8, 0x2E49D8F0A16CB534,
                0xD7BCA35048E17A62, 0x40FE158C93D42D0B, 0xBAE2735F6C19E8C4, 0x073D94A1F5B0629D,
                0xEC58B246AD7F1037, 0x59A6D13CE8047BFA, 0xA4F0387B6215CD81, 0x1BFDE6529C8E4A50
            ],
            [
                0x9D4317F5AE20C64C, 0x28B6EC904DF719E3, 0xF741D3A8625B8F11, 0x4A09BE37C6E18475,
                0xC25ED0149BF63AA0, 0x16A8F94730DC527E, 0xB8D204E76F15CB39, 0x63CF1A5809EAB4D2,
                0x0C7BE34D92F67018, 0xE59A10BC48D37F65, 0x35D4C6E12A8FB903, 0x8B12AF70D64CEEDA,
                0xD0683B59F12457A1, 0x41EFC8057B9AD23C, 0xAF37D196C58E608B, 0x72B0E4FA13CD9546
            ],
            [
                0xF4C72D318E0AB67F, 0x5B190AE4D36FC8D2, 0x82DE47B95014A3A6, 0x1E4A8CF263DB7190,
                0xC916F2A07D58EC3B, 0x30B7D8491FC5A46E, 0xE7A25C03B64F12D5, 0x0F6D934E81AB7BC1,
                0xA3C50871DE2948FA, 0x68F14D2CB037E506, 0xDB920E57A4C16F84, 0x25AC6B194FD87A3D,
                0x94E3F7C80A5162B7, 0x47D8093EBC2F0CD9, 0xBE1574A260D3E841, 0x13FAE0D58C79B52A
            ]
        ]
    ];
}