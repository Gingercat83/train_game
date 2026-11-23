use anyhow::{Result, bail};
use color_print::{cformat, cprintln};
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, KeyCode},
    terminal::enable_raw_mode,
};
use rand::random_range;
use std::{
    io::stdout,
    thread::{self, sleep},
    time::Duration,
};

use crate::{
    generation::{Trees, generate},
    render::{DynamicRender, StaticRender, render_static_ui},
    signal::Signals,
    station::Station,
    track_gen::TrackPieces,
    train::Train,
};

pub mod control_point;
pub mod generation;
pub mod render;
pub mod signal;
pub mod station;
pub mod track_gen;
pub mod train;
pub mod train_control;

#[cfg(test)]
mod test;

pub const TICKS_PER_SECOND: u64 = 20;
pub const NUM_TRAIN_CARS: u16 = 3;

/// The main game function. Call this to start the game.
pub fn train_game() -> Result<()> {
    enable_raw_mode()?;

    let mut game = Game::new();

    let quit_message = game_loop(&mut game)?;

    clearscreen::clear()?;
    println!("{quit_message}");
    sleep(Duration::from_secs(2));

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
    pub tracks: TrackPieces,
    pub tick_counter: u64,
}

impl Game {
    pub fn new() -> Self {
        Game {
            train: Train::new(NUM_TRAIN_CARS),
            trees: Trees::new(),
            next_station: Station::new(0.0),
            signals: Signals::new(),
            tracks: TrackPieces::new(),
            tick_counter: 0,
        }
    }

    pub fn update(&mut self) -> Action {
        self.signals
            .update_stop_signals(self.train.front_position(), self.tick_counter);
        self.tick_counter += 1;

        self.train.tick(&self.signals)
    }
}

impl StaticRender for Game {
    fn render(&self) -> Result<()> {
        let train_pos = self.train.get_position_meters();

        clearscreen::clear()?;

        render_static_ui()?;
        self.trees.render(train_pos)?;
        self.next_station.render(train_pos)?;
        self.train.render()?;
        self.signals.render(train_pos)?;
        self.train.render_ui()?;

        // move cursor to corner
        stdout().execute(cursor::MoveTo(0, 0))?;

        Ok(())
    }
}

pub fn is_overlapping(pos_a: (f64, f64), pos_b: (f64, f64)) -> bool {
    // the objects are overlapping any point in b is between a0 and a1
    (pos_a.0 < pos_b.0 && pos_b.0 < pos_a.1) || (pos_a.0 < pos_b.1 && pos_b.1 < pos_a.1)
    // special case: b contains a
    || (pos_b.0 < pos_a.0 && pos_a.0 < pos_b.1)
}

pub fn random_speed_limit(min: u16, max: u16) -> u16 {
    random_range((min / 10)..=(max / 10)) * 10
}

pub fn game_loop(game: &mut Game) -> Result<String> {
    generate(game);

    loop {
        // start tick timer
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000 / TICKS_PER_SECOND));
        });

        // get input
        let input = match get_input()? {
            Action::Continue => None,
            Action::Input(input) => Some(input),
            Action::Quit(message) => return Ok(message),
        };

        // run game logic
        match tick(game, input)? {
            Action::Quit(message) => return Ok(message),
            _ => ()
        }

        // wait for tick timer
        let result = handle.join();
        if let Err(e) = result {
            bail!("{e:?}");
        }
    }
}

pub fn tick(game: &mut Game, input: Option<Input>) -> Result<Action> {
    // handle input
    if let Some(inp) = input {
        game.train.handle_input(inp);
    }

    // update game
    let result = game.update();

    // render game
    game.render()?;

    // FIXME: temporary station position
    // Element {
    //     lines: vec![
    //         format!(
    //             "Next station: {:.0}m",
    //             game.next_station.get_position_meters() - game.train.front_position()
    //         ),
    //         format!("current position: {}", game.train.get_position_meters()),
    //         format!(
    //             "speed limit: {}",
    //             game.signals.get_line_speed(game.train.front_position())
    //         ),
    //         format!("ticks: {}", game.tick_counter), // format!("signals: {:?}", game.signals),
    //                                                  // format!("tracks: {:?}", game.tracks),
    //     ],
    //     position: Position { x: 30, y: 10 },
    // }
    // .render()?;

    // move cursor to corner
    stdout().execute(cursor::MoveTo(0, 0))?;

    Ok(result)
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Continue,
    Input(Input),
    Quit(String),
}

pub fn get_input() -> Result<Action> {
    // if input is immediately available, get input
    if event::poll(Duration::from_secs(0))?
        && let Some(key) = event::read()?.as_key_event()
        && key.is_press()
    {
        match key.code {
            // if the escape or space key is pressed, pause the game
            KeyCode::Esc | KeyCode::Char(' ') => pause_game(),

            KeyCode::Up => Ok(Action::Input(Input::Up)),
            KeyCode::Down => Ok(Action::Input(Input::Down)),
            KeyCode::Left => Ok(Action::Input(Input::Left)),
            KeyCode::Right => Ok(Action::Input(Input::Right)),
            KeyCode::Backspace => Ok(Action::Input(Input::EBrake)),
            _ => Ok(Action::Continue),
        }
    } else {
        Ok(Action::Continue)
    }
}

pub fn pause_game() -> Result<Action> {
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
                KeyCode::Char(' ') => return Ok(Action::Continue),
                // escape to quit
                KeyCode::Esc => {
                    return Ok(Action::Quit(cformat!(
                        "<green,bold>Thank you for playing!</>"
                    )));
                }
                _ => (),
            }
        }

        // do some sleep so we don't hog all the cpu while waiting
        thread::sleep(Duration::from_millis(100));
    }
}
