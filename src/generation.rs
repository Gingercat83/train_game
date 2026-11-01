use anyhow::Result;
use color_print::cformat;
use rand::random_range;

use crate::{
    Game, render::{DynamicRender, Element, Entity, Position}, signal::Signal,
};

#[derive(Debug, Clone)]

pub struct Trees {
    pub trees: Vec<Entity>,
}

impl DynamicRender for Trees {
    fn render(&self, position_meters: f64) -> Result<()> {
        for tree in &self.trees {
            tree.render(position_meters)?;
        }
        Ok(())
    }
}

impl Trees {
    pub fn new() -> Self {
        Trees { trees: Vec::new() }
    }

    pub fn add_tree(&mut self, position_meters: f64) {
        self.trees.push(Entity {
            position_meters,
            element: Element {
                lines: vec![
                    cformat!("<g> #@# </>"),
                    cformat!("<g>#@@@#</>"),
                    cformat!("<g> #@# </>"),
                    cformat!("<r>  |  </>"),
                ],
                position: Position {
                    x: random_range(0..6),
                    y: random_range(0..3),
                },
            },
        })
    }
}

pub fn generate_trees(game: &mut Game) {
    const TREE_DIST: f64 = 100.0;

    let mut tree_pos: f64 = 0.0;
    let station_pos = game.next_station.get_position_meters();

    while tree_pos < station_pos {
        game.trees.add_tree(tree_pos);
        tree_pos += TREE_DIST;
    }
}

pub fn generate_signals(game: &mut Game) {
    const SIGNAL_DIST: f64 = 500.0;

    let mut signal_pos = game.next_station.get_position_meters() + 100.0;
    let start_pos = game.train.front_position() + 40.0;

    while signal_pos > start_pos {
        game.signals.add_signal(Signal::new(signal_pos));
        signal_pos -= SIGNAL_DIST;
    }

    game.signals.clear_all_signals();
}

pub fn generate(game: &mut Game) {
    generate_trees(game);
    generate_signals(game);
}