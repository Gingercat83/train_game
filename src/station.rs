use anyhow::Result;
use color_print::cformat;
use rand::{random_bool, random_range};

use crate::render::{DynamicRender, Element, Entity, Position, TRACK_Y};

pub const MIN_STATION_DIST: f64 = 1000.0;
pub const MAX_STATION_DIST: f64 = 5000.0;

#[derive(Debug, Clone)]
pub struct Station {
    position_meters: f64,
    name: String,
}

impl DynamicRender for Station {
    fn render(&self, position_meters: f64) -> Result<()> {
        Entity {
            position_meters: self.position_meters,
            element: Element {
                lines: vec![
                    cformat!("{:^21}", self.name),
                    cformat!("<blue>{:^21}</>", cformat!("/{}\\", "#".repeat(13))),
                    cformat!("   <bg:y>{}</>", "-".repeat(15),),
                    cformat!(
                        "   <bg:y>{}<bg:black> </>{}</>",
                        " ".repeat(7),
                        " ".repeat(7)
                    ),
                    cformat!("<bg:r>{}</>", " ".repeat(21)),
                    cformat!("<g>{}</>", "=".repeat(21)),
                ],
                position: Position {
                    x: 0,
                    y: TRACK_Y - 5,
                },
            },
        }
        .render(position_meters)?;

        Ok(())
    }
}

impl Station {
    pub fn new(position_meters: f64) -> Self {
        Station {
            position_meters,
            name: generate_station_name(),
        }
    }

    pub fn get_position_meters(&self) -> f64 {
        self.position_meters
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn next_station(position_meters: f64) -> Self {
        let distance = random_range(MIN_STATION_DIST..MAX_STATION_DIST);
        let new_position = position_meters + distance;
        Self::new(new_position)
    }
}

pub fn generate_station_name() -> String {
    let prefixes = ["New ", "East ", "West ", "North ", "South ", "Old "];
    let names = [
        "Chester",
        "Chardon",
        "Bellvue",
        "Spring",
        "Summer",
        "Ginger",
        "Auburn",
        "Cedar",
        "Shaker",
        "Mentor",
        "Chicken",
        "Burger",
        "Australia",
        "[ERROR 404] Not Found",
        "syntax error: `station_name` is not defined in the current scope",
        "The United States of America",
        "Mars",
        "Iacon",
        "Zyzzyx"
    ];
    let suffixes = [
        "ville", " City", " Center", "ton", "burg", "land", " Street", " Heights", " Central",
        " town",
    ];

    let use_prefix = random_bool(0.5);
    let use_suffix = random_bool(0.5);

    let prefix = if use_prefix {
        prefixes[random_range(0..prefixes.len())]
    } else {
        ""
    };
    let suffix = if use_suffix {
        suffixes[random_range(0..suffixes.len())]
    } else {
        ""
    };

    let name = names[random_range(0..names.len())];
    format!("{prefix}{name}{suffix}")
}
