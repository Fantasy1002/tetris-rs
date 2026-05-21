use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
    style::{Print, SetForegroundColor, Color, ResetColor, SetAttribute, Attribute},
};
use crate::game::{InputEvent, Lang};

pub fn poll_input(timeout: Duration) -> io::Result<Option<InputEvent>> {
    if !event::poll(timeout)? { return Ok(None); }
    match event::read()? {
        Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
            if kind == KeyEventKind::Release { return Ok(None); }
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return Ok(Some(InputEvent::Quit));
            }
            let ev = match code {
                KeyCode::Left  | KeyCode::Char('a') | KeyCode::Char('A') => InputEvent::Left,
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => InputEvent::Right,
                KeyCode::Up    | KeyCode::Char('w') | KeyCode::Char('W') => InputEvent::RotateCW,
                KeyCode::Down  | KeyCode::Char('s') | KeyCode::Char('S') => InputEvent::SoftDrop,
                KeyCode::Char(' ')                                         => InputEvent::HardDrop,
                KeyCode::Char('c') | KeyCode::Char('C')                   => InputEvent::Hold,
                KeyCode::Char('p') | KeyCode::Char('P')                   => InputEvent::Pause,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc    => InputEvent::Quit,
                _ => return Ok(None),
            };
            Ok(Some(ev))
        }
        _ => Ok(None),
    }
}

pub fn choose_language() -> io::Result<Lang> {
    let mut stdout = io::stdout();

    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    // Box
    queue!(stdout, MoveTo(10, 4))?;
    queue!(stdout, SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Magenta),
        Print("╔══════════════════════════════╗"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 5))?;
    queue!(stdout, SetForegroundColor(Color::Magenta),
        Print("║                              ║"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 6))?;
    queue!(stdout, SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Magenta),
        Print("║         T E T R I S          ║"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 7))?;
    queue!(stdout, SetForegroundColor(Color::Magenta),
        Print("║                              ║"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 8))?;
    queue!(stdout, SetForegroundColor(Color::Magenta),
        Print("╚══════════════════════════════╝"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 10))?;
    queue!(stdout, SetForegroundColor(Color::White),
        Print("  Sprache / Language:"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 12))?;
    queue!(stdout, SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("  [1]  Deutsch"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 13))?;
    queue!(stdout, SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold),
        Print("  [2]  English"),
        ResetColor)?;

    queue!(stdout, MoveTo(10, 15))?;
    queue!(stdout, SetForegroundColor(Color::DarkGrey),
        Print("  Drücke 1 oder 2 / Press 1 or 2"),
        ResetColor)?;

    stdout.flush()?;

    // Auf Eingabe warten
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                if kind == KeyEventKind::Release { continue; }
                match code {
                    KeyCode::Char('1') => return Ok(Lang::De),
                    KeyCode::Char('2') => return Ok(Lang::En),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(Lang::De),
                    _ => {}
                }
            }
        }
    }
}