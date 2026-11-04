use anyhow::Result;
use color_print::cformat;

use crate::{
    Input, TICKS_PER_SECOND,
    render::{Element, METERS_PER_CHAR, Position, StaticRender, TRACK_Y},
};

#[derive(Debug, Clone, Copy, PartialEq)]

pub enum Throttle {
    Power(u8),
    Neutral,
    Brake(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brake {
    Release,
    Hold,
    Apply,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct Train {
    num_cars: u16,
    position_meters: f64,
    speed_kph: f64,
    throttle: Throttle,
    brake_setting: Brake,
    brake_pipe_pressure: f64,
    equalize_pressure: f64,
}

impl StaticRender for Train {
    fn render(&self) -> Result<()> {
        let mut lines = vec![String::new(); 2];

        // add each car
        for _ in 0..self.num_cars {
            lines[0] += &cformat!("<bg:b>       </> ");
            lines[1] += &cformat!("<black!>oo---oo</> ");
        }

        // add locomotive
        lines[0] += &cformat!("<bg:b>        =</>");
        lines[1] += &cformat!("<black!>oo---oo</><bg:b>  *</>");

        Element {
            lines,
            position: Position {
                x: Self::LEFT_OFFSET,
                y: TRACK_Y - 2,
            },
        }
        .render()?;
        Ok(())
    }
}

impl Train {
    const CAR_LENGTH: u16 = 8;
    const LOCO_LENGTH: u16 = 10;
    const LEFT_OFFSET: u16 = 5;
    const CAR_WEIGHT: f64 = 1000.0;
    const EMERGENCY_PRESSURE: f64 = 1.0;
    const RELEASED_PRESSURE: f64 = 5.0;

    pub fn new(num_cars: u16) -> Self {
        let car_lengths = num_cars * Self::CAR_LENGTH;
        let total_length = car_lengths + Self::LOCO_LENGTH;
        let position = total_length + Self::LEFT_OFFSET;
        let position_meters = position as f64 / METERS_PER_CHAR;
        let brake_pipe_pressure = (Self::EMERGENCY_PRESSURE + Self::RELEASED_PRESSURE) / 2.0;
        Train {
            num_cars,
            speed_kph: 0.0,
            position_meters,
            throttle: Throttle::Neutral,
            brake_setting: Brake::Hold,
            brake_pipe_pressure,
            equalize_pressure: brake_pipe_pressure,
        }
    }

    pub fn get_speed(&self) -> f64 {
        self.speed_kph
    }

    pub fn get_position_meters(&self) -> f64 {
        self.position_meters
    }

    pub fn rear_position(&self) -> f64 {
        self.position_meters + (Self::LEFT_OFFSET as f64 * METERS_PER_CHAR)
    }

    pub fn front_position(&self) -> f64 {
        self.rear_position() + self.get_length_meters()
    }

    pub fn get_length_meters(&self) -> f64 {
        let car_lengths = self.num_cars * Self::CAR_LENGTH;
        let total_length = car_lengths + Self::LOCO_LENGTH;
        let length_meters = total_length as f64 * METERS_PER_CHAR;
        length_meters
    }

    pub fn get_weight(&self) -> f64 {
        Self::CAR_WEIGHT * ((self.num_cars + 1) as f64)
    }

    pub fn drag_force(&self) -> f64 {
        -0.7 * self.speed_kph
    }

    pub fn throttle_force(&self) -> f64 {
        match self.throttle {
            // power levels off at higher speeds
            Throttle::Power(p) => p as f64 * 200.0 * f64::exp(-self.speed_kph / 160.0),
            Throttle::Neutral => 0.0,
            // dynamic brakes are not effective at low speed
            Throttle::Brake(b) => b as f64 * -35.0 * f64::ln((self.speed_kph / 30.0) + 1.0),
        }
    }

    pub fn brake_application(&self) -> f64 {
        // normalize pressures so they start at zero
        let normalized_pressure = self.brake_pipe_pressure - Self::EMERGENCY_PRESSURE;
        let normalized_released = Self::RELEASED_PRESSURE - Self::EMERGENCY_PRESSURE;
        // get application ratio
        let application = 1.0 - (normalized_pressure / normalized_released);
        // ensure ratio is between 0 and 1
        if application > 1.0 {
            1.0
        } else if application < 0.0 {
            0.0
        } else {
            application
        }
    }

    pub fn brake_force(&self) -> f64 {
        // each car has brakes, so force is proportional to number of cars
        -500.0 * (self.num_cars as f64) * self.brake_application()
    }

    pub fn update_brake_pressure(&mut self) {
        let difference = self.equalize_pressure - (self.brake_pipe_pressure);
        self.brake_pipe_pressure += difference * 0.02;
        self.equalize_pressure -= difference * 0.02 * (self.num_cars as f64);

        // clamp pressure
        if self.equalize_pressure > Self::RELEASED_PRESSURE {
            self.equalize_pressure = Self::RELEASED_PRESSURE;
        }
        if self.equalize_pressure < 0.0 {
            self.equalize_pressure = 0.0;
        }

        if self.brake_pipe_pressure > Self::RELEASED_PRESSURE {
            self.brake_pipe_pressure = Self::RELEASED_PRESSURE;
        }
        if self.brake_pipe_pressure < 0.0 {
            self.brake_pipe_pressure = 0.0;
        }
    }

    pub fn tick(&mut self) {
        // update brake pressure
        match self.brake_setting {
            Brake::Release => self.equalize_pressure += 0.1,
            Brake::Hold => (),
            Brake::Apply => self.equalize_pressure -= 0.1,
            Brake::Emergency => self.equalize_pressure = 0.0,
        };
        self.update_brake_pressure();

        // if brakes are applied, throttle cannot be applied
        if self.brake_application() > 0.05
            && let Throttle::Power(_) = self.throttle
        {
            self.throttle = Throttle::Neutral;
        }

        // calculate force
        let force = self.drag_force() + self.throttle_force() + self.brake_force();
        // f = ma   a = f / m
        let acceleration = force / self.get_weight();
        self.speed_kph += acceleration;

        // ensure speed is not negative (backwards is not allowed)
        if self.speed_kph < 0.0 {
            self.speed_kph = 0.0;
        }

        // update position
        // 1 m/s = 3.6 kph
        let meters_per_second = self.speed_kph / 3.6;
        let meters_traveled = meters_per_second / TICKS_PER_SECOND as f64;
        self.position_meters += meters_traveled;
    }

    pub fn increase_throttle(&mut self) {
        match self.throttle {
            Throttle::Power(p) => {
                if p < 8 {
                    self.throttle = Throttle::Power(p + 1);
                }
            }
            Throttle::Neutral => {
                self.throttle = Throttle::Power(1);
            }
            Throttle::Brake(b) => {
                if b == 1 {
                    self.throttle = Throttle::Neutral;
                } else {
                    self.throttle = Throttle::Brake(b - 1);
                }
            }
        }
    }

    pub fn decrease_throttle(&mut self) {
        match self.throttle {
            Throttle::Power(p) => {
                if p == 1 {
                    self.throttle = Throttle::Neutral;
                } else {
                    self.throttle = Throttle::Power(p - 1);
                }
            }
            Throttle::Neutral => {
                self.throttle = Throttle::Brake(1);
            }
            Throttle::Brake(b) => {
                if b < 8 {
                    self.throttle = Throttle::Brake(b + 1);
                }
            }
        }
    }

    pub fn increase_brake(&mut self) {
        match self.brake_setting {
            Brake::Release => self.brake_setting = Brake::Hold,
            Brake::Hold => self.brake_setting = Brake::Apply,
            Brake::Apply => self.brake_setting = Brake::Apply,
            Brake::Emergency => self.brake_setting = Brake::Emergency,
        }
    }

    pub fn decrease_brake(&mut self) {
        match self.brake_setting {
            Brake::Release => self.brake_setting = Brake::Release,
            Brake::Hold => self.brake_setting = Brake::Release,
            Brake::Apply => self.brake_setting = Brake::Hold,
            Brake::Emergency => {
                // you can only exit emergency brake when stopped
                if self.speed_kph < 1.0 {
                    self.brake_setting = Brake::Apply
                } else {
                    self.brake_setting = Brake::Emergency
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::Up => self.increase_throttle(),
            Input::Down => self.decrease_throttle(),
            Input::Left => self.increase_brake(),
            Input::Right => self.decrease_brake(),
            Input::EBrake => self.brake_setting = Brake::Emergency,
        }
    }

    pub fn render_ui(&self) -> Result<()> {
        let speed_display = if self.speed_kph < 0.1 {
            cformat!("<black!,bold>(   0 KPH )</>")
        } else {
            cformat!("<c,bold>( {:3.0} KPH )</>", self.speed_kph)
        };

        let brake_appl = self.brake_application();

        let throttle_disp = match self.throttle {
            Throttle::Power(p) => cformat!("<c,bold>[P{p}]</>"),
            Throttle::Neutral => {
                if brake_appl > 0.05 {
                    cformat!("<black!,bold>[N ]</>")
                } else {
                    cformat!("<g,bold>[N ]</>")
                }
            }
            Throttle::Brake(b) => cformat!("<y,bold>[B{b}]</>"),
        };
        let throttle_power = if let Throttle::Power(p) = self.throttle {
            "=".repeat(p as usize)
        } else {
            "".to_string()
        };
        let throttle_brake = if let Throttle::Brake(b) = self.throttle {
            "=".repeat(b as usize)
        } else {
            "".to_string()
        };

        let (pressure_disp, brake_percent) = if self.brake_pipe_pressure <= Self::EMERGENCY_PRESSURE
            || self.brake_setting == Brake::Emergency
        {
            (
                cformat!("<r,bold>({:.1})</>", self.brake_pipe_pressure),
                cformat!("<r,bold>{:3.0}%</>", brake_appl * 100.0),
            )
        } else if self.brake_application() > 0.05 {
            (
                cformat!("<y,bold>({:.1})</>", self.brake_pipe_pressure),
                cformat!("<y,bold>{:3.0}%</>", brake_appl * 100.0),
            )
        } else {
            (
                cformat!("<g,bold>({:.1})</>", self.brake_pipe_pressure),
                cformat!("<g,bold>{:3.0}%</>", brake_appl * 100.0),
            )
        };

        let brake_application = "=".repeat((brake_appl * 17.0) as usize);

        let release = if self.brake_setting == Brake::Release {
            cformat!("<bg:g,bold>R</>")
        } else {
            cformat!("<g,bold>R</>")
        };

        let hold = if self.brake_setting == Brake::Hold {
            cformat!("<bg:w,bold>H</>")
        } else {
            cformat!("<w,bold>H</>")
        };

        let apply = if self.brake_setting == Brake::Apply {
            cformat!("<bg:y,bold>A</>")
        } else {
            cformat!("<y,bold>A</>")
        };

        let emergency = if self.brake_setting == Brake::Emergency {
            cformat!("<bg:r,bold>E</>")
        } else {
            cformat!("<r,bold>E</>")
        };

        let brake_status = match self.brake_setting {
            Brake::Release => cformat!("<g>Release</>"),
            Brake::Hold => cformat!("<w>Hold</>"),
            Brake::Apply => cformat!("<y>Apply</>"),
            Brake::Emergency => cformat!("<r>Emergency</>"),
        };

        Element {
            lines: vec![
                cformat!("     {}", speed_display),
                cformat!(),
                cformat!(" <bold>Throttle:</>   {}", throttle_disp),
                cformat!(
                    " <bold>[<y>{:>8}</>|<c,bold>{:<8}</>]</>",
                    throttle_brake,
                    throttle_power,
                ),
                cformat!(),
                cformat!(" <bold>Brake:</>   {:4}", brake_percent),
                cformat!(" <bold>[<r>{:>17}</>]</>", brake_application),
                cformat!(),
                cformat!("Brake pressure: {}", pressure_disp),
                cformat!(
                    " <<{} {} {} {}>>  <bold>{}</>",
                    emergency,
                    apply,
                    hold,
                    release,
                    brake_status
                ),
            ],
            position: Position {
                x: 4,
                y: TRACK_Y + 3,
            },
        }
        .render()?;
        Ok(())
    }
}
