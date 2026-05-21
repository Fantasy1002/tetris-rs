mod game;
mod render;
mod input;

use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use game::{Game, GameEvent};
use render::Renderer;
use input::poll_input;

const TICK: Duration = Duration::from_millis(16);

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, Hide)?;

    let result = run_game();

    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    match result {
        Ok(score) => println!("Spiel beendet! Dein Score: {score}"),
        Err(e)    => eprintln!("Fehler: {e}"),
    }
    Ok(())
}

fn run_game() -> io::Result<u32> {
    let mut game = Game::new();
    let mut renderer = Renderer::new()?;
    let mut last_drop = Instant::now();
    let mut last_render = Instant::now();

    renderer.draw_full(&game)?;

    loop {
        if let Some(event) = poll_input(Duration::from_millis(1))? {
            match game.handle_event(event) {
                GameEvent::Quit => break,
                GameEvent::Redraw | GameEvent::None => {}
            }
        }

        let drop_interval = game.drop_interval();
        if last_drop.elapsed() >= drop_interval {
            game.tick();
            last_drop = Instant::now();
        }

        if last_render.elapsed() >= TICK {
            renderer.draw_full(&game)?;
            last_render = Instant::now();
        }

        if game.is_over() {
            renderer.draw_game_over(&game)?;
            loop {
                if poll_input(Duration::from_millis(100))?.is_some() {
                    break;
                }
            }
            break;
        }
    }

    Ok(game.score())
}