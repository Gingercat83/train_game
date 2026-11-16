use std::fmt::Debug;

use rand::{random_bool, random_range};

use crate::{
    random_speed_limit,
    signal::{MAX_JUNCTION_SPEED, MAX_SPEED_LIMIT, MIN_LINE_SPEED, MIN_SPEED_LIMIT},
    station::{STATION_LENGTH, STATION_SPEED_LIMIT},
};

// pub trait TrackPiece: DynamicRender + Any + Debug {
//     fn length(&self) -> f64;
//     fn speed_limit(&self) -> u16;
//     fn generate(position: f64) -> Self
//     where
//         Self: Sized;
// }

#[derive(Debug, Clone, PartialEq)]
pub enum TrackPieceType {
    Track,
    Junction,
    Station,
}

#[derive(Debug, Clone)]
pub struct TrackPiece {
    pub length: f64,
    pub speed_limit: u16,
    pub track_type: TrackPieceType,
}

impl TrackPiece {
    fn track_generate() -> Self {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 2500.0;

        let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
        let speed_limit: u16 = random_speed_limit(MIN_LINE_SPEED, MAX_SPEED_LIMIT);

        Self {
            length,
            speed_limit,
            track_type: TrackPieceType::Track,
        }
    }

    fn junction_generate() -> Self {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 300.0;

        let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
        let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, MAX_JUNCTION_SPEED);

        Self {
            length,
            speed_limit,
            track_type: TrackPieceType::Junction,
        }
    }

    fn station_generate() -> Self {
        const MIN_LENGTH: f64 = 150.0;
        const MAX_LENGTH: f64 = 300.0;

        let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH) + STATION_LENGTH;
        let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, STATION_SPEED_LIMIT);

        Self {
            length,
            speed_limit,
            track_type: TrackPieceType::Station,
        }
    }

    pub fn generate(track_type: TrackPieceType) -> Self {
        match track_type {
            TrackPieceType::Track => Self::track_generate(),
            TrackPieceType::Junction => Self::junction_generate(),
            TrackPieceType::Station => Self::station_generate(),
        }
    }
}

// impl<'a> Into<&'a dyn TrackPiece> for &'a TrackPieceType {
//     fn into(self) -> &'a dyn TrackPiece {
//         use TrackPieceType as T;
//         match self {
//             T::Track(t) => t as &dyn TrackPiece,
//             T::Junction(t) => t as &dyn TrackPiece,
//             T::Station(t) => t as &dyn TrackPiece,
//         }
//     }
// }

// impl<'a> TryFrom<&'a dyn TrackPiece> for TrackPieceType {
//     type Error = Error;

