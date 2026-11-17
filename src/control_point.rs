use anyhow::Result;
use color_print::cformat;

use crate::{
    render::{DynamicRender, Element, Entity, Position, TRACK_Y},
    signal::{DECELERATION, calc_stopping_distance},
};

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

    pub fn set_position(&mut self, position_meters: f64) {
        self.position_meters = position_meters;
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
