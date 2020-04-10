use clap::{App, Arg, ArgMatches, OsValues};
use termcolor::ColorChoice;

const COLOR: &str = "color";
const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";
const PATHS: &str = "paths";
const PATTERN: &str = "pattern";
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
                    .help("Output color configuration."),
            )
            .arg(
                Arg::with_name(ZERO_TERMINATED_STDIN)
                    .short("z")
                    .long("--read0")
                    .help("Paths from stdin are delimited by NUL byte instead of newline."),
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

    pub fn zero_terminated_stdin(&self) -> bool {
        self.matches.is_present(ZERO_TERMINATED_STDIN)
    }

    pub fn paths(&self) -> Option<OsValues> {
        self.matches.values_of_os(PATHS)
    }
}
