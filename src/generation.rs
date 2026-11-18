use anyhow::Result;
use color_print::cformat;
use rand::random_range;

use crate::{
    Game,
    render::{DynamicRender, Element, Entity, Position},
    control_point::{Signal, SpeedSign, SpeedSignType},
    station::{MAX_STATION_DIST, MIN_STATION_DIST, STATION_LENGTH, Station},
    track_gen::TrackPieceType,
};

const SIGNAL_SEPERATION: f64 = 500.0;
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

fn generate_tracks(game: &mut Game) {
    game.tracks
        .generate_tracks(MIN_STATION_DIST, MAX_STATION_DIST);
}

fn generate_station(game: &mut Game) {
    let station_track = game.tracks.get_station_track().unwrap();
    let station_position =
        (station_track.start_position + station_track.track.length) - (STATION_LENGTH * 2.0);

    game.next_station = Station::new(station_position);
}

fn generate_signals(game: &mut Game) {
    // generate signal after station
    game.signals
        .add_signal(Signal::new(game.next_station.get_position_meters() + 160.0));

    let junctions: Box<[_]> = game.tracks.get_junctions().collect();

    for i in 0..junctions.len() {
        let junc = &junctions[i];

        // generate a signal at each junction
        let mut signal = Signal::new(junc.start_position);
        signal.speed_limit = Some(junc.track.speed_limit);
        game.signals.add_signal(signal);

        let prev_junc = if i > 0 { &junctions[i - 1] } else { continue };

        // generate intermediate signals between this and previous junction
        let dist = junc.start_position - prev_junc.start_position;

        // number of signals between junctions should be at least one
        let mut num_signals = (dist / SIGNAL_SEPERATION).abs().round() as usize;
        if num_signals == 0 {
            num_signals = 1;
        }

        let sep = dist / ((num_signals + 1) as f64);

        let mut signal_pos = junc.start_position;

        for _ in 0..num_signals {
            signal_pos -= sep;
            println!("  Adding intermediate signal at {signal_pos}");
            game.signals.add_signal(Signal::new(signal_pos));
        }
    }

    game.signals.clear_all_signals();
    game.signals.set_random_stop_signals();
}

pub fn generate_speed_limits(game: &mut Game) {
    let track_segments: Box<[_]> = game.tracks.get_pieces().collect();

    for i in 1..track_segments.len() {
        let this_piece = &track_segments[i];
        let prev_piece = &track_segments[i - 1];

        // only put speed limit signs at the start of normal track sections
        // junction and station speed limits are shown by the signals
        if this_piece.track.track_type == TrackPieceType::Track {
            let speed_limit = this_piece.track.speed_limit;

            let limit_sign =
                SpeedSign::new(SpeedSignType::Limit(speed_limit), this_piece.start_position);

            let prev_speed_limit = prev_piece.track.speed_limit;
            // check if we need a reduction sign
            if prev_speed_limit > speed_limit
                && let Some(reduction_sign) = limit_sign.reduction_sign(prev_speed_limit)
            {
                game.signals.add_sign(reduction_sign);
            }

            game.signals.add_sign(limit_sign);
        }
    }

    game.signals.nudge_signs();
    game.signals.update_signals();
}

pub fn generate(game: &mut Game) {
    generate_tracks(game);
    generate_station(game);
    generate_signals(game);
    generate_speed_limits(game);
    generate_trees(game);
}
