use crate::pattern::META_CHARS;
use crate::state::RegexTarget;
use clap::{App, Arg, ArgMatches, OsValues};
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
            matches: Cli::init(),
        }
    }

    fn init() -> ArgMatches<'a> {
        App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name(PATTERN)
                    .index(1)
                    .required(true)
                    .value_name("PATTERN")
                    .help("Output pattern."),
            )
            .arg(
                Arg::with_name(PATHS)
                    .index(2)
                    .multiple(true)
                    .value_name("PATH")
                    .help("Paths to process. If none provided, read paths from stdin."),
            )
            .arg(
                Arg::with_name(COLOR)
                    .long("color")
                    .takes_value(true)
                    .value_name("WHEN")
                    .possible_values(&[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI])
                    .default_value(COLOR_AUTO)
                    .help("Output color configuration."),
            )
            .arg(
                Arg::with_name(ESCAPE)
                    .long("escape")
                    .takes_value(true)
                    .value_name("CHAR")
                    .validator(|value| {
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
                    })
                    .help("Custom escape character."),
            )
            .arg(
                Arg::with_name(REGEX)
                    .short("e")
                    .long("regex")
                    .takes_value(true)
                    .value_name("EXPR")
                    .help("Regular expression to match against input."),
            )
            .arg(
                Arg::with_name(REGEX_TARGET)
                    .long("regex-target")
                    .takes_value(true)
                    .value_name("TARGET")
                    .possible_values(&[REGEX_TARGET_PATH, REGEX_TARGET_FILENAME])
                    .default_value(REGEX_TARGET_FILENAME)
                    .help("Part of input that is matched against regular expression."),
            )
            .arg(
                Arg::with_name(ZERO_TERMINATED_STDIN)
                    .short("z")
                    .long("read0")
                    .help("Paths from stdin are delimited by NUL byte instead of newline."),
            )
            .get_matches()
    }

    pub fn pattern(&self) -> &str {
        self.matches.value_of(PATTERN).unwrap()
    }

    pub fn paths(&self) -> Option<OsValues> {
        self.matches.values_of_os(PATHS)
    }

    pub fn color(&self) -> ColorChoice {
        match self.matches.value_of(COLOR) {
            Some(COLOR_ALWAYS) => ColorChoice::Always,
            Some(COLOR_ANSI) => ColorChoice::AlwaysAnsi,
            Some(COLOR_NEVER) => ColorChoice::Never,
            _ => ColorChoice::Auto,
        }
    }

    pub fn escape(&self) -> Option<char> {
        self.matches
            .value_of(ESCAPE)
            .and_then(|value| value.chars().next())
    }

    pub fn regex(&self) -> Option<&str> {
        self.matches.value_of(REGEX)
    }

    pub fn regex_target(&self) -> RegexTarget {
        match self.matches.value_of(REGEX_TARGET) {
            Some(REGEX_TARGET_PATH) => RegexTarget::Path,
            _ => RegexTarget::Filename,
        }
    }

    pub fn zero_terminated_stdin(&self) -> bool {
        self.matches.is_present(ZERO_TERMINATED_STDIN)
    }
}
