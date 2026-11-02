use std::{cmp::Ordering::Equal};

use anyhow::Result;
use color_print::cformat;
use rand::{random_bool, random_range};

use crate::{
    Game, is_overlapping,
    render::{DynamicRender, Element, Entity, Position},
    signal::{MAX_SPEED_LIMIT, MIN_SPEED_LIMIT, Signal},
};

const SIGNAL_SEPERATION: f64 = 500.0;
const JUNCTION_SEPERATION: f64 = 2500.0;
const TREE_DIST: f64 = 100.0;

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
    pub length: f64,
    pub diverging_speed: Option<u16>,
}

fn generate_junction_speed() -> Option<u16> {
    if random_bool(0.75) {
        Some(random_range((MIN_SPEED_LIMIT / 10)..=(MAX_SPEED_LIMIT / 15)) * 10)
    } else {
        None
    }
}

#[test]
fn junc_speed() {
    for _ in 0..10 {
        let s = generate_junction_speed();
        println!("{s:?}");
    }
}

fn generate_junctions(game: &mut Game) {
    let station_pos = game.next_station.get_position_meters();

    let num_junctions = (station_pos / JUNCTION_SEPERATION) as usize ;

    // generate a junction right at the start
    game.junctions.push(Junction {
        position: game.train.front_position() + 80.0,
        length: random_range(150.0..300.0),
        diverging_speed: generate_junction_speed(),
    });

    // generate a junction at the station
    game.junctions.push(Junction {
        position: game.next_station.get_position_meters() - 80.0,
        length: 1000.0,
        diverging_speed: Some(60),
    });

    'gen_junction: for _ in 0..num_junctions {
        let position = random_range(0.0..station_pos);
        let length = random_range(150.0..300.0);

        // make sure junctions are not too close
        for i in 0..game.junctions.len() {
            // if junctions would be overlapping, skip this one
            let junc = &game.junctions[i];
            if is_overlapping(
                (position, position + length),
                (junc.position, junc.position + junc.length),
            ) {
                continue 'gen_junction;
            }
        }

        let diverging_speed = generate_junction_speed();

        game.junctions.push(Junction {
            position,
            length,
            diverging_speed,
        });
    }

    game.junctions
        .sort_by(|s0, s1| s1.position.partial_cmp(&s0.position).unwrap_or(Equal));
}

fn generate_signals(game: &mut Game) {
    generate_junctions(game);

    // generate signal after station
    game.signals.add_signal(Signal::new(game.next_station.get_position_meters() + 160.0));

    
    for i in 0..game.junctions.len() {
        // generate a signal at each junction
        let junc = &game.junctions[i];
        let mut signal = Signal::new(junc.position);
        signal.speed_limit = junc.diverging_speed;
        game.signals.add_signal(signal);

        // get the previous junction
        let Some(prev_junc) = &game.junctions.get(i + 1) else {
            continue;
        };

        // generate intermediate signals between this and previous junction
        let mut signal_pos = prev_junc.position;

        let dist = prev_junc.position - junc.position;
        let num_signals = (dist / SIGNAL_SEPERATION).round();
        let sep = dist / num_signals;

        while signal_pos < junc.position {
            game.signals.add_signal(Signal::new(signal_pos));
            signal_pos += sep;
        }
    }

    game.signals.clear_all_signals();
}

pub fn generate(game: &mut Game) {
    generate_trees(game);
    generate_signals(game);
}
