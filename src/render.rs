use std::io::{self, Write, BufWriter};
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{self, Clear, ClearType},
    style::{Print, SetForegroundColor, Color, ResetColor, SetAttribute, Attribute},
};
use crate::game::{Game, Lang, PieceKind, COLS, ROWS};

const BOARD_W: u16 = (COLS as u16) * 2 + 2;
const BOARD_H: u16 = ROWS as u16 + 2;
const PANEL_R: u16 = 14; // rechtes Panel Breite
const PANEL_L: u16 = 10; // linkes Panel Breite

fn board_origin(tw: u16, th: u16) -> (u16, u16) {
    let total_w = PANEL_L + BOARD_W + PANEL_R;
    let bc = tw.saturating_sub(total_w) / 2 + PANEL_L;
    let br = th.saturating_sub(BOARD_H + 6) / 2 + 1;
    (bc, br)
}

pub struct Renderer {
    out:     BufWriter<io::Stdout>,
    last_tw: u16,
    last_th: u16,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let (_tw, _th) = terminal::size().unwrap_or((80, 24));
        Ok(Self {
            out: BufWriter::with_capacity(16384, io::stdout()),
            last_tw: 0,   // 0 erzwingt vollständigen Redraw beim ersten Frame
            last_th: 0,
        })
    }

    pub fn draw_full(&mut self, game: &Game) -> io::Result<()> {
        let (tw, th) = terminal::size()?;

        if tw < 52 || th < 26 {
            queue!(self.out, MoveTo(0, 0), Clear(ClearType::All),
                Print("Terminal too small (min. 52x26)!"))?;
            self.out.flush()?;
            return Ok(());
        }

        let (bc, br) = board_origin(tw, th);

        // Vollständiger Redraw bei Resize oder erstem Frame
        if tw != self.last_tw || th != self.last_th {
            self.last_tw = tw;
            self.last_th = th;
            queue!(self.out, Clear(ClearType::All))?;
            self.draw_border(bc, br)?;
            self.draw_static_labels(game.lang, bc, br)?;
        }

        self.draw_board(game, bc, br)?;
        self.draw_dynamic_panels(game, bc, br)?;

        if game.paused && !game.is_over() {
            self.draw_pause(game.lang, bc, br)?;
        }

        queue!(self.out, MoveTo(tw - 1, th - 1))?;
        self.out.flush()
    }

    fn draw_border(&mut self, bc: u16, br: u16) -> io::Result<()> {
        // Obere Linie
        queue!(self.out, MoveTo(bc - 1, br - 1))?;
        queue!(self.out, SetAttribute(Attribute::Dim),
            Print(format!("┌{}┐", "──".repeat(COLS))), ResetColor)?;
        // Seitenlinien
        for r in 0..ROWS as u16 {
            queue!(self.out, MoveTo(bc - 1, br + r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
            queue!(self.out, MoveTo(bc + COLS as u16 * 2, br + r))?;
            queue!(self.out, SetAttribute(Attribute::Dim), Print("│"), ResetColor)?;
        }
        // Untere Linie
        queue!(self.out, MoveTo(bc - 1, br + ROWS as u16))?;
        queue!(self.out, SetAttribute(Attribute::Dim),
            Print(format!("└{}┘", "──".repeat(COLS))), ResetColor)?;
        Ok(())
    }

    fn draw_static_labels(&mut self, lang: Lang, bc: u16, br: u16) -> io::Result<()> {
        let rp = bc + COLS as u16 * 2 + 3;
        let lp = bc.saturating_sub(PANEL_L);

        // Titel über dem Board
        queue!(self.out, MoveTo(bc + COLS as u16 - 3, br - 1))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Magenta), Print("TETRIS"), ResetColor)?;

        let (score_lbl, level_lbl, lines_lbl, next_lbl, hold_lbl) = match lang {
            Lang::De => ("PUNKTE", "LEVEL", "LINIEN", "NÄCHSTES", "HALTEN"),
            Lang::En => ("SCORE",  "LEVEL", "LINES",  "NEXT",     "HOLD"),
        };

        for (row, label) in [
            (br + 2,  score_lbl),
            (br + 5,  level_lbl),
            (br + 8,  lines_lbl),
            (br + 13, next_lbl),
        ] {
            queue!(self.out, MoveTo(rp, row))?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(label), ResetColor)?;
        }

        // Hold Label
        queue!(self.out, MoveTo(lp, br))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(hold_lbl), ResetColor)?;

        // Steuerung unter dem Board (2 Spalten)
        let controls: &[(&str, &str)] = match lang {
            Lang::De => &[
                ("A/←  links",    "D/→  rechts"),
                ("W/↑  drehen",   "S/↓  soft drop"),
                ("SPC  hard drop","C    halten"),
                ("P    pause",    "Q    beenden"),
            ],
            Lang::En => &[
                ("A/←  left",     "D/→  right"),
                ("W/↑  rotate",   "S/↓  soft drop"),
                ("SPC  hard drop","C    hold"),
                ("P    pause",    "Q    quit"),
            ],
        };

        let ctrl_y = br + ROWS as u16 + 2;
        for (i, (left, right)) in controls.iter().enumerate() {
            queue!(self.out, MoveTo(bc, ctrl_y + i as u16))?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(left), ResetColor)?;
            queue!(self.out, MoveTo(bc + COLS as u16, ctrl_y + i as u16))?;
            queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(right), ResetColor)?;
        }

        Ok(())
    }

    fn draw_board(&mut self, game: &Game, bc: u16, br: u16) -> io::Result<()> {
        let active = game.piece.cells();
        let ghost  = game.ghost_cells();

        for r in 0..ROWS {
            for c in 0..COLS {
                let sx = bc + c as u16 * 2;
                let sy = br + r as u16;
                let pos = (c as i32, r as i32);
                let is_active = active.iter().any(|&(ax, ay)| ax == pos.0 && ay == pos.1);
                let is_ghost  = !is_active && ghost.iter().any(|&(gx, gy)| gx == pos.0 && gy == pos.1);

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

    fn draw_dynamic_panels(&mut self, game: &Game, bc: u16, br: u16) -> io::Result<()> {
        let rp = bc + COLS as u16 * 2 + 3;
        let lp = bc.saturating_sub(PANEL_L);

        queue!(self.out, MoveTo(rp, br + 3))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::White),
            Print(format!("{:>8}", game.score)), ResetColor)?;

        queue!(self.out, MoveTo(rp, br + 6))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::Cyan),
            Print(format!("{:>8}", game.level)), ResetColor)?;

        queue!(self.out, MoveTo(rp, br + 9))?;
        queue!(self.out, SetAttribute(Attribute::Bold), SetForegroundColor(Color::Green),
            Print(format!("{:>8}", game.lines)), ResetColor)?;

        queue!(self.out, MoveTo(rp, br + 11))?;
        if game.combo > 1 {
            queue!(self.out, SetForegroundColor(Color::Yellow),
                Print(format!("COMBO x{:<4}", game.combo)), ResetColor)?;
        } else {
            queue!(self.out, Print("           "))?;
        }

        draw_piece_preview(&mut self.out, game.next_kind(), rp, br + 14)?;

        // Hold area
        for i in 0..4u16 {
            queue!(self.out, MoveTo(lp, br + 1 + i), Print("        "))?;
        }
        if let Some(h) = game.held {
            let color = if game.can_hold { kind_color(h) } else { Color::DarkGrey };
            draw_piece_preview_color(&mut self.out, h, color, lp, br + 1)?;
        }

        Ok(())
    }

    fn draw_pause(&mut self, lang: Lang, bc: u16, br: u16) -> io::Result<()> {
        let cx = bc + COLS as u16 - 4;
        let cy = br + ROWS as u16 / 2;
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Yellow), Print("[ PAUSE ]"), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy + 1))?;
        let msg = match lang { Lang::De => "P=Weiter ", Lang::En => "P=Resume " };
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(msg), ResetColor)?;
        Ok(())
    }

    pub fn draw_game_over(&mut self, game: &Game) -> io::Result<()> {
        let (tw, th) = terminal::size()?;
        let (bc, br) = board_origin(tw, th);
        let cx = bc + COLS as u16 - 6;
        let cy = br + ROWS as u16 / 2 - 1;
        let (title, score_lbl, press_lbl) = match game.lang {
            Lang::De => ("[ GAME OVER ]", "Punkte:", "Taste drücken..."),
            Lang::En => ("[ GAME OVER ]", "Score: ", "Press any key..."),
        };
        queue!(self.out, MoveTo(cx, cy))?;
        queue!(self.out, SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Red), Print(title), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy + 1))?;
        queue!(self.out, SetForegroundColor(Color::White),
            Print(format!("  {score_lbl} {:>6}", game.score)), ResetColor)?;
        queue!(self.out, MoveTo(cx, cy + 3))?;
        queue!(self.out, SetForegroundColor(Color::DarkGrey), Print(press_lbl), ResetColor)?;
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
        queue!(out, MoveTo(ox, oy + r), Print("        "))?;
    }
    for (cx, cy) in kind.rotations()[0] {
        queue!(out, MoveTo(ox + cx as u16 * 2, oy + cy as u16))?;
        queue!(out, SetForegroundColor(color), Print("██"), ResetColor)?;
    }
    Ok(())
}