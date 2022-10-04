use std::io::{self, Write};

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
        loop {
            self.make_move();
            self.draw_board();
        }
    }

    fn make_move(&mut self) {
        let (x, y) = self.get_input();
        self.state[x][y] = 'x';
    }

    fn get_input(&self) -> (usize, usize) {
        #[allow(unused_assignments)]
        let (mut x, mut y) = (0, 0);
        let mut line = String::new();
        let stdin = io::stdin();

        loop {
            line.clear();
            print!("Enter position: ");
            io::stdout().flush().ok();
            stdin.read_line(&mut line).unwrap();
            line = line.trim().to_owned();
            let entries: Vec<&str> = line.split_whitespace().collect();
            if entries.len() != 2 {
                println!(" --- bad input");
                continue;
            }
            match entries.get(0).unwrap().parse() {
                Ok(value) => x = value,
                Err(_) => {
                    println!(" --- bad input");
                    continue;
                }
            }
            match entries.get(1).unwrap().parse() {
                Ok(value) => y = value,
                Err(_) => {
                    println!(" --- bad input");
                    continue;
                }
            }
            if !(x < OldsGame::BOARD_SIZE) || !(y < OldsGame::BOARD_SIZE) || self.state[x][y] != ' ' {
                println!(" --- bad position");
                continue;
            }
            break;
        }

        (x, y)
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
