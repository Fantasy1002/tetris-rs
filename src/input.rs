use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use crate::game::InputEvent;

pub fn poll_input(timeout: Duration) -> std::io::Result<Option<InputEvent>> {
    if !event::poll(timeout)? { return Ok(None); }
    match event::read()? {
        Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
            // Release-Events ignorieren (wichtig auf Windows!)
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