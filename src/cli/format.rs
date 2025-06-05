use std::fmt::Display;

pub const ERROR_START: &str = "\x1b[1;31m";
pub const ERROR_END: &str = "\x1b[0m";

pub const QUOTE_START: &str = "'\x1b[1;33m";
pub const QUOTE_END: &str = "\x1b[0m'";

pub const SECTION_START: &str = "\x1b[4;1m";
pub const SECTION_END: &str = "\x1b[0m";

pub const HIGHLIGHT_START: &str = "\x1b[1m";
pub const HIGHLIGHT_END: &str = "\x1b[0m";

pub struct Fmt<T>(&'static str, T, &'static str);

impl<T> Fmt<T> {
    pub fn quote(value: T) -> Self {
        Self(QUOTE_START, value, QUOTE_END)
    }
}

impl<T: Display> Display for Fmt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.0, self.1, self.2)
    }
}

pub struct Highlight<'a>(pub &'a str);

impl Display for Highlight<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inside = false;

        for part in self.0.split('`') {
            if inside {
                write!(f, "'{HIGHLIGHT_START}{part}{HIGHLIGHT_END}'")?;
            } else {
                write!(f, "{part}")?;
            }
            inside = !inside;
        }

        Ok(())
    }
}
