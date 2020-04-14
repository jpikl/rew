use crate::pattern::META_CHARS;
use crate::state::RegexTarget;
use clap::{App, Arg, ArgMatches, OsValues};
use regex::Regex;
use termcolor::ColorChoice;

const COLOR: &str = "color";
const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";
const ESCAPE: &str = "escape";
const PATHS: &str = "paths";
const PATTERN: &str = "pattern";
const REGEX: &str = "regex";
const REGEX_TARGET: &str = "regex-target";
const REGEX_TARGET_FILENAME: &str = "filename";
const REGEX_TARGET_PATH: &str = "path";
const ZERO_TERMINATED_STDIN: &str = "zero-terminated-stdin";

pub struct Cli<'a> {
    matches: ArgMatches<'a>,
}

impl<'a> Cli<'a> {
    pub fn new() -> Self {
        Self {
            matches: App::new(env!("CARGO_PKG_NAME"))
                .version(env!("CARGO_PKG_VERSION"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .arg(Self::pattern_arg())
                .arg(Self::paths_arg())
                .arg(Self::color_arg())
                .arg(Self::escape_arg())
                .arg(Self::regex_arg())
                .arg(Self::regex_target_arg())
                .arg(Self::zero_terminated_stdin_arg())
                .get_matches(),
        }
    }

    fn pattern_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PATTERN)
            .index(1)
            .required(true)
            .value_name("PATTERN")
            .help("Output pattern.")
    }

    pub fn pattern(&self) -> &str {
        self.matches.value_of(PATTERN).unwrap()
    }

    fn paths_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PATHS)
            .index(2)
            .multiple(true)
            .value_name("PATH")
            .help("Paths to process. If none provided, read paths from stdin.")
    }

    pub fn paths(&self) -> Option<OsValues> {
        self.matches.values_of_os(PATHS)
    }

    fn color_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(COLOR)
            .long("color")
            .takes_value(true)
            .value_name("WHEN")
            .possible_values(&[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI])
            .help("Output colors.")
    }

    pub fn color(&self) -> Option<ColorChoice> {
        self.matches.value_of(COLOR).map(|value| match value {
            COLOR_AUTO => ColorChoice::Auto,
            COLOR_ALWAYS => ColorChoice::Always,
            COLOR_ANSI => ColorChoice::AlwaysAnsi,
            COLOR_NEVER => ColorChoice::Never,
            _ => panic!("Unexpected {} value {}", COLOR, value),
        })
    }

    fn escape_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(ESCAPE)
            .long("escape")
            .takes_value(true)
            .value_name("CHAR")
            .validator(Self::validate_escape)
            .help("Custom escape character.")
    }

    fn validate_escape(value: String) -> Result<(), String> {
        let chars: Vec<char> = value.chars().collect();
        if chars.len() != 1 {
            Err("Value must be a single character".to_string())
        } else if META_CHARS.contains(&chars[0]) {
            Err(format!(
                "Cannot use one of meta characters {}",
                META_CHARS
                    .iter()
                    .map(|char| format!("'{}'", char))
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        } else {
            Ok(())
        }
    }

    pub fn escape(&self) -> Option<char> {
        self.matches
            .value_of(ESCAPE)
            .map(|value| value.chars().next().expect("Validation failed"))
    }

    fn regex_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(REGEX)
            .short("e")
            .long("regex")
            .takes_value(true)
            .value_name("EXPR")
            .validator(Self::validate_regex)
            .help("Regular expression to match against input.")
    }

    fn validate_regex(value: String) -> Result<(), String> {
        if let Err(error) = Regex::new(&value) {
            Err(error.to_string())
        } else {
            Ok(())
        }
    }

    pub fn regex(&self) -> Option<Regex> {
        self.matches
            .value_of(REGEX)
            .map(|value| Regex::new(value).expect("Validation failed"))
    }

    fn regex_target_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(REGEX_TARGET)
            .long("regex-target")
            .takes_value(true)
            .value_name("TARGET")
            .possible_values(&[REGEX_TARGET_PATH, REGEX_TARGET_FILENAME])
            .help("Part of input that is matched against regular expression.")
    }

    pub fn regex_target(&self) -> Option<RegexTarget> {
        self.matches
            .value_of(REGEX_TARGET)
            .map(|value| match value {
                REGEX_TARGET_PATH => RegexTarget::Path,
                REGEX_TARGET_FILENAME => RegexTarget::Filename,
                _ => panic!("Unexpected {} value {}", REGEX_TARGET, value),
            })
    }

    fn zero_terminated_stdin_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(ZERO_TERMINATED_STDIN)
            .short("z")
            .long("read0")
            .help("Paths from stdin are delimited by NUL byte instead of newline.")
    }

    pub fn zero_terminated_stdin(&self) -> bool {
        self.matches.is_present(ZERO_TERMINATED_STDIN)
    }
}
