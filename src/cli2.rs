use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::str::FromStr;

#[derive(Clone)]
struct OptArg<'a> {
    id: u32,
    short: Option<char>,
    long: Option<&'a str>,
    description: &'a str,
    value: Option<Value<'a>>,
}

#[derive(Clone)]
struct PosArg<'a> {
    id: u32,
    description: &'a str,
    value: Value<'a>,
    required: bool,
    multiple: bool,
}

#[derive(Clone)]
struct Value<'a> {
    name: &'a str,
    default: Option<&'a str>,
    enum_items: &'a [EnumItem<'a>],
}

#[derive(Clone)]
struct EnumItem<'a> {
    name: &'a str,
    description: &'a str,
}

#[derive(Clone)]
struct Command<'a> {
    id: u32,
    name: &'a str,
    description: &'a str,
    options: &'a [Cow<'a, OptArg<'a>>],
    positionals: &'a [Cow<'a, PosArg<'a>>],
    subcommands: &'a [Cow<'a, Command<'a>>],
}

impl<'a> Command<'a> {
    fn parse<A: IntoIterator<IntoIter = I>, I: Iterator<Item = OsString>>(&'a self, args: A) -> Parser<'a, I> {
        Parser::new(self, args.into_iter())
    }
}

struct Parser<'a, I> {
    command: &'a Command<'a>,
    args: I,
}

impl<'a, I: Iterator<Item = OsString>> Parser<'a, I> {
    fn new(command: &'a Command<'a>, args: I) -> Self {
        Self { command, args }
    }

    fn next_match(&mut self) -> Result<Option<Match<'a>>, ValueError> {
        todo!()
    }

    fn handle_match(&self, m: Match<'a>) -> Result<(), ValueError> {
        todo!()
    }
}

enum Match<'a> {
    Arg(u32, ArgMatch<'a>),
    Command(u32, CommandMatch<'a>),
}

enum ArgMatch<'a> {
    Opt(&'a OptArg<'a>),
    OptVal(&'a OptArg<'a>, &'a OsStr),
    PosVal(&'a PosArg<'a>, &'a OsStr),
}

impl<'a> ArgMatch<'a> {
    fn osstr_value(&self) -> Result<&'a OsStr, ValueError> {
        match self {
            Self::Opt(_) => Err(ValueError::MissingValue),
            Self::OptVal(_, val) => Ok(val),
            Self::PosVal(_, val) => Ok(val),
        }
    }

    fn str_value(&self) -> Result<&'a str, ValueError> {
        match self.osstr_value() {
            Ok(os_str) => os_str.to_str().ok_or(ValueError::InvalidValue),
            Err(err) => Err(err),
        }
    }

    fn parse_value<T: FromStr>(&self) -> Result<T, ValueError> {
        match self.str_value() {
            Ok(str) => FromStr::from_str(str).map_err(|_| ValueError::InvalidValue),
            Err(err) => Err(err),
        }
    }
}

enum ValueError {
    MissingValue,
    InvalidValue,
}

struct CommandMatch<'a> {
    command: &'a Command<'a>,
}

macro_rules! declare_ids {
    // Entry: first item with required value, optionally followed by more names
    ( $first:ident = $start:expr $(, $($rest:ident),* )? $(,)? ) => {
        pub const $first: u32 = $start;
        // If there are more identifiers, begin emitting them starting at start+1
        $( declare_ids!(@emit ($start + 1u32) $($rest),*); )?
    };

    // Emit multiple remaining identifiers (recursive)
    (@emit ($curr:expr) $name:ident, $($rest:ident),+ ) => {
        pub const $name: u32 = $curr;
        declare_ids!(@emit (($curr) + 1u32) $($rest),+);
    };

    // Emit last remaining identifier
    (@emit ($curr:expr) $name:ident $(,)?) => {
        pub const $name: u32 = $curr;
    };
}

declare_ids! {
    COMMAND_ID = 100,
    FLAG_ID,
    OPTION_ID,
    POSITIONAL_ID,
}

#[test]
fn test() {
    assert!(run().is_ok());
}

fn run() -> Result<(), ValueError> {
    let flag: OptArg = OptArg {
        id: FLAG_ID,
        short: Some('f'),
        long: Some("flag"),
        description: "The flag",
        value: None,
    };

    let option: OptArg = OptArg {
        id: OPTION_ID,
        short: Some('o'),
        long: Some("option"),
        description: "The option",
        value: Some(Value {
            name: "VALUE",
            default: None,
            enum_items: &[],
        }),
    };

    let command: Command = Command {
        id: COMMAND_ID,
        name: "cmd",
        description: "The command",
        options: &[Cow::Borrowed(&flag), Cow::Borrowed(&option)],
        positionals: &[Cow::Owned(PosArg {
            id: POSITIONAL_ID,
            description: "The positional",
            value: Value {
                name: "POSITIONAL",
                default: None,
                enum_items: &[],
            },
            required: false,
            multiple: false,
        })],
        subcommands: &[],
    };

    let mut flag = false;
    let mut option = 0;
    let mut positional = "";

    let mut parser = command.parse(std::env::args_os());

    while let Some(res) = parser.next_match()? {
        match res {
            Match::Arg(FLAG_ID, _) => flag = true,
            Match::Arg(OPTION_ID, arg) => option = arg.parse_value()?,
            Match::Arg(POSITIONAL_ID, arg) => positional = arg.str_value()?,
            _ => parser.handle_match(res)?,
        }
    }

    assert!(flag);
    assert_eq!(option, 1);
    assert_eq!(positional, "x");

    Ok(())
}