//     fn try_from(value: &'a dyn TrackPiece) -> Result<Self, Self::Error> {
//         if let Some(track) = (value as &dyn Any).downcast_ref::<Track>() {
//             Ok(TrackPieceType::Track(track.clone()))
//         } else if let Some(track) = (value as &dyn Any).downcast_ref::<JunctionTrack>() {
//             Ok(TrackPieceType::Junction(track.clone()))
//         } else if let Some(track) = (value as &dyn Any).downcast_ref::<StationTrack>() {
//             Ok(TrackPieceType::Station(track.clone()))
//         } else {
//             Err(anyhow!("Cannot convert to TrackPieceType"))
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Track {
//     pub length: f64,
//     pub speed_limit: u16,
// }

// impl DynamicRender for Track {
//     fn render(&self, _position_meters: f64) -> anyhow::Result<()> {
//         // do nothing
//         Ok(())
//     }
// }

// impl TrackPiece for Track {
//     fn length(&self) -> f64 {
//         self.length
//     }
//     fn speed_limit(&self) -> u16 {
//         self.speed_limit
//     }
//     fn generate(_position: f64) -> Self {
//         const MIN_LENGTH: f64 = 150.0;
//         const MAX_LENGTH: f64 = 2500.0;

//         let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
//         let speed_limit: u16 = random_speed_limit(MIN_LINE_SPEED, MAX_SPEED_LIMIT);

//         Self {
//             length,
//             speed_limit,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct JunctionTrack {
//     pub length: f64,
//     pub speed_limit: u16,
// }

// impl DynamicRender for JunctionTrack {
//     fn render(&self, _position_meters: f64) -> anyhow::Result<()> {
//         // do nothing
//         Ok(())
//     }
// }

// impl TrackPiece for JunctionTrack {
//     fn length(&self) -> f64 {
//         self.length
//     }
//     fn speed_limit(&self) -> u16 {
//         self.speed_limit
//     }
//     fn generate(_position: f64) -> Self {
//         const MIN_LENGTH: f64 = 150.0;
//         const MAX_LENGTH: f64 = 300.0;

//         let length: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
//         let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, MAX_JUNCTION_SPEED);

//         Self {
//             length,
//             speed_limit,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct StationTrack {
//     pub length: f64,
//     pub speed_limit: u16,
//     pub station: Station,
// }

// impl DynamicRender for StationTrack {
//     fn render(&self, position_meters: f64) -> anyhow::Result<()> {
//         self.station.render(position_meters)
//     }
// }

// impl TrackPiece for StationTrack {
//     fn length(&self) -> f64 {
//         self.length
//     }
//     fn speed_limit(&self) -> u16 {
//         self.speed_limit
//     }
//     fn generate(position: f64) -> Self
//     where
//         Self: Sized,
//     {
//         const MIN_LENGTH: f64 = 150.0;
//         const MAX_LENGTH: f64 = 300.0;

//         let length_addition: f64 = random_range(MIN_LENGTH..MAX_LENGTH);
//         let speed_limit: u16 = random_speed_limit(MIN_SPEED_LIMIT, STATION_SPEED_LIMIT);

//         Self {
//             length: length_addition + STATION_LENGTH,
//             speed_limit,
//             station: Station::new(position + length_addition),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct TrackWithPosition {
    pub track: TrackPiece,
    pub start_position: f64,
}

pub struct TrackIter<I: Iterator<Item = TrackPiece>> {
    tracks_iter: I,
    current_length: f64,
}

impl<I: Iterator<Item = TrackPiece>> Iterator for TrackIter<I> {
    type Item = TrackWithPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(track) = self.tracks_iter.next() {
            let start_position = self.current_length;
            self.current_length += track.length;

            Some(TrackWithPosition {
                track,
                start_position,
            })
        } else {
            None
        }
    }
}

impl<I: Iterator<Item = TrackPiece>> TrackIter<I> {
    fn new(tracks_iter: I) -> Self {
        Self {
            tracks_iter,
            current_length: 0.0,
        }
    }
}

macro_rules! track_iter {
    () => {
        impl Iterator<Item = TrackWithPosition>
    };
}

#[derive(Debug)]
pub struct TrackPieces(Vec<TrackPiece>);

impl TrackPieces {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_piece(&mut self, piece: TrackPiece) {
        self.0.push(piece);
    }

    pub fn get_piece(&self, index: usize) -> Option<&TrackPiece> {
        self.0.get(index)
    }

    pub fn get_pieces(&self) -> track_iter!() {
        TrackIter::new(self.0.iter().cloned())
    }

    pub fn get_junctions(&self) -> track_iter!() {
        TrackIter::new(self.0.iter().cloned())
            .filter(|t| t.track.track_type == TrackPieceType::Junction)
    }

    pub fn get_station_track(&self) -> Option<TrackWithPosition> {
        let tracks = TrackIter::new(self.0.iter().cloned()).collect::<Box<[_]>>();

        for track_piece in tracks.iter().rev() {
            if track_piece.track.track_type == TrackPieceType::Station {
                return Some(track_piece.clone());
            }
        }
        None
    }

    pub fn get_segment_start(&self, index: usize) -> Option<f64> {
        if index > self.piece_count() {
            None
        } else {
            Some(self.0.iter().take(index).map(|p| p.length).sum())
        }
    }

    pub fn piece_count(&self) -> usize {
        self.0.len()
    }

    pub fn total_length(&self) -> f64 {
        self.0.iter().map(|p| p.length).sum()
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
                self.add_piece(TrackPiece::generate(TrackPieceType::Junction));
                segments_since_prev_junction = 0;
            } else {
                self.add_piece(TrackPiece::generate(TrackPieceType::Track));
                segments_since_prev_junction += 1;
            }
        }

        self.add_piece(TrackPiece::generate(TrackPieceType::Station));
    }
}
