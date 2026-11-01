use anyhow::Result;
use color_print::cformat;
use rand::{random_bool, random_range};

use crate::{
    Game,
    render::{DynamicRender, Element, Entity, Position},
    signal::{MAX_SPEED_LIMIT, MIN_SPEED_LIMIT, Signal, SpeedSign, SpeedSignType},
    station::MAX_STATION_DIST,
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

fn generate_trees(game: &mut Game) {
    const TREE_DIST: f64 = 100.0;

    let mut tree_pos: f64 = 0.0;
    let station_pos = game.next_station.get_position_meters();

    while tree_pos < station_pos {
        game.trees.add_tree(tree_pos);
        tree_pos += TREE_DIST;
    }
}

#[derive(Debug)]
pub struct Junction {
    pub position: f64,
    pub diverging_speed: Option<u16>,
}

fn generate_junctions(game: &mut Game) {
    let station_pos = game.next_station.get_position_meters();

    let chance = random_range(0.0..station_pos) / MAX_STATION_DIST;

    while random_bool(chance) {
        let position = random_range(0.0..station_pos);
        let diverging_speed = if random_bool(0.75) {
            Some(random_range((MIN_SPEED_LIMIT / 10)..=(MAX_SPEED_LIMIT / 15)) * 10)
        } else {
            None
        };

        game.junctions.push(Junction {
            position,
            diverging_speed,
        });
    }
}

fn generate_signals(game: &mut Game) {
    const SIGNAL_DIST: f64 = 500.0;

    generate_junctions(game);
    game.junctions.sort_by(|j0, j1| {
        let speed0 = j0.diverging_speed.unwrap_or(MAX_SPEED_LIMIT);
        let speed1 = j1.diverging_speed.unwrap_or(MAX_SPEED_LIMIT);
        // we want descending order because the 'first' junction should
        // be further down the track
        speed1.cmp(&speed0)
    });

    for junc in &game.junctions {
        todo!("Implement signal generation using junctions")
    }

    game.signals.clear_all_signals();
}

pub fn generate(game: &mut Game) {
    generate_trees(game);
    generate_signals(game);
}
