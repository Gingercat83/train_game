use anyhow::Result;
use crossterm::terminal::enable_raw_mode;
use train_game::{Game, game_loop};

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut game = Game::new();

    game_loop(&mut game)?;

    // clearscreen::clear()?;
    // disable_raw_mode()?;
    // cprintln!("<green,bold>Thank you for playing!</>");

    Ok(())
}
