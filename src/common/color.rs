use termcolor::{Color, ColorChoice, ColorSpec};

pub const COLOR_VALUES: &[&str] = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI];

const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";

pub fn parse_color(string: &str) -> Result<ColorChoice, &'static str> {
    match string {
        COLOR_ALWAYS => Ok(ColorChoice::Always),
        COLOR_ANSI => Ok(ColorChoice::AlwaysAnsi),
        COLOR_AUTO => Ok(ColorChoice::Auto),
        COLOR_NEVER => Ok(ColorChoice::Never),
        _ => Err("invalid value"),
    }
}

pub fn detect_color(color: Option<ColorChoice>) -> ColorChoice {
    match color {
        Some(ColorChoice::Auto) | None => {
            if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
        Some(other) => other,
    }
}

pub fn spec_color(color: Color) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
}

pub fn spec_bold_color(color: Color) -> ColorSpec {
    let mut spec = spec_color(color);
    spec.set_bold(true);
    spec
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_color {
        use super::*;

        #[test]
        fn valid() {
            assert_eq!(parse_color(COLOR_ALWAYS), Ok(ColorChoice::Always));
            assert_eq!(parse_color(COLOR_ANSI), Ok(ColorChoice::AlwaysAnsi));
            assert_eq!(parse_color(COLOR_AUTO), Ok(ColorChoice::Auto));
            assert_eq!(parse_color(COLOR_NEVER), Ok(ColorChoice::Never));
        }

        #[test]
        fn invalid() {
            assert_eq!(parse_color(""), Err("invalid value"));
            assert_eq!(parse_color("x"), Err("invalid value"));
        }
    }

    mod detect_color {
        use super::*;

        #[test]
        fn selected() {
            assert_eq!(detect_color(Some(ColorChoice::Never)), ColorChoice::Never);
            assert_eq!(detect_color(Some(ColorChoice::Always)), ColorChoice::Always);
            assert_eq!(
                detect_color(Some(ColorChoice::AlwaysAnsi)),
                ColorChoice::AlwaysAnsi
            );
        }

        #[test]
        fn auto() {
            let color = if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            };
            assert_eq!(detect_color(None), color);
            assert_eq!(detect_color(Some(ColorChoice::Auto)), color);
        }
    }

    #[test]
    fn spec_color() {
        use super::*;

        assert_eq!(
            &spec_color(Color::Red),
            ColorSpec::new().set_fg(Some(Color::Red))
        );
    }

    #[test]
    fn spec_bold_color() {
        use super::*;

        assert_eq!(
            &spec_bold_color(Color::Red),
            ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)
        );
    }
}
