use anyhow::{Result, bail};
use color_print::cformat;

use crate::render::{DynamicRender, Element, Entity, Position, TRACK_Y};

pub const MAX_SPEED_LIMIT: u16 = 200;
pub const MIN_SPEED_LIMIT: u16 = 40;
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
                lines: vec![cformat!("{sign_text}"), cformat!("  <black>|</>")],
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
            S::Yellow(speed) | S::Green(speed) => speed,
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
            S::Red => cformat!("<r,bold>x</>"),
            S::Yellow(_) => cformat!("<y,bold>o</>"),
            S::Green(_) => cformat!("<g,bold>*</>"),
        };

        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![
                    cformat!("<black> _</>"),
                    cformat!("<black>|</>{}<black>|</>", signal_light),
                    cformat!("<black>|_|</>"),
                    cformat!(""),
                    cformat!("<black> |</>"),
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
                let SpeedControl { limit, reduction, ..} = speed;
                match (limit, reduction) {
                    (None, None) | (None, Some(0)) => cformat!("<black>___</>"),
                    (Some(limit), None) => cformat!("<w,bold>{limit:3}</>"),
                    (None, Some(red)) => cformat!("<y,i>{red:3}</>"),
                    (Some(limit), Some(red)) if (limit <= red || red == 0) => {
                        cformat!("<w,bold>{limit:3}</>")
                    }
                    (Some(_), Some(red)) => cformat!("<y,i>{red:3}</>"),
                }
            }
        };

        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![
                    cformat!("<black>[</>{}<black>]</>", speed_display),
                    cformat!("<black>  |</>"),
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
        self.signals.push(sig)
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
        if self.signals.len() == 0 {
            return;
        }

        let mut next_signal = Signal::new(f64::MAX);

        for i in 0..self.signals.len() {
            let dist = next_signal.position_meters - self.signals[i].position_meters;
            let line_speed = self.get_line_speed(self.signals[i].position_meters);

            // only update if this signal is clear
            // if a signal is red, updating is skipped
            if self.signals[i].is_clear() {
                let this_signal = &mut self.signals[i];

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
                let next_speed_limit = next_signal_speed.limit.unwrap_or(line_speed);

                let this_signal_speed = this_signal.speed_control();
                let this_speed_limit = this_signal_speed
                    .limit
                    .unwrap_or(line_speed)
                    .min(this_signal.speed_limit.unwrap_or(line_speed));

                let max_speed = calc_start_speed(next_speed_limit as f64, dist, DECELERATION);

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

                // next signal is red
                if next_speed_limit == 0 {
                    this_signal.state = SignalState::Yellow(SpeedControl {
                        limit: max_speed_maybe,
                        reduction: Some(0),
                        is_permanent: false,
                    })
                }
                // next signal's speed limit is too low to slow down at this signal's speed
                else if max_speed < this_speed_limit && max_speed != next_speed_limit {
                    this_signal.state = SignalState::Yellow(SpeedControl {
                        limit: Some(max_speed),
                        reduction: Some(next_speed_limit),
                        is_permanent: false,
                    })
                }
                // next signal's speed limit is not too low
                else {
                    let reduction = if next_speed_limit < this_speed_limit {
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

            next_signal = self.signals[i].clone();
        }
    }

    pub fn get_line_speed(&self, _position_meters: f64) -> u16 {
        // 60 
        MAX_SPEED_LIMIT
        // todo!()
    }

    pub fn add_sign(&mut self, sign: SpeedSign) {
        self.speed_signs.push(sign)
    }

    pub fn get_sign(&self, index: usize) -> Option<&SpeedSign> {
        self.speed_signs.get(index)
    }

    pub fn signs(&self) -> &[SpeedSign] {
        &self.speed_signs
    }

    pub fn control_points(&self) -> impl Iterator<Item = &dyn ControlPoint> {
        let signals_iter = self.signals.iter().map(|s| s as &dyn ControlPoint);
        let signs_iter = self.speed_signs.iter().map(|s| s as &dyn ControlPoint);
        signals_iter.chain(signs_iter)
    }
}
