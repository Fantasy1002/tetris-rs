use std::io::{self, Write, BufWriter};
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{self, Clear, ClearType},
    style::{Print, SetForegroundColor, Color, ResetColor, SetAttribute, Attribute},
};
use crate::game::{Game, PieceKind, COLS, ROWS};

const BOARD_COL: u16 = 14;
const BOARD_ROW: u16 = 2;

pub struct Renderer {
    // BufWriter bündelt alle Writes in einen einzigen Systemaufruf → kein Flackern
    out: BufWriter<io::Stdout>,
    initialized: bool,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            out: BufWriter::with_capacity(8192, io::stdout()),
            initialized: false,
        })
    }

    pub fn draw_full(&mut self, game: &Game) -> io::Result<()> {
        let (term_w, term_h) = terminal::size()?;
        if term_w < 50 || term_h < 26 {
            queue!(self.out, MoveTo(0,0), Clear(ClearType::All),
                Print("Terminal zu klein! Bitte auf min. 50x26 vergrößern."))?;
            self.out.flush()?;
            return Ok(());
        }

        // Nur beim ersten Frame alles löschen
        if !self.initialized {
            queue!(self.out, MoveTo(0,0), Clear(ClearType::All))?;
            self.draw_border()?;
            self.draw_controls()?;
            self.initialized = true;
        }

        // Nur das Board und Panels neu zeichnen (kein Clear!)
        self.draw_board(game)?;
        self.draw_side_panels(game)?;

        if game.paused && !game.is_over() {
            self.draw_pause_overlay()?;
        }

        // Cursor verstecken (unten rechts parken)
        queue!(self.out, MoveTo(term_w - 1, term_h - 1))?;

        self.out.flush()?;
        Ok(())
    }

    fn draw_border(&mut self) -> io::Result<()> {
        queue!(self.out, MoveTo(BOARD_COL - 1, BOARD_ROW - 1))?;
        queue!(self.out, SetAttribute(Attribute::Dim), Print("┌"), ResetColor)?;
        for _ in 0..COLS {
            queue!(self.out, SetAttribute(Attribute::Dim), Print("──"), ResetColor)?;
        }
        queue!(self.out, SetAttribute(Attribute::Dim), Print("┐"), ResetColor)?;
        for r in 0..ROWS as u16 {
            queue!(self.out, MoveTo(BOARD_COL - 1, BOARD_ROW + r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
            queue!(self.out, MoveTo(BOARD_COL + COLS as u16 * 2, BOARD_ROW + r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
        }
        queue!(self.out, MoveTo(BOARD_COL - 1, BOARD_ROW + ROWS as u16))?;
        queue!(self.out, SetAttribute(Attribute::Dim), Print("└"), ResetColor)?;
        for _ in 0..COLS {
            queue!(self.out, SetAttribute(Attribute::Dim), Print("──"), ResetColor)?;
        }
        queue!(self.out, SetAttribute(Attribute::Dim), Print("┘"), ResetColor)?;
        Ok(())
    }

    fn draw_board(&mut self, game: &Game) -> io::Result<()> {
        let ghost_y = game.ghost_y();

        // Jede Zelle einzeln schreiben – kein Clear nötig
        for r in 0..ROWS {
            for c in 0..COLS {
                let sx = BOARD_COL + c as u16 * 2;
                let sy = BOARD_ROW + r as u16;

                // Ist diese Zelle Teil des aktiven Pieces?
                let is_active = (0..4).any(|pr| (0..4).any(|pc| {
                    game.piece.shape[pr][pc]
                        && game.piece.y + pr as i32 == r as i32
                        && game.piece.x + pc as i32 == c as i32
                }));

                // Ist diese Zelle Ghost?
                let is_ghost = !is_active && (0..4).any(|pr| (0..4).any(|pc| {
                    game.piece.shape[pr][pc]
                        && ghost_y + pr as i32 == r as i32
                        && game.piece.x + pc as i32 == c as i32
                }));

                queue!(self.out, MoveTo(sx, sy))?;

                if is_active {
                    queue!(self.out,
                        SetForegroundColor(piece_color(game.piece.kind)),
                        Print("██"),
                        ResetColor)?;
                } else if is_ghost {
                    queue!(self.out,
                        SetForegroundColor(Color::DarkGrey),
                        Print("░░"),
                        ResetColor)?;
                } else if let Some(kind) = game.board[r][c] {
                    queue!(self.out,
                        SetForegroundColor(piece_color(kind)),
                        Print("██"),
                        ResetColor)?;
                } else {
                    // Leere Zelle – mit Leerzeichen überschreiben
                    queue!(self.out, Print("  "))?;
                }
            }
        }
        Ok(())
    }

    fn draw_side_panels(&mut self, game: &Game) -> io::Result<()> {
        let right = BOARD_COL + COLS as u16 * 2 + 3;
        let left: u16 = 1;

        // Titel
        queue!(self.out, MoveTo(right, BOARD_ROW))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta), Print("T E T R I S"), ResetColor)?;

        // Score
        queue!(self.out, MoveTo(right, BOARD_ROW + 2))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("SCORE  "), ResetColor)?;
        queue!(self.out, MoveTo(right, BOARD_ROW + 3))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::White),
            Print(format!("{:>8}", game.score())),
            ResetColor)?;

        // Level
        queue!(self.out, MoveTo(right, BOARD_ROW + 5))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("LEVEL  "), ResetColor)?;
        queue!(self.out, MoveTo(right, BOARD_ROW + 6))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Cyan),
            Print(format!("{:>8}", game.level)),
            ResetColor)?;

        // Linien
        queue!(self.out, MoveTo(right, BOARD_ROW + 8))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("LINIEN "), ResetColor)?;
        queue!(self.out, MoveTo(right, BOARD_ROW + 9))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Green),
            Print(format!("{:>8}", game.lines)),
            ResetColor)?;

        // Combo
        queue!(self.out, MoveTo(right, BOARD_ROW + 11))?;
        if game.combo > 1 {
            queue!(self.out, SetForegroundColor(Color::Yellow),
                Print(format!("COMBO x{:<3}", game.combo)), ResetColor)?;
        } else {
            queue!(self.out, Print("          "))?;
        }

        // Nächstes Stück
        queue!(self.out, MoveTo(right, BOARD_ROW + 13))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("NÄCHSTES"), ResetColor)?;
        draw_mini_piece(&mut self.out, game.next_kind(), right, BOARD_ROW + 14)?;

        // Hold
        queue!(self.out, MoveTo(left, BOARD_ROW))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("HALTEN"), ResetColor)?;
        if let Some(held) = game.held {
            let color = if game.can_hold { piece_color(held) } else { Color::DarkGrey };
            draw_mini_piece_color(&mut self.out, held, color, left, BOARD_ROW + 1)?;
        }
        Ok(())
    }

    fn draw_controls(&mut self) -> io::Result<()> {
        let col: u16 = 1;
        let row: u16 = BOARD_ROW + 12;
        let controls = [
            ("←→ ", "Bewegen  "),
            ("↑   ", "Drehen   "),
            ("↓   ", "Soft Drop"),
            ("SPC ", "Hard Drop"),
            ("C   ", "Halten   "),
            ("P   ", "Pause    "),
            ("Q   ", "Beenden  "),
        ];
        for (i, (key, action)) in controls.iter().enumerate() {
            queue!(self.out, MoveTo(col, row + i as u16))?;
            queue!(self.out, SetForegroundColor(Color::DarkCyan), Print(key), ResetColor)?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(action), ResetColor)?;
        }
        Ok(())
    }

    fn draw_pause_overlay(&mut self) -> io::Result<()> {
        let cx = BOARD_COL + COLS as u16 - 5;
        let cy = BOARD_ROW + ROWS as u16 / 2 - 1;
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Yellow), Print("  PAUSE  "), ResetColor)?;
        queue!(self.out, MoveTo(cx - 1, cy + 1))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("P = Weiter"), ResetColor)?;
        Ok(())
    }

    pub fn draw_game_over(&mut self, game: &Game) -> io::Result<()> {
        let cx = BOARD_COL + COLS as u16 - 6;
        let cy = BOARD_ROW + ROWS as u16 / 2 - 2;
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Red), Print("  GAME OVER  "), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy + 1))?;
        queue!(self.out, SetForegroundColor(Color::White),
            Print(format!("  Score: {:>6}  ", game.score())), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy + 3))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey),
            Print("Taste drücken..."), ResetColor)?;
        self.out.flush()?;
        Ok(())
    }
}

fn piece_color(kind: PieceKind) -> Color {
    match kind {
        PieceKind::I => Color::Cyan,
        PieceKind::O => Color::Yellow,
        PieceKind::T => Color::Magenta,
        PieceKind::S => Color::Green,
        PieceKind::Z => Color::Red,
        PieceKind::J => Color::Blue,
        PieceKind::L => Color::AnsiValue(208),
    }
}

fn draw_mini_piece(out: &mut BufWriter<io::Stdout>, kind: PieceKind, ox: u16, oy: u16) -> io::Result<()> {
    draw_mini_piece_color(out, kind, piece_color(kind), ox, oy)
}

fn draw_mini_piece_color(out: &mut BufWriter<io::Stdout>, kind: PieceKind, color: Color, ox: u16, oy: u16) -> io::Result<()> {
    let shape = kind.shape();
    for r in 0..4u16 {
        queue!(out, MoveTo(ox, oy + r))?;
        queue!(out, Print("        "))?;
    }
    for r in 0..4 {
        for c in 0..4 {
            if shape[r][c] {
                queue!(out, MoveTo(ox + c as u16 * 2, oy + r as u16))?;
                queue!(out, SetForegroundColor(color), Print("██"), ResetColor)?;
            }
        }
    }
    Ok(())
}