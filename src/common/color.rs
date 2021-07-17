use termcolor::{Color, ColorChoice, ColorSpec};

pub const COLOR_CHOICES: &[&str] = &[AUTO, ALWAYS, ANSI, NEVER];

const AUTO: &str = "auto";
const ALWAYS: &str = "always";
const ANSI: &str = "ansi";
const NEVER: &str = "never";

pub fn parse_color(string: &str) -> Result<ColorChoice, &'static str> {
    match string {
        AUTO => Ok(ColorChoice::Auto),
        ALWAYS => Ok(ColorChoice::Always),
        ANSI => Ok(ColorChoice::AlwaysAnsi),
        NEVER => Ok(ColorChoice::Never),
        _ => Err("invalid value"),
    }
}

pub fn choose_color(color: Option<ColorChoice>) -> ColorChoice {
    match color {
        Some(ColorChoice::Auto) | None => detect_color(),
        Some(other) => other,
    }
}

fn detect_color() -> ColorChoice {
    if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
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
    use test_case::test_case;

    use super::*;

    #[test_case("",     Err("invalid value")        ; "empty")]
    #[test_case("x",    Err("invalid value")        ; "invalid")]
    #[test_case(ALWAYS, Ok(ColorChoice::Always)     ; "always")]
    #[test_case(ANSI,   Ok(ColorChoice::AlwaysAnsi) ; "ansi")]
    #[test_case(AUTO,   Ok(ColorChoice::Auto)       ; "auto")]
    #[test_case(NEVER,  Ok(ColorChoice::Never)      ; "never")]
    fn parse_color(input: &str, result: Result<ColorChoice, &'static str>) {
        assert_eq!(super::parse_color(input), result);
    }

    #[test_case(None,                          detect_color()          ; "none")]
    #[test_case(Some(ColorChoice::Auto),       detect_color()          ; "auto")]
    #[test_case(Some(ColorChoice::Always),     ColorChoice::Always     ; "always")]
    #[test_case(Some(ColorChoice::AlwaysAnsi), ColorChoice::AlwaysAnsi ; "ansi")]
    #[test_case(Some(ColorChoice::Never),      ColorChoice::Never      ; "never")]
    fn choose_color(value: Option<ColorChoice>, result: ColorChoice) {
        assert_eq!(super::choose_color(value), result)
    }

    #[test]
    fn spec_color() {
        assert_eq!(
            &super::spec_color(Color::Red),
            ColorSpec::new().set_fg(Some(Color::Red))
        );
    }

    #[test]
    fn spec_bold_color() {
        assert_eq!(
            &super::spec_bold_color(Color::Red),
            ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)
        );
    }
}
