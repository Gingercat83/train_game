use std::{io::stdout, thread, time::Duration};

use anyhow::{Result, bail};
use color_print::cprintln;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, KeyCode},
    terminal::enable_raw_mode,
};

use crate::{
    generation::{Junction, Trees, generate},
    render::{DynamicRender, Element, Position, StaticRender, render_static_ui},
    signal::{ControlPoint, Signals},
    station::Station,
    train::Train,
};

pub mod generation;
pub mod render;
pub mod signal;
pub mod station;
pub mod train;

#[cfg(test)]
mod test;

pub const TICKS_PER_SECOND: u64 = 20;
pub const NUM_TRAIN_CARS: u16 = 3;

/// The main game function. Call this to start the game.
pub fn train_game() -> Result<()> {
    enable_raw_mode()?;

    let mut game = Game::new();

    game_loop(&mut game)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    EBrake,
}

#[derive(Debug)]
pub struct Game {
    pub train: Train,
    pub trees: Trees,
    pub next_station: Station,
    pub signals: Signals,
    pub junctions: Vec<Junction>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            train: Train::new(NUM_TRAIN_CARS),
            trees: Trees::new(),
            next_station: Station::next_station(0.0),
            signals: Signals::new(),
            junctions: Vec::new(),
        }
    }
}

pub fn is_overlapping(pos_a: (f64, f64), pos_b: (f64, f64)) -> bool {
    // the objects are overlapping any point in b is between a0 and a1
    (pos_a.0 < pos_b.0 && pos_b.0 < pos_a.1) || (pos_a.0 < pos_b.1 && pos_b.1 < pos_a.1)
    // special case: b contains a
    || (pos_b.0 < pos_a.0 && pos_a.0 < pos_b.1)
}

pub fn game_loop(game: &mut Game) -> Result<()> {
    generate(game);

    loop {
        // start tick timer
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000 / TICKS_PER_SECOND));
        });

        // if input is immediately available, get input
        let input = if event::poll(Duration::from_secs(0))?
            && let Some(key) = event::read()?.as_key_event()
            && key.is_press()
        {
            match key.code {
                // if the escape or space key is pressed, pause the game
                KeyCode::Esc | KeyCode::Char(' ') => {
                    let pause_result = pause_game()?;
                    if pause_result == PauseMenu::Quit {
                        return Ok(());
                    }
                    None
                }

                KeyCode::Up => Some(Input::Up),
                KeyCode::Down => Some(Input::Down),
                KeyCode::Left => Some(Input::Left),
                KeyCode::Right => Some(Input::Right),
                KeyCode::Backspace => Some(Input::EBrake),
                _ => None,
            }
        } else {
            None
        };

        // run game logic
        tick(game, input)?;

        // wait for tick timer
        let result = handle.join();
        if let Err(e) = result {
            bail!("{e:?}");
        }
    }
}

pub fn tick(game: &mut Game, input: Option<Input>) -> Result<()> {
    clearscreen::clear()?;

    // handle input
    if let Some(inp) = input {
        game.train.handle_input(inp);
    }

    // update game
    game.train.tick();

    let train_pos = game.train.get_position_meters();

    // render ui

    render_static_ui()?;
    game.trees.render(train_pos)?;
    game.next_station.render(train_pos)?;
    game.train.render()?;
    game.signals.render(train_pos)?;
    game.train.render_ui()?;

    // FIXME: temporary station position
    Element {
        lines: vec![
            format!(
                "Next station: {:.0}m",
                game.next_station.get_position_meters() - game.train.front_position()
            ),
            format!("current position: {}", game.train.get_position_meters()),
            format!(
                "speed limit: {}",
                game.signals.get_line_speed(game.train.front_position())
            ),
        ],
        position: Position { x: 30, y: 10 },
    }
    .render()?;

    // move cursor to corner
    stdout().execute(cursor::MoveTo(0, 0))?;

    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum PauseMenu {
    Resume,
    Quit,
}

pub fn pause_game() -> Result<PauseMenu> {
    stdout().execute(cursor::MoveTo(0, 0))?;
    cprintln!("<bold>--- Game Paused ---</>     ");
    cprintln!("<g,bold>[Space]</> - Resume      ");
    cprintln!("<r,bold>[Esc]</>   - Quit        ");

    loop {
        if event::poll(Duration::from_secs(0))?
            && let Some(key) = event::read()?.as_key_event()
            && key.is_press()
        {
            match key.code {
                // space to resume
                KeyCode::Char(' ') => return Ok(PauseMenu::Resume),
                // escape to quit
                KeyCode::Esc => return Ok(PauseMenu::Quit),
                _ => (),
            }
        }

        // do some sleep so we don't hog all the cpu while waiting
        thread::sleep(Duration::from_millis(100));
    }
}
