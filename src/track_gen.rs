use std::{any::Any, fmt::Debug};

use rand::{random_bool, random_range};

use crate::{
    random_speed_limit,
    render::DynamicRender,
    signal::{MAX_JUNCTION_SPEED, MAX_SPEED_LIMIT, MIN_LINE_SPEED, MIN_SPEED_LIMIT},
    station::{STATION_LENGTH, STATION_SPEED_LIMIT, Station},
};

pub trait TrackPiece: DynamicRender + Any + Debug {
    fn length(&self) -> f64;
    fn speed_limit(&self) -> u16;
    fn generate(position: f64) -> Self
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Track {
    pub length: f64,
    pub speed_limit: u16,
}

impl DynamicRender for Track {
    fn render(&self, _position_meters: f64) -> anyhow::Result<()> {
        // do nothing
        Ok(())
    }
}

impl TrackPiece for Track {
    fn length(&self) -> f64 {
        self.length
    }
    fn speed_limit(&self) -> u16 {
        self.speed_limit
    }
    fn generate(_position: f64) -> Self {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 2500.0;

        let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
        let speed_limit: u16 = random_speed_limit(MIN_LINE_SPEED, MAX_SPEED_LIMIT);

        Self {
            length,
            speed_limit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JunctionTrack {
    pub length: f64,
    pub speed_limit: u16,
}

impl DynamicRender for JunctionTrack {
    fn render(&self, _position_meters: f64) -> anyhow::Result<()> {
        // do nothing
        Ok(())
    }
}

impl TrackPiece for JunctionTrack {
    fn length(&self) -> f64 {
        self.length
    }
    fn speed_limit(&self) -> u16 {
        self.speed_limit
    }
    fn generate(_position: f64) -> Self {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 300.0;

        let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
        let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, MAX_JUNCTION_SPEED);

        Self {
            length,
            speed_limit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StationTrack {
    pub length: f64,
    pub speed_limit: u16,
    pub station: Station,
}

impl DynamicRender for StationTrack {
    fn render(&self, position_meters: f64) -> anyhow::Result<()> {
        self.station.render(position_meters)
    }
}

impl TrackPiece for StationTrack {
    fn length(&self) -> f64 {
        self.length
    }
    fn speed_limit(&self) -> u16 {
        self.speed_limit
    }
    fn generate(position: f64) -> Self
    where
        Self: Sized,
    {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 300.0;

        let length_addition: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
        let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, STATION_SPEED_LIMIT);

        Self {
            length: length_addition + STATION_LENGTH,
            speed_limit,
            station: Station::new(position + length_addition),
        }
    }
}

#[derive(Debug)]
pub struct TrackPieces(Vec<Box<dyn TrackPiece>>);

impl TrackPieces {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_piece<P: TrackPiece + 'static>(&mut self, piece: P) {
        self.0.push(Box::new(piece));
    }

    pub fn get_piece(&self, index: usize) -> Option<&dyn TrackPiece> {
        self.0.get(index).map(|p| p.as_ref())
    }

    pub fn get_segment_start(&self, index: usize) -> Option<f64> {
        if index > self.piece_count() {
            None
        } else {
            Some(self.0.iter().take(index).map(|p| p.length()).sum())
        }
    }

    pub fn piece_count(&self) -> usize {
        self.0.len()
    }

    pub fn total_length(&self) -> f64 {
        self.0.iter().map(|p| p.length()).sum()
    }

    pub fn generate_tracks(&mut self, min_station_dist: f64, max_station_dist: f64) {
        let current_length = self.total_length();
        let min_station_pos = current_length + min_station_dist;
        let max_station_pos = current_length + max_station_dist;

        let target_length = random_range(min_station_pos..max_station_pos);

        let mut segments_since_prev_junction: usize = 0;

        while self.total_length() < target_length {
            let junction_chance = (0.3 * (segments_since_prev_junction as f64)).clamp(0.0, 1.0);
            let spawn_junction = random_bool(junction_chance);

            if spawn_junction {
                self.add_piece(JunctionTrack::generate(self.total_length()));
                segments_since_prev_junction = 0;
            } else {
                self.add_piece(Track::generate(self.total_length()));
                segments_since_prev_junction += 1;
            }
        }

        self.add_piece(StationTrack::generate(self.total_length()));
    }
}
