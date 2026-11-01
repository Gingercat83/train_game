use std::{io::stdout, thread, time::Duration};

use anyhow::{Result, bail};
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, KeyCode},
};

use crate::{
    generation::{Trees, generate, generate_trees}, render::{DynamicRender, Element, Position, StaticRender, render_static_ui}, signal::Signals, station::Station, train::Train,
};

pub mod generation;
pub mod render;
pub mod signal;
pub mod station;
pub mod train;

#[cfg(test)]
mod test;

pub const TICKS_PER_SECOND: u64 = 20;

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
}

impl Game {
    pub fn new(num_cars: u16) -> Self {
        Game {
            train: Train::new(num_cars),
            trees: Trees::new(),
            next_station: Station::next_station(0.0),
            signals: Signals::new(),
        }
    }
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
                // if the escape key is pressed, exit the program
                KeyCode::Esc => return Ok(()),
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

    // FIXME: temporary signal position
    Element {
        lines: vec![format!(
            "Next station: {:.0}m",
            game.next_station.get_position_meters() - game.train.front_position()
        )],
        position: Position { x: 30, y: 10 },
    }
    .render()?;

    // move cursor to corner
    stdout().execute(cursor::MoveTo(0, 0))?;

    Ok(())
}
