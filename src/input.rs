use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crate::game::InputEvent;

pub fn poll_input(timeout: Duration) -> crossterm::Result<Option<InputEvent>> {
    if !event::poll(timeout)? { return Ok(None); }
    match event::read()? {
        Event::Key(KeyEvent { code, modifiers, .. }) => {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return Ok(Some(InputEvent::Quit));
            }
            let ev = match code {
                KeyCode::Left  | KeyCode::Char('a') => InputEvent::Left,
                KeyCode::Right | KeyCode::Char('d') => InputEvent::Right,
                KeyCode::Up    | KeyCode::Char('w') => InputEvent::RotateCW,
                KeyCode::Down  | KeyCode::Char('s') => InputEvent::SoftDrop,
                KeyCode::Char(' ')                  => InputEvent::HardDrop,
                KeyCode::Char('c')                  => InputEvent::Hold,
                KeyCode::Char('p')                  => InputEvent::Pause,
                KeyCode::Char('q') | KeyCode::Esc   => InputEvent::Quit,
                _ => return Ok(None),
            };
            Ok(Some(ev))
        }
        _ => Ok(None),
    }
}