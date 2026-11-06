use std::cmp::Ordering::Equal;

use anyhow::{Result, bail};
use color_print::cformat;

use crate::render::{DynamicRender, Element, Entity, METERS_PER_CHAR, Position, TRACK_Y};

pub const MAX_SPEED_LIMIT: u16 = 160;
pub const MIN_SPEED_LIMIT: u16 = 40;
pub const MIN_LINE_SPEED: u16 = 80;
pub const MAX_JUNCTION_SPEED: u16 = 120;

pub const CONTROL_POINT_RANGE: f64 = 250.0;
pub const DECELERATION: f64 = 13600.0;

pub fn calc_start_speed(end_speed: f64, distance_meters: f64, deceleration: f64) -> f64 {
    // v^2 = v0^2 + 2a*x
    // v0^2 = v^2 - 2a*x
    let distance_km = distance_meters / 1000.0;
    let accel = -f64::abs(deceleration);
    let start_speed = f64::sqrt(f64::powi(end_speed, 2) - (2.0 * distance_km * accel));
    start_speed
}

pub fn calc_stopping_distance(end_speed: f64, start_speed: f64, deceleration: f64) -> f64 {
    // v^2 = v0^2 + 2a*x
    // v^2 - v0^2 = 2a*x
    // x = (v^2 - v0^2) / 2a
    let accel = -f64::abs(deceleration);
    let distance_km = (f64::powi(end_speed, 2) - f64::powi(start_speed, 2)) / (2.0 * accel);
    let distance_meters = distance_km * 1000.0;
    distance_meters
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpeedControl {
    pub limit: Option<u16>,
    pub reduction: Option<u16>,
    pub is_permanent: bool,
}

pub trait ControlPoint {
    fn speed_control(&self) -> SpeedControl;
    fn position_meters(&self) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeedSignType {
    Limit(u16),
    Reduction(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedSign {
    speed_type: SpeedSignType,
    position_meters: f64,
}

impl DynamicRender for SpeedSign {
    fn render(&self, position_meters: f64) -> Result<()> {
        let sign_text = match self.speed_type {
            SpeedSignType::Limit(lim) => cformat!("<c,bold>[{lim:3}]</>"),
            SpeedSignType::Reduction(red) => cformat!("<y>[<i>{red:3}</>]</>"),
        };

        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![cformat!("{sign_text}"), cformat!("  <black!>|</>")],
                position: Position {
                    x: 0,
                    y: TRACK_Y - 2,
                },
            },
        }
        .render(position_meters)?;

        Ok(())
    }
}

impl ControlPoint for SpeedSign {
    fn speed_control(&self) -> SpeedControl {
        match self.speed_type {
            SpeedSignType::Limit(lim) => SpeedControl {
                limit: Some(lim),
                reduction: None,
                is_permanent: true,
            },
            SpeedSignType::Reduction(red) => SpeedControl {
                limit: None,
                reduction: Some(red),
                is_permanent: true,
            },
        }
    }

    fn position_meters(&self) -> f64 {
        self.position_meters
    }
}

impl SpeedSign {
    pub fn new(speed_type: SpeedSignType, position_meters: f64) -> Self {
        Self {
            speed_type,
            position_meters,
        }
    }

    pub fn reduction_sign(&self, start_speed: u16) -> Option<SpeedSign> {
        let speed_limit = self.speed_control().limit?;

        let dist = calc_stopping_distance(speed_limit as f64, start_speed as f64, DECELERATION);

        let reduction = SpeedSign {
            speed_type: SpeedSignType::Reduction(speed_limit),
            position_meters: self.position_meters - dist,
        };

        Some(reduction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SignalState {
    #[default]
    Red,
    Yellow(SpeedControl),
    Green(SpeedControl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub state: SignalState,
    pub speed_limit: Option<u16>,
    position_meters: f64,
}

impl ControlPoint for Signal {
    fn speed_control(&self) -> SpeedControl {
        use SignalState as S;

        match self.state {
            S::Red => SpeedControl {
                limit: Some(0),
                reduction: None,
                is_permanent: false,
            },
            S::Yellow(SpeedControl {
                mut limit,
                reduction,
                ..
            }) => {
                // signal has a permanent speed limit
                if let Some(perm_limit) = self.speed_limit {
                    if let Some(lim) = limit {
                        limit = Some(perm_limit.min(lim));
                    } else {
                        limit = Some(perm_limit)
                    }
                };

                SpeedControl {
                    limit,
                    reduction,
                    // a yellow signal is never permanent because it could
                    // change to green
                    is_permanent: false,
                }
            }
            S::Green(SpeedControl {
                mut limit,
                reduction,
                ..
            }) => {
                // a green signal is permanent if it has a junction speed limit
                // and its speed limit is not lowered by the next signal.
                let is_permanent = limit.is_some() && limit == self.speed_limit;

                // signal has a permanent speed limit
                if let Some(perm_limit) = self.speed_limit {
                    if let Some(lim) = limit {
                        limit = Some(perm_limit.min(lim));
                    } else {
                        limit = Some(perm_limit)
                    }
                };

                SpeedControl {
                    limit,
                    reduction,
                    is_permanent,
                }
            }
        }
    }

    fn position_meters(&self) -> f64 {
        self.position_meters
    }
}

impl DynamicRender for Signal {
    fn render(&self, position_meters: f64) -> Result<()> {
        use SignalState as S;

        let signal_light = match self.state {
            S::Red => cformat!("<r!,bold>x</>"),
            S::Yellow(_) => cformat!("<y!,bold>o</>"),
            S::Green(_) => cformat!("<g!,bold>*</>"),
        };

        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![
                    cformat!("<black!> _</>"),
                    cformat!("<black!>|</>{}<black!>|</>", signal_light),
                    cformat!("<black!>|_|</>"),
                    cformat!(""),
                    cformat!("<black!> |</>"),
                ],
                position: Position {
                    x: 1,
                    y: TRACK_Y - 5,
                },
            },
        }
        .render(position_meters)?;

        let speed_display = match self.state {
            S::Red => cformat!("<black>___</>"),
            S::Yellow(speed) | S::Green(speed) => {
                let SpeedControl {
                    limit, reduction, ..
                } = speed;
                match (limit, reduction) {
                    (None, None) | (None, Some(0)) => cformat!("<black!>___</>"),
                    (Some(limit), None) => cformat!("<w!,bold>{limit:3}</>"),
                    (None, Some(red)) => cformat!("<y!,i>{red:3}</>"),
                    (Some(limit), Some(red)) if (limit <= red || red == 0) => {
                        cformat!("<w!,bold>{limit:3}</>")
                    }
                    (Some(_), Some(red)) => cformat!("<y!,i>{red:3}</>"),
                }
            }
        };

        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![
                    cformat!("<black!>[</>{}<black!>]</>", speed_display),
                    cformat!("<black!>  |</>"),
                ],
                position: Position {
                    x: 0,
                    y: TRACK_Y - 2,
                },
            },
        }
        .render(position_meters)?;

        Ok(())
    }
}

impl Signal {
    pub fn new(position_meters: f64) -> Self {
        Signal {
            state: SignalState::default(),
            speed_limit: None,
            position_meters,
        }
    }

    pub fn is_clear(&self) -> bool {
        match self.state {
            SignalState::Red => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct Signals {
    signals: Vec<Signal>,
    speed_signs: Vec<SpeedSign>,
}

impl DynamicRender for Signals {
    fn render(&self, position_meters: f64) -> Result<()> {
        for signal in &self.signals {
            signal.render(position_meters)?;
        }
        for sign in &self.speed_signs {
            sign.render(position_meters)?;
        }
        Ok(())
    }
}

impl Signals {
    pub fn new() -> Self {
        Signals {
            signals: Vec::new(),
            speed_signs: Vec::new(),
        }
    }

    pub fn add_signal(&mut self, sig: Signal) {
        self.signals.push(sig);

        // sort signals array by position in descending order
        // signals further down the track (greater position) should appear
        // first in the array
        self.signals.sort_by(|s0, s1| {
            s1.position_meters
                .partial_cmp(&s0.position_meters)
                .unwrap_or(Equal)
        });
    }

    /// Index 0 is the signal furthest down the track.
    pub fn get_signal(&self, index: usize) -> Option<&Signal> {
        self.signals.get(index)
    }

    /// Index 0 is the signal furthest down the track.
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    pub fn set_signal_stop(&mut self, index: usize) -> Result<()> {
        let Some(sig) = self.signals.get_mut(index) else {
            bail!("Signal #{index} does not exist")
        };

        sig.state = SignalState::Red;

        self.update_signals();

        Ok(())
    }

    pub fn set_signal_clear(&mut self, index: usize) -> Result<()> {
        let Some(sig) = self.signals.get_mut(index) else {
            bail!("Signal #{index} does not exist")
        };

        sig.state = SignalState::Green(SpeedControl::default());

        self.update_signals();

        Ok(())
    }

    pub fn set_signal_speed_limit(&mut self, index: usize, speed_limit: Option<u16>) -> Result<()> {
        let Some(sig) = self.signals.get_mut(index) else {
            bail!("Signal #{index} does not exist")
        };

        sig.speed_limit = speed_limit;

        self.update_signals();

        Ok(())
    }

    pub fn clear_all_signals(&mut self) {
        for (i, signal) in self.signals.iter_mut().enumerate() {
            // we sill want the last signal to be red
            if i == 0 {
                signal.state = SignalState::Red;
            } else {
                signal.state = SignalState::Green(SpeedControl::default());
            }
        }

        self.update_signals();
    }

    pub fn update_signals(&mut self) {
        if self.signals.len() <= 1 {
            return;
        }

        // create a working copy to update
        let mut signals_copy = self.signals.clone();

        for i in 1..signals_copy.len() {
            // only update if this signal is clear
            // if a signal is red, updating is skipped
            if !signals_copy[i].is_clear() {
                continue;
            }

            let next_signal = signals_copy[i - 1].clone();
            let this_signal = &mut signals_copy[i];

            let dist = next_signal.position_meters - this_signal.position_meters;

            let line_speed = self.get_line_speed(this_signal.position_meters);
            let next_line_speed = self.get_line_speed(next_signal.position_meters);

            // Update Logic:
            // - if the next signal is red, this signal should be yellow
            // and should have a speed limit that allows train to stop in time
            // - if the next signal is showing a speed limit and there is not
            // enough distance to slow down from the line speed from this signal,
            // then this signal should show a speed limit that allows the train
            // to slow down in time
            // - if the next signal shows a speed limit but there is enough
            // distance to slow down, the signal should show green

            let next_signal_speed = next_signal.speed_control();
            let next_speed_limit = next_signal_speed
                .limit
                .unwrap_or(next_line_speed)
                .min(next_signal.speed_limit.unwrap_or(next_line_speed));

            let this_signal_speed = this_signal.speed_control();
            let this_speed_limit = this_signal_speed
                .limit
                .unwrap_or(line_speed)
                .min(this_signal.speed_limit.unwrap_or(line_speed));

            // find the max speed for this signal
            // if the next signal has a reduced speed limit, calculate the max speed
            // if there is no reduced speed limit at the next signal, do not reduce max speed
            // signals should not show line speed information
            let max_speed = if next_speed_limit < next_line_speed {
                calc_start_speed(next_speed_limit as f64, dist, DECELERATION)
            } else {
                line_speed as f64
            };

            // round to nearest 10
            let max_speed = ((max_speed / 10.0).round() as u16) * 10;

            // the max speed cannot be faster than current signal's speed limit
            let max_speed = max_speed.min(this_speed_limit);

            // check max speed against line speed
            let max_speed_maybe = if max_speed < line_speed {
                Some(max_speed)
            } else {
                None
            };

            // let this_signal = &mut signals_copy[i];

            // next signal is red
            if next_speed_limit == 0 {
                this_signal.state = SignalState::Yellow(SpeedControl {
                    limit: max_speed_maybe,
                    reduction: Some(0),
                    is_permanent: false,
                })
            }
            // next signal's speed limit is too low to slow down at this signal's speed
            else if max_speed < this_speed_limit
                && max_speed != next_speed_limit
                && next_speed_limit < next_line_speed
            {
                this_signal.state = SignalState::Yellow(SpeedControl {
                    limit: Some(max_speed),
                    reduction: Some(next_speed_limit),
                    is_permanent: false,
                })
            }
            // next signal's speed limit is not too low
            else {
                let reduction =
                    if next_speed_limit < this_speed_limit && next_speed_limit < next_line_speed {
                        Some(next_speed_limit)
                    } else {
                        None
                    };
                this_signal.state = SignalState::Green(SpeedControl {
                    limit: max_speed_maybe,
                    reduction,
                    is_permanent: false,
                })
            }
        }

        // merge copy back into original
        self.signals = signals_copy;
    }

    pub fn get_line_speed(&self, position_meters: f64) -> u16 {
        let mut line_speed = MAX_SPEED_LIMIT;

        for sign in &self.speed_signs {
            // get the sign's speed limit and position
            // if the sign is not a speed limit sign, go to the next one
            let Some(limit) = sign.speed_control().limit else {
                continue;
            };
            let sign_pos = sign.position_meters();

            // the line speed is the most recently passed speed limit sign
            if sign_pos < position_meters {
                line_speed = limit;
                break;
            }
        }

        line_speed
    }

    pub fn add_sign(&mut self, sign: SpeedSign) {
        self.speed_signs.push(sign);

        // sort signs array by position in descending order
        // signs further down the track (greater position) should appear
        // first in the array
        self.speed_signs.sort_by(|s0, s1| {
            s1.position_meters
                .partial_cmp(&s0.position_meters)
                .unwrap_or(Equal)
        });
    }

    pub fn get_sign(&self, index: usize) -> Option<&SpeedSign> {
        self.speed_signs.get(index)
    }

    pub fn signs(&self) -> &[SpeedSign] {
        &self.speed_signs
    }

    pub fn nudge_signs(&mut self) {
        const MIN_SEPERATION: f64 = METERS_PER_CHAR * 6.0;

        // create working copy of signs array
        let mut signs_copy = self.speed_signs.clone();

        for sign in &mut signs_copy {

            // check against each sign or signal position
            for signal in &self.signals {
                let seperation = sign.position_meters() - signal.position_meters();

                // check if sign is too close to signal
                if seperation.abs() < MIN_SEPERATION {
                    let direction = seperation.signum();

                    // nudge sign
                    sign.position_meters += MIN_SEPERATION * direction;
                }
            }
        }

        // merge copy back into original
        self.speed_signs = signs_copy;
    }

    pub fn control_points(&self) -> impl Iterator<Item = &dyn ControlPoint> {
        let signals_iter = self.signals.iter().map(|s| s as &dyn ControlPoint);
        let signs_iter = self.speed_signs.iter().map(|s| s as &dyn ControlPoint);
        signals_iter.chain(signs_iter)
    }
}
