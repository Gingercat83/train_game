use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::{cursor, queue, style::Print};

pub const METERS_PER_CHAR: f64 = 4.0;
pub const MAX_WIDTH: u16 = 120;
pub const TRACK_Y: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VerticalAlignment {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HorizontalAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Alignment {
    pub vertical: VerticalAlignment,
    pub horizontal: HorizontalAlignment,
}

impl Alignment {
    pub fn from_str(value: &str) -> Self {
        let value = value.to_lowercase();
        if value.len() > 2 {
            panic!("expected 0-2 characters, got {}", value.len());
        }

        let has_r = value.contains('r');
        let has_l = value.contains('l');
        let has_t = value.contains('t');
        let has_b = value.contains('b');

        if has_r && has_l {
            panic!("Cannot have left and right");
        }
        if has_t && has_b {
            panic!("Cannot have top and bottom");
        }

        let vertical = if has_t {
            VerticalAlignment::Top
        } else if has_b {
            VerticalAlignment::Bottom
        } else {
            VerticalAlignment::Center
        };

        let horizontal = if has_l {
            HorizontalAlignment::Left
        } else if has_r {
            HorizontalAlignment::Right
        } else {
            HorizontalAlignment::Center
        };

        Self {
            vertical,
            horizontal,
        }
    }
}

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

pub fn render_static_ui() -> Result<()> {
    // track
    Element {
        lines: vec!["=".repeat(MAX_WIDTH as usize)],
        position: Position { x: 0, y: TRACK_Y },
    }
    .render()?;

    // ui
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
    Ok(())
}
