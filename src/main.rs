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

const FRAME: Duration = Duration::from_millis(16);

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, Hide)?;
    let result = run_game();
    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    match result {
        Ok(score) => println!("Spiel beendet! Score: {score}"),
        Err(e)    => eprintln!("Fehler: {e}"),
    }
    Ok(())
}

fn run_game() -> io::Result<u32> {
    let mut game     = Game::new();
    let mut renderer = Renderer::new()?;
    let mut last_drop   = Instant::now();
    let mut last_render = Instant::now();

    renderer.draw_full(&game)?;

    loop {
        // Alle Events auf einmal leeren
        while let Some(ev) = poll_input(Duration::from_millis(0))? {
            match game.handle_event(ev) {
                GameEvent::Quit => return Ok(game.score),
                GameEvent::HardDropped => { last_drop = Instant::now(); }
                GameEvent::Redraw | GameEvent::None => {}
            }
            if game.is_over() { break; }
        }

        if !game.is_over() && !game.paused {
            if last_drop.elapsed() >= game.drop_interval() {
                game.tick();
                last_drop = Instant::now();
            }
        }

        if last_render.elapsed() >= FRAME {
            renderer.draw_full(&game)?;
            last_render = Instant::now();
        }

        if game.is_over() {
            renderer.draw_full(&game)?;
            renderer.draw_game_over(&game)?;
            loop {
                if poll_input(Duration::from_millis(100))?.is_some() { break; }
            }
            break;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(game.score)
}