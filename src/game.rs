use rand::{seq::SliceRandom, thread_rng};

pub const COLS: usize = 10;
pub const ROWS: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind { I, O, T, S, Z, J, L }

impl PieceKind {
    pub fn all() -> [PieceKind; 7] {
        [Self::I, Self::O, Self::T, Self::S, Self::Z, Self::J, Self::L]
    }

    pub fn shape(self) -> [[bool; 4]; 4] {
        match self {
            Self::I => [
                [false, false, false, false],
                [true,  true,  true,  true ],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::O => [
                [false, true,  true,  false],
                [false, true,  true,  false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::T => [
                [false, true,  false, false],
                [true,  true,  true,  false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::S => [
                [false, true,  true,  false],
                [true,  true,  false, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::Z => [
                [true,  true,  false, false],
                [false, true,  true,  false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::J => [
                [true,  false, false, false],
                [true,  true,  true,  false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            Self::L => [
                [false, false, true,  false],
                [true,  true,  true,  false],
                [false, false, false, false],
                [false, false, false, false],
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub kind:  PieceKind,
    pub shape: [[bool; 4]; 4],
    pub x:     i32,
    pub y:     i32,
}

impl Piece {
    pub fn new(kind: PieceKind) -> Self {
        Self {
            kind,
            shape: kind.shape(),
            x: (COLS as i32 / 2) - 2,
            y: 0,
        }
    }

    pub fn rotated_cw(&self) -> [[bool; 4]; 4] {
        let mut out = [[false; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                out[c][3 - r] = self.shape[r][c];
            }
        }
        out
    }
}

struct Bag { queue: Vec<PieceKind> }

impl Bag {
    fn new() -> Self {
        let mut b = Self { queue: Vec::new() };
        b.refill();
        b
    }
    fn refill(&mut self) {
        let mut pieces: Vec<PieceKind> = PieceKind::all().to_vec();
        pieces.shuffle(&mut thread_rng());
        self.queue.extend(pieces);
    }
    fn next(&mut self) -> PieceKind {
        if self.queue.len() < 2 { self.refill(); }
        self.queue.remove(0)
    }
    fn peek(&self) -> PieceKind { self.queue[0] }
}

pub type Board = [[Option<PieceKind>; COLS]; ROWS];

pub enum GameEvent { None, Redraw, HardDropped, Quit }

pub struct Game {
    pub board:    Board,
    pub piece:    Piece,
    pub held:     Option<PieceKind>,
    pub can_hold: bool,
    bag:          Bag,
    score:        u32,
    pub lines:    u32,
    pub level:    u32,
    pub game_over: bool,
    pub paused:   bool,
    pub combo:    i32,
}

impl Game {
    pub fn new() -> Self {
        let mut bag = Bag::new();
        let kind = bag.next();
        Self {
            board:     [[None; COLS]; ROWS],
            piece:     Piece::new(kind),
            held:      None,
            can_hold:  true,
            bag,
            score:     0,
            lines:     0,
            level:     1,
            game_over: false,
            paused:    false,
            combo:     0,
        }
    }

    pub fn score(&self) -> u32 { self.score }
    pub fn is_over(&self) -> bool { self.game_over }

    pub fn drop_interval(&self) -> std::time::Duration {
        let ms = (800.0_f64 * 0.85_f64.powi(self.level as i32 - 1)).max(50.0) as u64;
        std::time::Duration::from_millis(ms)
    }

    fn is_valid(&self, shape: &[[bool; 4]; 4], ox: i32, oy: i32) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if !shape[r][c] { continue; }
                let nx = ox + c as i32;
                let ny = oy + r as i32;
                if nx < 0 || nx >= COLS as i32 || ny >= ROWS as i32 { return false; }
                if ny >= 0 && self.board[ny as usize][nx as usize].is_some() { return false; }
            }
        }
        true
    }

    fn lock_piece(&mut self) {
        for r in 0..4 {
            for c in 0..4 {
                if !self.piece.shape[r][c] { continue; }
                let ny = self.piece.y + r as i32;
                let nx = self.piece.x + c as i32;
                if ny < 0 { self.game_over = true; return; }
                self.board[ny as usize][nx as usize] = Some(self.piece.kind);
            }
        }
        self.can_hold = true;
        self.clear_lines();
        self.spawn_next();
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut new_board: Vec<[Option<PieceKind>; COLS]> = Vec::new();
        for r in 0..ROWS {
            if self.board[r].iter().all(|c| c.is_some()) { cleared += 1; }
            else { new_board.push(self.board[r]); }
        }
        while new_board.len() < ROWS { new_board.insert(0, [None; COLS]); }
        for (i, row) in new_board.into_iter().enumerate() { self.board[i] = row; }

        if cleared > 0 {
            self.combo += 1;
            let base = match cleared { 1 => 100, 2 => 300, 3 => 500, 4 => 800, _ => 0 };
            let combo_bonus = if self.combo > 1 { 50 * (self.combo as u32 - 1) } else { 0 };
            self.score += (base + combo_bonus) * self.level;
            self.lines += cleared;
            self.level = (self.lines / 10) + 1;
        } else {
            self.combo = 0;
        }
    }

    fn spawn_next(&mut self) {
        let kind = self.bag.next();
        let p = Piece::new(kind);
        if !self.is_valid(&p.shape, p.x, p.y) { self.game_over = true; }
        else { self.piece = p; }
    }

    pub fn ghost_y(&self) -> i32 {
        let mut gy = self.piece.y;
        while self.is_valid(&self.piece.shape, self.piece.x, gy + 1) { gy += 1; }
        gy
    }

    pub fn next_kind(&self) -> PieceKind { self.bag.peek() }

    pub fn tick(&mut self) {
        if self.game_over || self.paused { return; }
        if self.is_valid(&self.piece.shape, self.piece.x, self.piece.y + 1) {
            self.piece.y += 1;
        } else {
            self.lock_piece();
        }
    }

    pub fn handle_event(&mut self, ev: InputEvent) -> GameEvent {
        if self.game_over { return GameEvent::None; }
        match ev {
            InputEvent::Quit  => return GameEvent::Quit,
            InputEvent::Pause => {
                self.paused = !self.paused;
                return GameEvent::Redraw;
            }
            _ => {}
        }
        if self.paused { return GameEvent::None; }

        match ev {
            InputEvent::Left => {
                if self.is_valid(&self.piece.shape, self.piece.x - 1, self.piece.y) {
                    self.piece.x -= 1;
                }
            }
            InputEvent::Right => {
                if self.is_valid(&self.piece.shape, self.piece.x + 1, self.piece.y) {
                    self.piece.x += 1;
                }
            }
            InputEvent::RotateCW => {
                // SRS Wall Kicks: 0, -1, +1, -2, +2
                let rotated = self.piece.rotated_cw();
                let kicks: &[i32] = match self.piece.kind {
                    PieceKind::I => &[0, -2, 1, -3, 2],
                    _            => &[0, -1, 1, -2, 2],
                };
                for &dx in kicks {
                    if self.is_valid(&rotated, self.piece.x + dx, self.piece.y) {
                        self.piece.shape = rotated;
                        self.piece.x += dx;
                        break;
                    }
                    // Auch einen Tick nach oben versuchen (für enge Stellen)
                    if self.is_valid(&rotated, self.piece.x + dx, self.piece.y - 1) {
                        self.piece.shape = rotated;
                        self.piece.x += dx;
                        self.piece.y -= 1;
                        break;
                    }
                }
            }
            InputEvent::SoftDrop => {
                if self.is_valid(&self.piece.shape, self.piece.x, self.piece.y + 1) {
                    self.piece.y += 1;
                    self.score += 1;
                }
            }
            InputEvent::HardDrop => {
                let gy = self.ghost_y();
                self.score += 2 * (gy - self.piece.y) as u32;
                self.piece.y = gy;
                self.lock_piece();
                return GameEvent::HardDropped; // Drop-Timer zurücksetzen!
            }
            InputEvent::Hold => {
                if self.can_hold {
                    let current_kind = self.piece.kind;
                    match self.held {
                        None => { self.held = Some(current_kind); self.spawn_next(); }
                        Some(held_kind) => {
                            self.held = Some(current_kind);
                            let p = Piece::new(held_kind);
                            if !self.is_valid(&p.shape, p.x, p.y) { self.game_over = true; }
                            else { self.piece = p; }
                        }
                    }
                    self.can_hold = false;
                }
            }
            _ => {}
        }
        GameEvent::Redraw
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent { Left, Right, RotateCW, SoftDrop, HardDrop, Hold, Pause, Quit }