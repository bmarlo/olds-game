struct OldsGame {
    state: [[char; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
    board: String
}

impl OldsGame {
    const BOARD_SIZE: usize = 3;

    fn new() -> OldsGame {
        OldsGame {
            state: [[' '; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
            board: String::new()
        }
    }

    fn play(&mut self) {
        self.draw_board();
    }

    fn draw_board(&mut self) {
        self.board.clear();
        self.board.push('\n');
        let mut i: usize = 0;
        while i < OldsGame::BOARD_SIZE - 1 {
            self.draw_line(i);
            self.draw_break();
            i += 1;
        }

        self.draw_line(i);
        println!("{}", self.board);
    }

    fn draw_line(&mut self, i: usize) {
        for j in 0 .. OldsGame::BOARD_SIZE {
            self.board.push(' ');
            self.board.push(self.state[i][j]);
            self.board.push(' ');
            self.board.push('|');
        }
        self.board.pop();
        self.board.push('\n');
    }

    fn draw_break(&mut self) {
        for _ in 0 .. OldsGame::BOARD_SIZE {
            self.board.push_str("---+");
        }
        self.board.pop();
        self.board.push('\n');
    }
}

fn main() {
    let mut game = OldsGame::new();
    game.play();
}
