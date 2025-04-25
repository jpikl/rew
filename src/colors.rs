use std::fmt::Display;

pub const RESET: &str = "\x1b[0m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";

pub const BOLD: &str = "\x1b[1m";
pub const BOLD_RED: &str = "\x1b[1;31m";

pub struct Colorizer<T>(pub T);

impl<T: Display> Display for Colorizer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inside = false;

        for part in self.0.to_string().split('`') {
            if inside {
                write!(f, "'{YELLOW}{part}{RESET}'")?;
            } else {
                write!(f, "{part}")?;
            }
            inside = !inside;
        }

        Ok(())
    }
}
