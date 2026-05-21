use std::io::{self, Write, BufWriter};
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{self, Clear, ClearType},
    style::{Print, SetForegroundColor, Color, ResetColor, SetAttribute, Attribute},
};
use crate::game::{Game, PieceKind, COLS, ROWS};

const BC: u16 = 13;
const BR: u16 = 2;

pub struct Renderer {
    out:  BufWriter<io::Stdout>,
    init: bool,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        Ok(Self { out: BufWriter::with_capacity(16384, io::stdout()), init: false })
    }

    pub fn draw_full(&mut self, game: &Game) -> io::Result<()> {
        let (tw, th) = terminal::size()?;
        if tw < 52 || th < 24 {
            queue!(self.out, MoveTo(0,0), Clear(ClearType::All),
                Print("Terminal zu klein (min. 52x24)!"))?;
            self.out.flush()?;
            return Ok(());
        }
        if !self.init {
            queue!(self.out, Clear(ClearType::All))?;
            self.draw_border()?;
            self.draw_static_labels()?;
            self.init = true;
        }
        self.draw_board(game)?;
        self.draw_dynamic_panels(game)?;
        if game.paused && !game.is_over() { self.draw_pause()?; }
        queue!(self.out, MoveTo(tw-1, th-1))?;
        self.out.flush()
    }

    fn draw_border(&mut self) -> io::Result<()> {
        queue!(self.out, MoveTo(BC-1, BR-1))?;
        queue!(self.out, SetAttribute(Attribute::Dim),
            Print(format!("┌{}┐", "──".repeat(COLS))), ResetColor)?;
        for r in 0..ROWS as u16 {
            queue!(self.out, MoveTo(BC-1, BR+r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
            queue!(self.out, MoveTo(BC + COLS as u16*2, BR+r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
        }
        queue!(self.out, MoveTo(BC-1, BR+ROWS as u16))?;
        queue!(self.out, SetAttribute(Attribute::Dim),
            Print(format!("└{}┘", "──".repeat(COLS))), ResetColor)?;
        Ok(())
    }

    fn draw_static_labels(&mut self) -> io::Result<()> {
        let r = BC + COLS as u16*2 + 3;
        queue!(self.out, MoveTo(r, BR))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta), Print("TETRIS"), ResetColor)?;
        for (row, label) in [
            (BR+2,"SCORE"), (BR+5,"LEVEL"), (BR+8,"LINIEN"), (BR+13,"NÄCHSTES")
        ] {
            queue!(self.out, MoveTo(r, row))?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(label), ResetColor)?;
        }
        queue!(self.out, MoveTo(1, BR))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("HOLD"), ResetColor)?;
        let controls = [
            "← →  bewegen", "↑    drehen",  "↓    soft drop",
            "SPC  hard drop","C    hold",    "P    pause","Q    beenden",
        ];
        for (i, line) in controls.iter().enumerate() {
            queue!(self.out, MoveTo(1, BR+12+i as u16))?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(line), ResetColor)?;
        }
        Ok(())
    }

    fn draw_board(&mut self, game: &Game) -> io::Result<()> {
        let active = game.piece.cells();
        let ghost  = game.ghost_cells();
        for r in 0..ROWS {
            for c in 0..COLS {
                let sx = BC + c as u16 * 2;
                let sy = BR + r as u16;
                let pos = (c as i32, r as i32);
                let is_active = active.iter().any(|&(ax,ay)| ax==pos.0 && ay==pos.1);
                let is_ghost  = !is_active && ghost.iter().any(|&(gx,gy)| gx==pos.0 && gy==pos.1);
                queue!(self.out, MoveTo(sx, sy))?;
                if is_active {
                    queue!(self.out, SetForegroundColor(kind_color(game.piece.kind)), Print("██"), ResetColor)?;
                } else if is_ghost {
                    queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("░░"), ResetColor)?;
                } else if let Some(k) = game.board[r][c] {
                    queue!(self.out, SetForegroundColor(kind_color(k)), Print("██"), ResetColor)?;
                } else {
                    queue!(self.out, Print("  "))?;
                }
            }
        }
        Ok(())
    }

    fn draw_dynamic_panels(&mut self, game: &Game) -> io::Result<()> {
        let r = BC + COLS as u16*2 + 3;
        queue!(self.out, MoveTo(r, BR+3))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::White),
            Print(format!("{:>8}", game.score)), ResetColor)?;
        queue!(self.out, MoveTo(r, BR+6))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::Cyan),
            Print(format!("{:>8}", game.level)), ResetColor)?;
        queue!(self.out, MoveTo(r, BR+9))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::Green),
            Print(format!("{:>8}", game.lines)), ResetColor)?;
        queue!(self.out, MoveTo(r, BR+11))?;
        if game.combo > 1 {
            queue!(self.out, SetForegroundColor(Color::Yellow),
                Print(format!("COMBO x{:<4}", game.combo)), ResetColor)?;
        } else {
            queue!(self.out, Print("           "))?;
        }
        draw_piece_preview(&mut self.out, game.next_kind(), r, BR+14)?;
        // Hold area leeren
        for i in 0..4u16 {
            queue!(self.out, MoveTo(1, BR+1+i), Print("        "))?;
        }
        if let Some(h) = game.held {
            let color = if game.can_hold { kind_color(h) } else { Color::DarkGrey };
            draw_piece_preview_color(&mut self.out, h, color, 1, BR+1)?;
        }
        Ok(())
    }

    fn draw_pause(&mut self) -> io::Result<()> {
        let cx = BC + COLS as u16 - 4;
        let cy = BR + ROWS as u16 / 2;
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Yellow), Print("[ PAUSE ]"), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy+1))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print("P=Weiter "), ResetColor)?;
        Ok(())
    }

    pub fn draw_game_over(&mut self, game: &Game) -> io::Result<()> {
        let cx = BC + COLS as u16 - 6;
        let cy = BR + ROWS as u16 / 2 - 1;
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Red), Print("[ GAME OVER ]"), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy+1))?;
        queue!(self.out, SetForegroundColor(Color::White),
            Print(format!("  Score: {:>6}", game.score)), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy+3))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey),
            Print("Taste drücken..."), ResetColor)?;
        self.out.flush()
    }
}

fn kind_color(k: PieceKind) -> Color {
    match k {
        PieceKind::I => Color::Cyan,
        PieceKind::O => Color::Yellow,
        PieceKind::T => Color::Magenta,
        PieceKind::S => Color::Green,
        PieceKind::Z => Color::Red,
        PieceKind::J => Color::Blue,
        PieceKind::L => Color::AnsiValue(208),
    }
}

fn draw_piece_preview(out: &mut BufWriter<io::Stdout>, kind: PieceKind, ox: u16, oy: u16) -> io::Result<()> {
    draw_piece_preview_color(out, kind, kind_color(kind), ox, oy)
}

fn draw_piece_preview_color(out: &mut BufWriter<io::Stdout>, kind: PieceKind, color: Color, ox: u16, oy: u16) -> io::Result<()> {
    for r in 0..4u16 {
        queue!(out, MoveTo(ox, oy+r), Print("        "))?;
    }
    for (cx, cy) in kind.rotations()[0] {
        queue!(out, MoveTo(ox + cx as u16 * 2, oy + cy as u16))?;
        queue!(out, SetForegroundColor(color), Print("██"), ResetColor)?;
    }
    Ok(())
}