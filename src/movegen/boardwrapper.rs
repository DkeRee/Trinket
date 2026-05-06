use cozy_chess::*;

pub struct BoardWrapper {
    pub board: Board
}

impl BoardWrapper {
    pub fn new() -> BoardWrapper {
        BoardWrapper {
            board: Board::default()
        }
    }

    pub fn new_set(&self, board: Board) -> BoardWrapper {
        BoardWrapper {
            board: board
        }
    }

    pub fn clone(&self) -> BoardWrapper {
        BoardWrapper {
            board: self.board.clone()
        }
    }

    pub fn update_fen(&mut self, fen: String) {
		self.board = Board::from_fen(&*fen.trim(), false).unwrap();
    }

    pub fn null_move(&self) -> BoardWrapper {
        let null_board = self.board.null_move().unwrap();

        BoardWrapper::new_set(&self, null_board)
    }

    pub fn play_unchecked(&mut self, mv: Move) {
        self.board.play_unchecked(mv);
    }
}