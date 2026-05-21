use std::io::{self, Write};
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
    stdout: io::Stdout,
}

impl Renderer {
    pub fn new() -> crossterm::Result<Self> {
        Ok(Self { stdout: io::stdout() })
    }

    pub fn draw_full(&mut self, game: &Game) -> crossterm::Result<()> {
        let (term_w, term_h) = terminal::size()?;
        if term_w < 50 || term_h < 26 {
            queue!(self.stdout, MoveTo(0,0), Clear(ClearType::All),
                Print("Terminal zu klein! Bitte auf min. 50x26 vergrößern."))?;
            self.stdout.flush()?;
            return Ok(());
        }
        queue!(self.stdout, MoveTo(0,0), Clear(ClearType::All))?;
        self.draw_border()?;
        self.draw_board(game)?;
        self.draw_side_panels(game)?;
        self.draw_controls()?;
        if game.paused && !game.is_over() { self.draw_pause_overlay()?; }
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_border(&mut self) -> crossterm::Result<()> {
        queue!(self.stdout, MoveTo(BOARD_COL - 1, BOARD_ROW - 1))?;
        queue!(self.stdout, SetAttribute(Attribute::Dim), Print("┌"), ResetColor)?;
        for _ in 0..COLS {
            queue!(self.stdout, SetAttribute(Attribute::Dim), Print("──"), ResetColor)?;
        }
        queue!(self.stdout, SetAttribute(Attribute::Dim), Print("┐"), ResetColor)?;
        for r in 0..ROWS as u16 {
            queue!(self.stdout, MoveTo(BOARD_COL - 1, BOARD_ROW + r))?;
            queue!(self.stdout, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
            queue!(self.stdout, MoveTo(BOARD_COL + COLS as u16 * 2, BOARD_ROW + r))?;
            queue!(self.stdout, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
        }
        queue!(self.stdout, MoveTo(BOARD_COL - 1, BOARD_ROW + ROWS as u16))?;
        queue!(self.stdout, SetAttribute(Attribute::Dim), Print("└"), ResetColor)?;
        for _ in 0..COLS {
            queue!(self.stdout, SetAttribute(Attribute::Dim), Print("──"), ResetColor)?;
        }
        queue!(self.stdout, SetAttribute(Attribute::Dim), Print("┘"), ResetColor)?;
        Ok(())
    }

    fn draw_board(&mut self, game: &Game) -> crossterm::Result<()> {
        let ghost_y = game.ghost_y();
        for r in 0..ROWS as u16 {
            queue!(self.stdout, MoveTo(BOARD_COL, BOARD_ROW + r))?;
            for _ in 0..COLS { queue!(self.stdout, Print("  "))?; }
        }
        // Locked cells
        for r in 0..ROWS {
            for c in 0..COLS {
                if let Some(kind) = game.board[r][c] {
                    queue!(self.stdout, MoveTo(BOARD_COL + c as u16 * 2, BOARD_ROW + r as u16))?;
                    queue!(self.stdout, SetForegroundColor(piece_color(kind)), Print("██"), ResetColor)?;
                }
            }
        }
        // Ghost
        for r in 0..4 {
            for c in 0..4 {
                if !game.piece.shape[r][c] { continue; }
                let py = ghost_y + r as i32;
                let px = game.piece.x + c as i32;
                if py < 0 || py >= ROWS as i32 || px < 0 || px >= COLS as i32 { continue; }
                if py != game.piece.y + r as i32 {
                    queue!(self.stdout, MoveTo(BOARD_COL + px as u16 * 2, BOARD_ROW + py as u16))?;
                    queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("░░"), ResetColor)?;
                }
            }
        }
        // Active piece
        for r in 0..4 {
            for c in 0..4 {
                if !game.piece.shape[r][c] { continue; }
                let py = game.piece.y + r as i32;
                let px = game.piece.x + c as i32;
                if py < 0 || py >= ROWS as i32 || px < 0 || px >= COLS as i32 { continue; }
                queue!(self.stdout, MoveTo(BOARD_COL + px as u16 * 2, BOARD_ROW + py as u16))?;
                queue!(self.stdout, SetForegroundColor(piece_color(game.piece.kind)), Print("██"), ResetColor)?;
            }
        }
        Ok(())
    }

    fn draw_side_panels(&mut self, game: &Game) -> crossterm::Result<()> {
        let right = BOARD_COL + COLS as u16 * 2 + 3;
        let left: u16 = 1;

        // Title
        queue!(self.stdout, MoveTo(right, BOARD_ROW))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta), Print("T E T R I S"), ResetColor)?;

        // Score
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 2))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("SCORE"), ResetColor)?;
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 3))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::White), Print(format!("{:>8}", game.score)), ResetColor)?;

        // Level
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 5))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("LEVEL"), ResetColor)?;
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 6))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Cyan), Print(format!("{:>8}", game.level)), ResetColor)?;

        // Lines
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 8))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("LINIEN"), ResetColor)?;
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 9))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Green), Print(format!("{:>8}", game.lines)), ResetColor)?;

        // Combo
        if game.combo > 1 {
            queue!(self.stdout, MoveTo(right, BOARD_ROW + 11))?;
            queue!(self.stdout, SetForegroundColor(Color::Yellow),
                Print(format!("COMBO x{}", game.combo)), ResetColor)?;
        }

        // Next piece
        queue!(self.stdout, MoveTo(right, BOARD_ROW + 13))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("NÄCHSTES"), ResetColor)?;
        draw_mini_piece(&mut self.stdout, game.next_kind(), right, BOARD_ROW + 14)?;

        // Hold
        queue!(self.stdout, MoveTo(left, BOARD_ROW))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("HALTEN"), ResetColor)?;
        if let Some(held) = game.held {
            let color = if game.can_hold { piece_color(held) } else { Color::DarkGrey };
            draw_mini_piece_color(&mut self.stdout, held, color, left, BOARD_ROW + 1)?;
        }
        Ok(())
    }

    fn draw_controls(&mut self) -> crossterm::Result<()> {
        let col: u16 = 1;
        let row: u16 = BOARD_ROW + 12;
        let controls = [
            ("←→ ", "Bewegen"),
            ("↑   ", "Drehen"),
            ("↓   ", "Soft Drop"),
            ("SPC ", "Hard Drop"),
            ("C   ", "Halten"),
            ("P   ", "Pause"),
            ("Q   ", "Beenden"),
        ];
        for (i, (key, action)) in controls.iter().enumerate() {
            queue!(self.stdout, MoveTo(col, row + i as u16))?;
            queue!(self.stdout, SetForegroundColor(Color::DarkCyan), Print(key), ResetColor)?;
            queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print(action), ResetColor)?;
        }
        Ok(())
    }

    fn draw_pause_overlay(&mut self) -> crossterm::Result<()> {
        let cx = BOARD_COL + COLS as u16 - 5;
        let cy = BOARD_ROW + ROWS as u16 / 2 - 1;
        queue!(self.stdout, MoveTo(cx, cy))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Yellow), Print("  PAUSE  "), ResetColor)?;
        queue!(self.stdout, MoveTo(cx - 1, cy + 1))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey), Print("P = Weiter"), ResetColor)?;
        Ok(())
    }

    pub fn draw_game_over(&mut self, game: &Game) -> crossterm::Result<()> {
        let cx = BOARD_COL + COLS as u16 - 6;
        let cy = BOARD_ROW + ROWS as u16 / 2 - 2;
        queue!(self.stdout, MoveTo(cx, cy))?;
        queue!(self.stdout, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Red), Print("  GAME OVER  "), ResetColor)?;
        queue!(self.stdout, MoveTo(cx, cy + 1))?;
        queue!(self.stdout, SetForegroundColor(Color::White),
            Print(format!("  Score: {:>6}  ", game.score)), ResetColor)?;
        queue!(self.stdout, MoveTo(cx, cy + 3))?;
        queue!(self.stdout, SetForegroundColor(Color::DarkGrey),
            Print("Taste drücken..."), ResetColor)?;
        self.stdout.flush()?;
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

fn draw_mini_piece(stdout: &mut io::Stdout, kind: PieceKind, ox: u16, oy: u16) -> crossterm::Result<()> {
    draw_mini_piece_color(stdout, kind, piece_color(kind), ox, oy)
}

fn draw_mini_piece_color(stdout: &mut io::Stdout, kind: PieceKind, color: Color, ox: u16, oy: u16) -> crossterm::Result<()> {
    let shape = kind.shape();
    for r in 0..4u16 {
        queue!(stdout, MoveTo(ox, oy + r))?;
        queue!(stdout, Print("        "))?;
    }
    for r in 0..4 {
        for c in 0..4 {
            if shape[r][c] {
                queue!(stdout, MoveTo(ox + c as u16 * 2, oy + r as u16))?;
                queue!(stdout, SetForegroundColor(color), Print("██"), ResetColor)?;
            }
        }
    }
    Ok(())
}