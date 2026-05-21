use rand::{seq::SliceRandom, thread_rng};

pub const COLS: usize = 10;
pub const ROWS: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { De, En }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind { I, O, T, S, Z, J, L }

impl PieceKind {
    fn all() -> [PieceKind; 7] {
        [Self::I, Self::O, Self::T, Self::S, Self::Z, Self::J, Self::L]
    }

    pub fn rotations(self) -> [[(i32, i32); 4]; 4] {
        match self {
            Self::I => [
                [(0,1),(1,1),(2,1),(3,1)],
                [(2,0),(2,1),(2,2),(2,3)],
                [(0,2),(1,2),(2,2),(3,2)],
                [(1,0),(1,1),(1,2),(1,3)],
            ],
            Self::O => [
                [(1,0),(2,0),(1,1),(2,1)],
                [(1,0),(2,0),(1,1),(2,1)],
                [(1,0),(2,0),(1,1),(2,1)],
                [(1,0),(2,0),(1,1),(2,1)],
            ],
            Self::T => [
                [(1,0),(0,1),(1,1),(2,1)],
                [(1,0),(1,1),(2,1),(1,2)],
                [(0,1),(1,1),(2,1),(1,2)],
                [(1,0),(0,1),(1,1),(1,2)],
            ],
            Self::S => [
                [(1,0),(2,0),(0,1),(1,1)],
                [(1,0),(1,1),(2,1),(2,2)],
                [(1,1),(2,1),(0,2),(1,2)],
                [(0,0),(0,1),(1,1),(1,2)],
            ],
            Self::Z => [
                [(0,0),(1,0),(1,1),(2,1)],
                [(2,0),(1,1),(2,1),(1,2)],
                [(0,1),(1,1),(1,2),(2,2)],
                [(1,0),(0,1),(1,1),(0,2)],
            ],
            Self::J => [
                [(0,0),(0,1),(1,1),(2,1)],
                [(1,0),(2,0),(1,1),(1,2)],
                [(0,1),(1,1),(2,1),(2,2)],
                [(1,0),(1,1),(0,2),(1,2)],
            ],
            Self::L => [
                [(2,0),(0,1),(1,1),(2,1)],
                [(1,0),(1,1),(1,2),(2,2)],
                [(0,1),(1,1),(2,1),(0,2)],
                [(0,0),(1,0),(1,1),(1,2)],
            ],
        }
    }
}

struct Bag { queue: Vec<PieceKind> }
impl Bag {
    fn new() -> Self { let mut b = Self { queue: vec![] }; b.refill(); b }
    fn refill(&mut self) {
        let mut p: Vec<PieceKind> = PieceKind::all().to_vec();
        p.shuffle(&mut thread_rng());
        self.queue.extend(p);
    }
    fn next(&mut self) -> PieceKind {
        if self.queue.len() < 2 { self.refill(); }
        self.queue.remove(0)
    }
    fn peek(&self) -> PieceKind { self.queue[0] }
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub kind:     PieceKind,
    pub rotation: usize,
    pub x:        i32,
    pub y:        i32,
}

impl Piece {
    pub fn new(kind: PieceKind) -> Self {
        Self { kind, rotation: 0, x: 3, y: 0 }
    }
    pub fn cells(&self) -> [(i32, i32); 4] {
        let mut out = [(0i32, 0i32); 4];
        for (i, &(cx, cy)) in self.kind.rotations()[self.rotation].iter().enumerate() {
            out[i] = (self.x + cx, self.y + cy);
        }
        out
    }
}

pub type Board = [[Option<PieceKind>; COLS]; ROWS];
pub enum GameEvent { None, Redraw, HardDropped, Quit }

pub struct Game {
    pub board:     Board,
    pub piece:     Piece,
    pub held:      Option<PieceKind>,
    pub can_hold:  bool,
    bag:           Bag,
    pub score:     u32,
    pub lines:     u32,
    pub level:     u32,
    pub game_over: bool,
    pub paused:    bool,
    pub combo:     i32,
    pub lang:      Lang,
}

impl Game {
    pub fn new(lang: Lang) -> Self {
        let mut bag = Bag::new();
        let kind = bag.next();
        Self {
            board: [[None; COLS]; ROWS],
            piece: Piece::new(kind),
            held: None, can_hold: true, bag,
            score: 0, lines: 0, level: 1,
            game_over: false, paused: false, combo: 0,
            lang,
        }
    }

    pub fn is_over(&self) -> bool { self.game_over }

    pub fn drop_interval(&self) -> std::time::Duration {
        let ms = (800.0_f64 * 0.85_f64.powi(self.level as i32 - 1)).max(50.0) as u64;
        std::time::Duration::from_millis(ms)
    }

