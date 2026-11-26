use std::array;

use anyhow::Result;
use color_print::cformat;

use crate::{
    Game, is_overlapping,
    render::{Element, MAX_WIDTH, Position, StaticRender, TRACK_Y},
    signal::MIN_SPEED_LIMIT,
    train::{Brake, Throttle, Train},
    train_control::{EMERGENCY_BRAKE_THRESHOLD, PENALTY_BRAKE_THRESHOLD, SpeedEnforcement},
};

pub fn render_ui(game: &Game) -> Result<()> {
    render_static_ui()?;
    render_train_ui(game)?;
    render_signals_ui(game)?;
    render_next_station_ui(game)?;

    Ok(())
}

pub fn render_static_ui() -> Result<()> {
    // track
    Element {
        lines: vec!["=".repeat(MAX_WIDTH as usize)],
        position: Position { x: 0, y: TRACK_Y },
    }
    .render()?;

    // train ui
    Element {
        lines: vec![
            format!("+{}+", "-".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("+{}+", "-".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("|{}|", " ".repeat(21)),
            format!("+{}+", "-".repeat(21)),
        ],
        position: Position {
            x: 3,
            y: TRACK_Y + 2,
        },
    }
    .render()?;

    // signaling system and station ui
    Element {
        lines: vec![
            format!("+{}+", "-".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("+{}+", "-".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("|{}|", " ".repeat(25)),
            format!("+{}+", "-".repeat(25)),
        ],
        position: Position {
            x: 30,
            y: TRACK_Y + 2,
        },
    }
    .render()?;

    Ok(())
}

pub fn render_train_ui(game: &Game) -> Result<()> {
    let speed_display = if game.train.get_speed() < 0.1 {
        cformat!("<black!,bold>(   0 KPH )</>")
    } else {
        cformat!("<c,bold>( {:3.0} KPH )</>", game.train.get_speed())
    };

    let brake_appl = game.train.brake_application();

    let throttle_disp = match game.train.get_throttle_setting() {
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
    let throttle_power = if let Throttle::Power(p) = game.train.get_throttle_setting() {
        "=".repeat(p as usize)
    } else {
        "".to_string()
    };
    let throttle_brake = if let Throttle::Brake(b) = game.train.get_throttle_setting() {
        "=".repeat(b as usize)
    } else {
        "".to_string()
    };

    let (pressure_disp, brake_percent) = if game.train.get_brake_pressure()
        <= Train::EMERGENCY_PRESSURE
        || game.train.get_brake_setting() == Brake::Emergency
    {
        (
            cformat!("<r,bold>({:.1})</>", game.train.get_brake_pressure()),
            cformat!("<r,bold>{:3.0}%</>", brake_appl * 100.0),
        )
    } else if game.train.brake_application() > 0.05 {
        (
            cformat!("<y,bold>({:.1})</>", game.train.get_brake_pressure()),
            cformat!("<y,bold>{:3.0}%</>", brake_appl * 100.0),
        )
    } else {
        (
            cformat!("<g,bold>({:.1})</>", game.train.get_brake_pressure()),
            cformat!("<g,bold>{:3.0}%</>", brake_appl * 100.0),
        )
    };

    let brake_application = "=".repeat((brake_appl * 17.0) as usize);

    let release = if game.train.get_brake_setting() == Brake::Release {
        cformat!("<bg:g,bold>R</>")
    } else {
        cformat!("<g,bold>R</>")
    };

    let hold = if game.train.get_brake_setting() == Brake::Hold {
        cformat!("<bg:w,bold>H</>")
    } else {
        cformat!("<w,bold>H</>")
    };

    let apply = if game.train.get_brake_setting() == Brake::Apply {
        cformat!("<bg:y,bold>A</>")
    } else {
        cformat!("<y,bold>A</>")
    };

    let emergency = if game.train.get_brake_setting() == Brake::Emergency {
        cformat!("<bg:r,bold>E</>")
    } else {
        cformat!("<r,bold>E</>")
    };

    let brake_status = match game.train.get_brake_setting() {
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

pub fn render_signals_ui(game: &Game) -> Result<()> {
    let enforcement = game.train.get_train_control_enforcement();

    let SpeedEnforcement {
        limit,
        reduction,
        stop_close,
    } = enforcement;

    let mut signal_display: [String; 3] = array::from_fn(|_| cformat!("<black!>-----</>"));

    // show stop
    if stop_close {
        signal_display[2] = cformat!("<r,bold>[000]</>");
    }
    // show speed limit
    else if limit <= reduction {
        signal_display[0] = cformat!("<g,bold>[{:>3}]</>", limit);
    }
    // show reduction
    else {
        signal_display[1] = cformat!("<y,bold>[<i>{:>3}</>]</>", reduction.max(MIN_SPEED_LIMIT));
    }

    let signal_display_string = signal_display.concat();

    let rounded_speed = game.train.get_speed().round() as i16;
    let speed_excess = rounded_speed - (enforcement.limit as i16);
    let speed_reduce = rounded_speed - (enforcement.reduction as i16);

    // speeding
    let status0 = if speed_excess > 0 {
        cformat!("<r,bold>OVERSPEED</>")
    }
    // stop ahead
    else if enforcement.reduction == 0 {
        cformat!("<y>Stop Ahead</>")
    }
    // reduce speed
    else if speed_reduce > 0 {
        cformat!("<y>Reduce Speed Ahead</>")
    } else {
        String::new()
    };

    // emergency brake
    let status1 = if speed_excess >= EMERGENCY_BRAKE_THRESHOLD
        || game.train.get_brake_setting() == Brake::Emergency
    {
        cformat!("<r,bold>EMERGENCY BRAKE APPLIED")
    }
    // penalty brake
    else if speed_excess >= PENALTY_BRAKE_THRESHOLD {
        cformat!("<r,bold>PENALTY BRAKE APPLIED</>")
    } else {
        String::new()
    };

    Element {
        lines: vec![
            format!("    {}", signal_display_string),
            String::new(),
            status0,
            status1,
        ],
        position: Position {
            x: 32,
            y: TRACK_Y + 3,
        },
    }
    .render()?;

    Ok(())
}

pub fn render_next_station_ui(game: &Game) -> Result<()> {
    let train_front_pos = game.train.front_position();
    let train_rear_pos = game.train.rear_position();

    let station_start = game.next_station.get_position_meters();
    let station_end = game.next_station.get_end_position();

    let distance_meters = station_start - train_front_pos;
    let distance_km = distance_meters / 1000.0;

    let is_at_station = is_overlapping(
        (train_rear_pos, train_front_pos),
        (station_start, station_end),
    );

    let message = if !is_at_station {
        cformat!("Next station: ")
            + &if distance_meters > 1000.0 {
                cformat!("<g,bold>{:.1}km</>", distance_km)
            } else {
                cformat!("<g,bold>{:.0}m</>", distance_meters)
            }
    } else {
        cformat!("Stop at the station")
    };

    let station_name = game.next_station.get_name();

    Element {
        lines: vec![message, cformat!("<c,bold>{}</>", station_name)],
        position: Position {
            x: 32,
            y: TRACK_Y + 8,
        },
    }
    .render()?;

    Ok(())
}
