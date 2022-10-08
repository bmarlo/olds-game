use std::io::{self, Write, Result, Error, ErrorKind};
use std::net::{UdpSocket, SocketAddrV4, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use bytebuffer::{ByteBuffer, Endian};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

fn get_time() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn random_number() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize
}

fn next_number() -> u16 {
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn broadcast(socket: &mut UdpSocket, packet: &[u8], port: u16) -> Result<()> {
    match NetworkInterface::show() {
        Ok(ifaces) => {
            for entry in ifaces {
                match entry.addr {
                    Some(iface) => {
                        match iface.broadcast() {
                            Some(addr) => {
                                if addr.is_ipv4() {
                                    let mut str = addr.to_string();
                                    str.push(':');
                                    str.push_str(port.to_string().as_str());
                                    socket.send_to(packet, &str)?;
                                }
                            },
                            None => {}
                        }
                    },
                    None => {}
                }
            }
            Ok(())
        },
        Err(_) => Err(Error::new(ErrorKind::Other, "can't get interfaces"))
    }
}

struct OldsGame {
    player: char,
    state: [[char; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
    slots: usize,
    board: String,
    socket: UdpSocket,
    bound: bool
}

impl OldsGame {
    const BOARD_SIZE: usize = 3;
    const LAN_PORT: u16 = 54545;
    const TIMEOUT: u128 = 30000;

    fn new() -> OldsGame {
        let mut bound = false;
        let any = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddrV4::new(any, OldsGame::LAN_PORT);
        let socket: Option<UdpSocket>;
        match UdpSocket::bind(addr) {
            Ok(value) => {
                socket = Some(value);
                bound = true;
            },
            Err(_) => {
                let addr = SocketAddrV4::new(any, 0);
                match UdpSocket::bind(addr) {
                    Ok(value) => socket = Some(value),
                    Err(_) => panic!("can't open socket")
                }
            }
        }

        let socket = socket.unwrap();
        socket.set_broadcast(true).expect("can't set socket broadcast option");
        socket.set_nonblocking(true).expect("can't set socket to nonblocking mode");

        OldsGame {
            player: 'x',
            state: [[' '; OldsGame::BOARD_SIZE]; OldsGame::BOARD_SIZE],
            slots: OldsGame::BOARD_SIZE * OldsGame::BOARD_SIZE,
            board: String::new(),
            socket: socket,
            bound: bound
        }
    }

    fn is_localhost(&self) -> bool {
        !self.bound
    }

    fn is_multiplayer(&self) -> bool {
        self.socket.peer_addr().is_ok()
    }

    fn play(&mut self) {
        println!("Finding opponent...");
        if self.is_localhost() {
            if self.connect("127.0.0.1").is_ok() {
                self.multiplayer();
            } else {
                println!(" --- can't connect to remote player");
                self.singleplayer();
            }
        } else {
            let probe = Packet::new(Opcode::Ping);
            if self.send(&probe, true).is_ok() {
                self.multiplayer();
            } else {
                println!(" --- none found, waiting connection");
                if self.accept().is_ok() {
                    self.multiplayer();
                } else {
                    println!(" --- connection timeout");
                    self.singleplayer();
                }
            }
        }
    }

    fn singleplayer(&mut self) {
        println!("\nSingle-player mode");
        self.draw_board();
        let mut winner = None;
        while winner.is_none() && self.slots > 0 {
            self.make_move().unwrap();
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

    fn multiplayer(&mut self) {
        println!(" --- connected to {}", self.socket.peer_addr().unwrap());
        println!("\nMultiplayer mode");

        self.draw_board();
        let mut winner = None;
        if self.player == 'x' {     // we start
            if self.make_move().is_err() {
                winner = Some(self.player);
                println!(" --- timeout");
            }
            self.draw_board();
        }

        while winner.is_none() && self.slots > 0 {
            if self.wait_move().is_err() {
                winner = Some(self.player);
                println!(" --- timeout");
                break;
            }
            self.draw_board();
            winner = self.check_win();
            if winner.is_some() || self.slots == 0 {
                break;
            }
            if self.make_move().is_err() {
                winner = Some(self.player);
                println!(" --- timeout");
                break;
            }
            self.draw_board();
            winner = self.check_win();
        }

        if winner.is_some() {
            let str = if winner.unwrap() == self.player { "win :D" } else { "lose :(" };
            println!(" --- you {}", str);
        } else {
            println!(" --- it's a draw :/");
        }
    }

    fn make_move(&mut self) -> Result<()> {
        let (x, y) = self.get_input();
        self.state[x][y] = self.player;
        self.slots -= 1;
        if self.is_multiplayer() {
            let packet = Packet::new_data(x, y);
            self.send(&packet, false)?;
        }
        Ok(())
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
            stdin.read_line(&mut line).expect("can't get input");
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

    fn wait_move(&mut self) -> Result<()> {
        if !self.is_multiplayer() {
            panic!("not in multiplayer mode");
        }

        println!("Waiting opponent's play...");
        let mut rxbuffer = vec![0; Packet::MAX_SIZE];
        let deadline = get_time() + OldsGame::TIMEOUT;

        while get_time() < deadline {
            let (mut tmp, _) = self.receive(&mut rxbuffer)?;
            if tmp.is_empty() {
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            let mut request = Packet::new(Opcode::Ping);
            if request.parse(&mut tmp) && request.is_data() {
                let data = request.data().unwrap();
                let (x, y) = (data.0 as usize, data.1 as usize);
                self.state[x][y] = if self.player == 'x' { 'o' } else { 'x' };
                self.slots -= 1;
                let mut reply = Packet::new(Opcode::Ack);
                reply.set_number(request.number());
                let bytes = reply.encode().to_bytes();
                match self.socket.send(&bytes) {
                    Ok(_) => return Ok(()),
                    Err(error) => return Err(error)
                }
            }
        }

        Err(Error::new(ErrorKind::TimedOut, "timeout"))
    }

    fn receive(&mut self, mut rxbuffer: &mut [u8]) -> Result<(ByteBuffer, SocketAddr)> {
        let (n, remote);
        match self.socket.recv_from(&mut rxbuffer) {
            Ok(res) => (n, remote) = res,
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    n = 0;
                    remote = SocketAddr::V4(SocketAddrV4::from_str("0.0.0.0:0").unwrap());
                } else {
                    return Err(error);
                }
            }
        }

        if !(n > 0) {
            return Ok((ByteBuffer::new(), remote));
        }

        let mut result = ByteBuffer::from_bytes(&rxbuffer);
        result.set_wpos(n);
        result.set_endian(Endian::BigEndian);
        Ok((result, remote))
    }

    fn send(&mut self, packet: &Packet, bcast: bool) -> Result<()> {
        let bytes = packet.encode().to_bytes();
        let mut rxbuffer = vec![0; Packet::MAX_SIZE];

        for _ in 0 .. 3 {
            if bcast {
                broadcast(&mut self.socket, &bytes, OldsGame::LAN_PORT)?;
            } else {
                self.socket.send(&bytes)?;
            }

            let deadline = get_time() + 1000;
            while get_time() < deadline {
                let (mut tmp, remote) = self.receive(&mut rxbuffer)?;
                if tmp.is_empty() {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let mut reply = Packet::new(Opcode::Ack);
                if reply.parse(&mut tmp) && reply.is_ack() && reply.number() == packet.number() {
                    if bcast {
                        self.socket.connect(remote)?;
                    }
                    return Ok(());
                }
            }
        }

        Err(Error::new(ErrorKind::TimedOut, "timeout"))
    }

    fn connect(&mut self, host: &str) -> Result<()> {
        let host = Ipv4Addr::from_str(host).expect("not an IPv4 literal");
        let addr = SocketAddrV4::new(host, OldsGame::LAN_PORT);
        self.socket.connect(addr)?;
        let ping = Packet::new(Opcode::Ping);
        self.send(&ping, false)
    }

    fn accept(&mut self) -> Result<()> {
        let mut rxbuffer = vec![0; Packet::MAX_SIZE];
        let deadline = get_time() + OldsGame::TIMEOUT;

        while get_time() < deadline {
            let (mut tmp, remote) = self.receive(&mut rxbuffer)?;
            if tmp.is_empty() {
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            let mut request = Packet::new(Opcode::Ping);
            if request.parse(&mut tmp) && request.is_ping() {
                let mut reply = Packet::new(Opcode::Ack);
                reply.set_number(request.number());
                self.socket.connect(remote)?;
                self.player = 'o';
                let bytes = reply.encode().to_bytes();
                match self.socket.send(&bytes) {
                    Ok(_) => return Ok(()),
                    Err(error) => return Err(error)
                }
            }
        }

        Err(Error::new(ErrorKind::TimedOut, "timeout"))
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

#[derive(PartialEq, Clone, Copy)]
enum Opcode {
    Ping = 0x01, Ack, Data
}

struct Packet {
    opcode: Opcode,
    number: u16,
    data: Option<(u16, u16)>
}

#[allow(unused)]
impl Packet {
    const MIN_SIZE: usize = 4;
    const MAX_SIZE: usize = 8;

    fn new(opcode: Opcode) -> Packet {
        Packet {
            opcode: opcode,
            number: next_number(),
            data: None
        }
    }

    fn new_data(x: usize, y: usize) -> Packet {
        Packet {
            opcode: Opcode::Data,
            number: next_number(),
            data: Some((x as u16, y as u16))
        }
    }

    fn is_ping(&self) -> bool {
        self.opcode == Opcode::Ping
    }

    fn is_ack(&self) -> bool {
        self.opcode == Opcode::Ack
    }

    fn is_data(&self) -> bool {
        self.opcode == Opcode::Data
    }

    fn set_number(&mut self, value: u16) {
        self.number = value
    }

    fn set_data(&mut self, x: usize, y: usize) {
        if self.is_data() {
            self.data = Some((x as u16, y as u16));
        }
    }

    fn what(&self) -> Opcode {
        self.opcode
    }

    fn number(&self) -> u16 {
        self.number
    }

    fn data(&self) -> Option<(u16, u16)> {
        self.data
    }

    fn size(&self) -> usize {
        if self.is_data() {
            Packet::MAX_SIZE
        } else {
            Packet::MIN_SIZE
        }
    }

    fn encode(&self) -> ByteBuffer {
        let mut packet = ByteBuffer::new();
        packet.set_endian(Endian::BigEndian);
        packet.write_u16(self.opcode as u16);
        packet.write_u16(self.number);
        if self.is_data() {
            let (x, y) = self.data.expect("data not set");
            packet.write_u16(x);
            packet.write_u16(y);
        }
        packet
    }

    fn parse(&mut self, packet: &mut ByteBuffer) -> bool {
        let size = packet.get_wpos() - packet.get_rpos();
        if size < Packet::MIN_SIZE || size > Packet::MAX_SIZE {
            return false;
        }

        let code = packet.read_u16().unwrap();
        if code < Opcode::Ping as u16 || code > Opcode::Data as u16 {
            return false;
        }

        let opcode = match code {
            1 => Opcode::Ping,
            2 => Opcode::Ack,
            _ => Opcode::Data
        };

        self.opcode = opcode;
        self.number = packet.read_u16().unwrap();
        if self.is_data() {
            if size < Packet::MAX_SIZE {
                return false;
            }
            let x = packet.read_u16().unwrap();
            let y = packet.read_u16().unwrap();
            self.data = Some((x, y));
        }

        packet.get_rpos() == packet.get_wpos()
    }
}

fn main() {
    let mut game = OldsGame::new();
    game.play();
}
