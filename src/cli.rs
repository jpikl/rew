use clap::{App, Arg, ArgMatches, OsValues};
use termcolor::ColorChoice;

const COLOR: &str = "color";
const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";
const FILES: &str = "files";
const PATTERN: &str = "pattern";
const ZERO_TERMINATED: &str = "zero-terminated";

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
                    .help("Rename pattern."),
            )
            .arg(
                Arg::with_name(FILES)
                    .index(2)
                    .multiple(true)
                    .value_name("FILE")
                    .help("Files to rename. Omit to read filenames as lines from standard input."),
            )
            .arg(
                Arg::with_name(COLOR)
                    .long("color")
                    .takes_value(true)
                    .value_name("WHEN")
                    .possible_values(&[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI])
                    .help("Color usage."),
            )
            .arg(
                Arg::with_name(ZERO_TERMINATED)
                    .short("z")
                    .long("zero-terminated")
                    .help("Filenames from stdin are separated by NUL instead of LF."),
            )
            .get_matches()
    }

    pub fn pattern(&self) -> &str {
        self.matches.value_of(PATTERN).unwrap()
    }

    pub fn color(&self) -> ColorChoice {
        match self.matches.value_of(COLOR) {
            Some(COLOR_ALWAYS) => ColorChoice::Always,
            Some(COLOR_ANSI) => ColorChoice::AlwaysAnsi,
            Some(COLOR_NEVER) => ColorChoice::Never,
            _ => ColorChoice::Auto,
        }
    }

    pub fn zero_terminated(&self) -> bool {
        self.matches.is_present(ZERO_TERMINATED)
    }

    pub fn files(&self) -> Option<OsValues> {
        self.matches.values_of_os(FILES)
    }
}