    fn cells_valid(&self, cells: &[(i32, i32); 4]) -> bool {
        for &(cx, cy) in cells {
            if cx < 0 || cx >= COLS as i32 || cy >= ROWS as i32 { return false; }
            if cy >= 0 && self.board[cy as usize][cx as usize].is_some() { return false; }
        }
        true
    }

    fn lock_piece(&mut self) {
        let cells = self.piece.cells();
        for (cx, cy) in cells {
            if cy < 0 { self.game_over = true; return; }
            self.board[cy as usize][cx as usize] = Some(self.piece.kind);
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
            let base = match cleared { 1=>100, 2=>300, 3=>500, 4=>800, _=>0 };
            let cb = if self.combo > 1 { 50 * (self.combo as u32 - 1) } else { 0 };
            self.score += (base + cb) * self.level;
            self.lines += cleared;
            self.level = (self.lines / 10) + 1;
        } else { self.combo = 0; }
    }

    fn spawn_next(&mut self) {
        let kind = self.bag.next();
        let p = Piece::new(kind);
        if !self.cells_valid(&p.cells()) { self.game_over = true; }
        else { self.piece = p; }
    }

    pub fn ghost_y(&self) -> i32 {
        let mut offset = 0i32;
        loop {
            let test = Piece { y: self.piece.y + offset + 1, ..self.piece.clone() };
            if self.cells_valid(&test.cells()) { offset += 1; } else { break; }
        }
        self.piece.y + offset
    }

    pub fn ghost_cells(&self) -> [(i32, i32); 4] {
        let gy = self.ghost_y();
        Piece { y: gy, ..self.piece.clone() }.cells()
    }

    pub fn next_kind(&self) -> PieceKind { self.bag.peek() }

    pub fn tick(&mut self) {
        if self.game_over || self.paused { return; }
        let moved = Piece { y: self.piece.y + 1, ..self.piece.clone() };
        if self.cells_valid(&moved.cells()) { self.piece.y += 1; }
        else { self.lock_piece(); }
    }

    pub fn handle_event(&mut self, ev: InputEvent) -> GameEvent {
        if self.game_over { return GameEvent::None; }
        match ev {
            InputEvent::Quit  => return GameEvent::Quit,
            InputEvent::Pause => { self.paused = !self.paused; return GameEvent::Redraw; }
            _ => {}
        }
        if self.paused { return GameEvent::None; }
        match ev {
            InputEvent::Left => {
                let p = Piece { x: self.piece.x - 1, ..self.piece.clone() };
                if self.cells_valid(&p.cells()) { self.piece.x -= 1; }
            }
            InputEvent::Right => {
                let p = Piece { x: self.piece.x + 1, ..self.piece.clone() };
                if self.cells_valid(&p.cells()) { self.piece.x += 1; }
            }
            InputEvent::RotateCW => {
                let next_rot = (self.piece.rotation + 1) % 4;
                let kicks: &[(i32, i32)] = match (self.piece.kind, self.piece.rotation) {
                    (PieceKind::I, 0) => &[(0,0),(-2,0),(1,0),(-2,-1),(1,2)],
                    (PieceKind::I, 1) => &[(0,0),(-1,0),(2,0),(-1,2),(2,-1)],
                    (PieceKind::I, 2) => &[(0,0),(2,0),(-1,0),(2,1),(-1,-2)],
                    (PieceKind::I, 3) => &[(0,0),(1,0),(-2,0),(1,-2),(-2,1)],
                    (_, 0) => &[(0,0),(-1,0),(-1,1),(0,-2),(-1,-2)],
                    (_, 1) => &[(0,0),(1,0),(1,-1),(0,2),(1,2)],
                    (_, 2) => &[(0,0),(1,0),(1,1),(0,-2),(1,-2)],
                    (_, 3) => &[(0,0),(-1,0),(-1,-1),(0,2),(-1,2)],
                    _      => &[(0,0)],
                };
                for &(dx, dy) in kicks {
                    let p = Piece {
                        rotation: next_rot,
                        x: self.piece.x + dx,
                        y: self.piece.y + dy,
                        ..self.piece.clone()
                    };
                    if self.cells_valid(&p.cells()) { self.piece = p; break; }
                }
            }
            InputEvent::SoftDrop => {
                let p = Piece { y: self.piece.y + 1, ..self.piece.clone() };
                if self.cells_valid(&p.cells()) { self.piece.y += 1; self.score += 1; }
            }
            InputEvent::HardDrop => {
                let gy = self.ghost_y();
                self.score += 2 * (gy - self.piece.y) as u32;
                self.piece.y = gy;
                self.lock_piece();
                return GameEvent::HardDropped;
            }
            InputEvent::Hold => {
                if self.can_hold {
                    let cur = self.piece.kind;
                    match self.held {
                        None => { self.held = Some(cur); self.spawn_next(); }
                        Some(prev) => {
                            self.held = Some(cur);
                            let p = Piece::new(prev);
                            if !self.cells_valid(&p.cells()) { self.game_over = true; }
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