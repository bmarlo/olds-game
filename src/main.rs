use std::{io::{self, Write}, time::{SystemTime, UNIX_EPOCH}};

fn random_number() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize
}

struct OldsGame {
    state: [[char; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
    slots: usize,
    board: String
}

impl OldsGame {
    const BOARD_SIZE: usize = 3;

    fn new() -> OldsGame {
        OldsGame {
            state: [[' '; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
            slots: OldsGame::BOARD_SIZE * OldsGame::BOARD_SIZE,
            board: String::new()
        }
    }

    fn play(&mut self) {
        self.draw_board();
        let mut winner = None;
        while winner.is_none() && self.slots > 0 {
            self.make_move();
            winner = self.check_win();
            if winner.is_some() {
                self.draw_board();
                break;
            }
            self.random_move();
            self.draw_board();
            winner = self.check_win();
        }

        if winner.is_some() {
            println!(" --- got winner: {} :D", winner.unwrap());
        } else {
            println!(" --- it's a draw :/");
        }
    }

    fn make_move(&mut self) {
        let (x, y) = self.get_input();
        self.state[x][y] = 'x';
        self.slots -= 1;
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

    fn random_move(&mut self) {
        if self.slots > 0 {
            let (mut x, mut y);
            loop {
                x = random_number() % OldsGame::BOARD_SIZE;
                y = random_number() % OldsGame::BOARD_SIZE;
                if self.state[x][y] == ' ' {
                    break;
                }
            }
            self.state[x][y] = 'o';
            self.slots -= 1;
        }
    }

    fn check_win(&self) -> Option<char> {
        let get_horizontal = |k: usize, m: usize| {
            self.state[k][m]
        };

        let get_vertical = |k: usize, m: usize| {
            self.state[m][k]
        };

        match self.check_straight(get_horizontal) {
            Some(value) => return Some(value),
            None => {}
        }

        match self.check_straight(get_vertical) {
            Some(value) => return Some(value),
            None => {}
        }

        let get_main = |k: usize| {
            self.state[k][k]
        };

        let get_anti = |k: usize| {
            self.state[k][OldsGame::BOARD_SIZE - k - 1]
        };

        match self.check_diagonal(get_main) {
            Some(value) => return Some(value),
            None => {}
        }

        match self.check_diagonal(get_anti) {
            Some(value) => return Some(value),
            None => {}
        }

        None
    }

    fn check_straight<F: Fn(usize, usize) -> char>(&self, get_value: F) -> Option<char> {
        for k in 0 .. OldsGame::BOARD_SIZE {
            let value = get_value(k, 0);
            if value != ' ' {
                let mut all = true;
                for m in 1 .. OldsGame::BOARD_SIZE {
                    all &= get_value(k, m) == value;
                }
                if all {
                    return Some(value);
                }
            }
        }
        None
    }

    fn check_diagonal<F: Fn(usize) -> char>(&self, get_value: F) -> Option<char> {
        let value = get_value(0);
        if value != ' ' {
            let mut all = true;
            for k in 1 .. OldsGame::BOARD_SIZE {
                all &= get_value(k) == value;
            }
            if all {
                return Some(value);
            }
        }
        None
    }
}

fn main() {
    let mut game = OldsGame::new();
    game.play();
}
