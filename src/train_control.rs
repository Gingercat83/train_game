use std::fmt::Debug;

use crate::{
    control_point::{ControlPoint, Signal, SpeedControl, SpeedSign},
    signal::{MAX_SPEED_LIMIT, Signals},
};

pub const PENALTY_BRAKE_THRESHOLD: i16 = 5;
pub const EMERGENCY_BRAKE_THRESHOLD: i16 = 20;

pub struct SpeedEnforcement {
    pub limit: u16,
    pub reduction: u16,
    pub stop_close: bool,
}

pub trait TrainControl: Debug {
    fn update(&mut self, signals: &Signals, train_position: f64);
    fn get_speed_enforcement(&self) -> SpeedEnforcement;
}

#[derive(Debug)]
pub struct DebugTrainControl;

impl TrainControl for DebugTrainControl {
    fn update(&mut self, _signals: &Signals, _train_position: f64) {}
    fn get_speed_enforcement(&self) -> SpeedEnforcement {
        SpeedEnforcement {
            limit: MAX_SPEED_LIMIT,
            reduction: MAX_SPEED_LIMIT,
            stop_close: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Speeds {
    speed_limit: u16,
    next_speed_limit: u16,
    speed_reduction: u16,
}

impl Speeds {
    fn get_speed_reduction(&self) -> u16 {
        let next_speed_limit = self.next_speed_limit;
        let speed_reduction = self.speed_reduction;
        // get the lowest between the two
        u16::min(next_speed_limit, speed_reduction)
    }
}

#[derive(Debug)]
pub struct StandardTrainControl {
    sign_speed: Speeds,
    signal_speed: Speeds,
}

impl TrainControl for StandardTrainControl {
    fn update(&mut self, signals: &Signals, train_position: f64) {
        self.update_signals(signals.signals(), train_position);
        self.update_speed_signs(signals.signs(), train_position);
    }

    fn get_speed_enforcement(&self) -> SpeedEnforcement {
        let mut limit = MAX_SPEED_LIMIT;
        let mut reduction = MAX_SPEED_LIMIT;
        let mut stop_close = false;

        for speed in [self.sign_speed, self.signal_speed] {
            limit = limit.min(speed.speed_limit);

            reduction = reduction.min(limit).min(speed.get_speed_reduction());

            // check if there is a stop close
            if speed.next_speed_limit == 0 {
                stop_close = true;
            }
        }

        SpeedEnforcement {
            limit,
            reduction,
            stop_close,
        }
    }
}

impl StandardTrainControl {
    const FAR_DETECTION_RANGE: f64 = 250.0;
    const CLOSE_DETECTION_RANGE: f64 = 25.0;

    pub fn new() -> Self {
        let default_spd = Speeds {
            speed_limit: MAX_SPEED_LIMIT,
            next_speed_limit: MAX_SPEED_LIMIT,
            speed_reduction: MAX_SPEED_LIMIT,
        };
        StandardTrainControl {
            sign_speed: default_spd,
            signal_speed: default_spd,
        }
    }

    fn update_signals(&mut self, signals: &[Signal], train_position: f64) {
        let (this_signal, next_signal) = self.detect_control_points(signals, train_position);

        // there is a signal ahead
        if let Some(next_signal) = next_signal {
            let next_speed = next_signal.speed_control().limit;

            if let Some(next_limit) = next_speed {
                self.signal_speed.next_speed_limit = next_limit;
                self.signal_speed.speed_reduction = next_limit;
            } else {
                self.signal_speed.next_speed_limit = MAX_SPEED_LIMIT;
                self.signal_speed.speed_reduction = MAX_SPEED_LIMIT;
            }
        }
        // no signal ahead
        else {
            self.signal_speed.next_speed_limit = MAX_SPEED_LIMIT;
        }

        // the train just passed a signal
        if let Some(this_signal) = this_signal {
            let SpeedControl {
                limit, reduction, ..
            } = this_signal.speed_control();

            if let Some(limit) = limit {
                self.signal_speed.speed_limit = limit;
            } else {
                self.signal_speed.speed_limit = MAX_SPEED_LIMIT;
            }

            if let Some(reduction) = reduction {
                self.signal_speed.speed_reduction = reduction;
            } else {
                self.signal_speed.speed_reduction = MAX_SPEED_LIMIT;
            }
        }
    }

    fn update_speed_signs(&mut self, signs: &[SpeedSign], train_position: f64) {
        let (this_sign, next_sign) = self.detect_control_points(signs, train_position);

        if let Some(next_sign) = next_sign {
            if let Some(next_speed) = next_sign.speed_control().limit {
                self.sign_speed.next_speed_limit = next_speed;
            } else {
                self.sign_speed.next_speed_limit = MAX_SPEED_LIMIT;
            }
        }

        if let Some(this_sign) = this_sign {
            let SpeedControl {
                limit, reduction, ..
            } = this_sign.speed_control();

            if let Some(limit) = limit {
                self.sign_speed.speed_limit = limit;

                // reset the signal speed limit
                self.signal_speed.speed_limit = MAX_SPEED_LIMIT;

                // if the limit matches the previous reduction, clear the reduction
                if limit == self.sign_speed.speed_reduction {
                    self.sign_speed.speed_reduction = MAX_SPEED_LIMIT;
                }
            }
            if let Some(reduction) = reduction {
                if reduction < self.sign_speed.speed_reduction {
                    self.sign_speed.speed_reduction = reduction;
                }
            }
        }
    }

    fn detect_control_points<'a, T: ControlPoint>(
        &self,
        control_points: &'a [T],
        train_position: f64,
    ) -> (Option<&'a T>, Option<&'a T>) {
        let detection_range_end = train_position + Self::FAR_DETECTION_RANGE;
        let detection_range_start = train_position - Self::CLOSE_DETECTION_RANGE;

        let mut this_signal: Option<&T> = None;
        let mut next_signal: Option<&T> = None;

        for signal in control_points {
            let signal_pos = signal.position_meters();

            // check if it is in detection range
            if signal_pos > detection_range_end || signal_pos < detection_range_start {
                continue;
            }

            // check if it is in close or far range
            if signal_pos > train_position {
                // far range
                next_signal = Some(signal);
            } else {
                // close range
                this_signal = Some(signal);
            }
        }

        (this_signal, next_signal)
    }
}
