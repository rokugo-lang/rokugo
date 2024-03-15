use std::io;

use termcolor::{Color, WriteColor};

pub trait ColoredDisplay {
    fn fmt_with_color(&self, f: &mut dyn WriteColor) -> io::Result<()>;
}

#[derive(Debug)]
pub struct ColorSpec {
    pub fg: Option<Color>,
    pub intense: bool,
}

impl ColorSpec {
    pub const fn default() -> Self {
        Self {
            fg: None,
            intense: false,
        }
    }
}

impl From<ColorSpec> for termcolor::ColorSpec {
    fn from(spec: ColorSpec) -> Self {
        let mut c = termcolor::ColorSpec::new();
        c.set_fg(spec.fg);
        c.set_intense(spec.intense);
        c
    }
}
