use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::{cursor, queue, style::Print};

pub const METERS_PER_CHAR: f64 = 4.0;
pub const MAX_WIDTH: u16 = 120;
pub const TRACK_Y: u16 = 6;

pub trait StaticRender {
    fn render(&self) -> Result<()>;
}
pub trait DynamicRender {
    fn render(&self, position_meters: f64) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub lines: Vec<String>,
    pub position: Position,
}

impl StaticRender for Element {
    fn render(&self) -> Result<()> {
        let Position { x, y } = self.position;
        let mut stdout = stdout();
        for (i, row) in self.lines.iter().enumerate() {
            queue!(stdout, cursor::MoveTo(x, y + (i as u16)), Print(row),)?;
        }
        stdout.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub position_meters: f64,
    pub element: Element,
}

impl DynamicRender for Entity {
    fn render(&self, screen_position_meters: f64) -> Result<()> {
        let meter_offset = self.position_meters - screen_position_meters;

        if meter_offset < 0.0 {
            return Ok(());
        }

        let offset = (meter_offset / METERS_PER_CHAR).round() as u16;

        let Position { mut x, y } = self.element.position;
        x += offset;

        if x > MAX_WIDTH {
            return Ok(());
        }

        let updated_element = Element {
            lines: self.element.lines.clone(),
            position: Position { x, y },
        };
        updated_element.render()
    }
}

// pub fn render_static_ui() -> Result<()> {
//     // track
//     Element {
//         lines: vec!["=".repeat(MAX_WIDTH as usize)],
//         position: Position { x: 0, y: TRACK_Y },
//     }
//     .render()?;

//     // train ui
//     Element {
//         lines: vec![
//             format!("+{}+", "-".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("+{}+", "-".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("|{}|", " ".repeat(21)),
//             format!("+{}+", "-".repeat(21)),
//         ],
//         position: Position {
//             x: 3,
//             y: TRACK_Y + 2,
//         },
//     }
//     .render()?;

//     // signaling system and station ui
//     Element {
//         lines: vec![
//             format!("+{}+", "-".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("+{}+", "-".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("|{}|", " ".repeat(25)),
//             format!("+{}+", "-".repeat(25)),
//         ],
//         position: Position {
//             x: 30,
//             y: TRACK_Y + 2,
//         },
//     }
//     .render()?;

//     Ok(())
// }
