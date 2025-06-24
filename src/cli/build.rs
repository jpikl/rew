use super::ARGUMENTS;
use super::COMMANDS;
use super::Command;
use super::CommandAlias;
use super::CommandRun;
use super::Enum;
use super::EnumItem;
use super::Flag;
use super::Group;
use super::OPTIONS;
use super::Opt;
use super::OptArg;
use super::OptErrHint;
use super::OptRun;
use super::ParseValue;
use super::Pos;
use super::PosArg;
use super::TypedArg;
use super::Value;
use std::ffi::OsStr;
use std::marker::PhantomData;

pub struct CommandBuilder<'a> {
    name: &'a str,
    description: &'a str,
    description_ex: &'a [&'a str],
    version: Option<&'a str>,
    group: &'a Group<'a>,
    options: &'a [&'a OptArg<'a>],
    positionals: &'a [&'a PosArg<'a>],
    subcommands: &'a [&'a Command<'a>],
    subcommand_aliases: &'a [&'a CommandAlias<'a>],
    run: CommandRun,
}

impl<'a> CommandBuilder<'a> {
    pub const fn new() -> Self {
        Self {
            name: "",
            description: "",
            description_ex: &[],
            version: None,
            group: &COMMANDS,
            options: &[],
            positionals: &[],
            subcommands: &[],
            subcommand_aliases: &[],
            run: |_| unimplemented!("command not implemented"),
        }
    }

    pub const fn from(other: &Command<'a>) -> Self {
        Self {
            name: other.name,
            description: other.description,
            description_ex: other.description_ex,
            version: other.version,
            group: other.group,
            options: other.options,
            positionals: other.positionals,
            subcommands: other.subcommands,
            subcommand_aliases: other.subcommand_aliases,
            run: other.run,
        }
    }

    pub const fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'a [&'a str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub const fn group(mut self, group: &'a Group) -> Self {
        self.group = group;
        self
    }

    pub const fn options(mut self, options: &'a [&'a OptArg<'a>]) -> Self {
        self.options = options;
        self
    }

    pub const fn positionals(mut self, positionals: &'a [&'a PosArg]) -> Self {
        self.positionals = positionals;
        self
    }

    pub const fn subcommands(mut self, subcommands: &'a [&'a Command<'a>]) -> Self {
        self.subcommands = subcommands;
        self
    }

    pub const fn subcommand_aliases(mut self, aliases: &'a [&'a CommandAlias<'a>]) -> Self {
        self.subcommand_aliases = aliases;
        self
    }

    pub const fn run(mut self, run: CommandRun) -> Self {
        self.run = run;
        self
    }

    pub const fn done(self) -> Command<'a> {
        Command {
            name: self.name,
            description: self.description,
            description_ex: self.description_ex,
            version: self.version,
            group: self.group,
            options: self.options,
            positionals: self.positionals,
            subcommands: self.subcommands,
            subcommand_aliases: self.subcommand_aliases,
            run: self.run,
        }
    }
}

pub struct FlagBuilder<'a> {
    short: Option<char>,
    long: Option<&'a str>,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
    run: Option<OptRun>,
    err_hint: Option<OptErrHint>,
}

impl<'a> FlagBuilder<'a> {
    pub const fn new() -> Self {
        Self {
            short: None,
            long: None,
            description: "",
            description_ex: &[],
            group: &OPTIONS,
            environment: None,
            run: None,
            err_hint: None,
        }
    }

    pub const fn from(other: &OptArg<'a>) -> Self {
        Self {
            short: other.short,
            long: other.long,
            description: other.description,
            description_ex: other.description_ex,
            group: other.group,
            environment: other.environment,
            run: other.run,
            err_hint: other.err_hint,
        }
    }

    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub const fn long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'a [&'a str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn group(mut self, group: &'a Group) -> Self {
        self.group = group;
        self
    }

    pub const fn environment(mut self, environment: &'a str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn run(mut self, run: OptRun) -> Self {
        self.run = Some(run);
        self
    }

    pub const fn err_hint(mut self, err_hint: OptErrHint) -> Self {
        self.err_hint = Some(err_hint);
        self
    }

    pub const fn done(self) -> Flag<'a> {
        TypedArg::new(OptArg {
            short: self.short,
            long: self.long,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            run: self.run,
            err_hint: self.err_hint,
            value: None,
        })
    }
}

pub struct OptBuilder<'a, T> {
    short: Option<char>,
    long: Option<&'a str>,
    value_name: &'a str,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
    enum_items: &'a [EnumItem<'a>],
    default: Option<&'a str>,
    run: Option<OptRun>,
    err_hint: Option<OptErrHint>,
    _type: PhantomData<T>,
}

impl<'a, T: ParseValue<OsStr> + Enum<'a> + 'a> OptBuilder<'a, T> {
    pub const fn new_enum() -> Self {
        Self::new_with_enum_items(T::ENUM_ITEMS)
    }
}

impl<'a, T: ParseValue<OsStr> + 'a> OptBuilder<'a, T> {
    pub const fn new() -> Self {
        Self::new_with_enum_items(&[])
    }

    const fn new_with_enum_items(enum_items: &'a [EnumItem]) -> Self {
        Self {
            short: None,
            long: None,
            value_name: "VALUE",
            description: "",
            description_ex: &[],
            group: &OPTIONS,
            environment: None,
            enum_items,
            default: None,
            run: None,
            err_hint: None,
            _type: PhantomData,
        }
    }

    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub const fn long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    pub const fn value_name(mut self, value_name: &'a str) -> Self {
        self.value_name = value_name;
        self
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'a [&'a str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn group(mut self, group: &'a Group) -> Self {
        self.group = group;
        self
    }

    pub const fn environment(mut self, environment: &'a str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn default(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    #[allow(dead_code)]
    pub const fn run(mut self, run: OptRun) -> Self {
        self.run = Some(run);
        self
    }

    #[allow(dead_code)]
    pub const fn err_hint(mut self, err_hint: OptErrHint) -> Self {
        self.err_hint = Some(err_hint);
        self
    }

    pub const fn done(self) -> Opt<'a, T> {
        TypedArg::new(OptArg {
            short: self.short,
            long: self.long,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            run: self.run,
            err_hint: self.err_hint,
            value: Some(Value {
                name: self.value_name,
                default: self.default,
                enum_items: self.enum_items,
                parse: T::parse_value,
            }),
        })
    }
}

pub struct PosBuilder<'a, T> {
    name: &'a str,
    required: bool,
    multiple: bool,
    negative: bool,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
    enum_items: &'a [EnumItem<'a>],
    default: Option<&'a str>,
    _type: PhantomData<T>,
}

impl<'a, T: ParseValue<OsStr> + 'a> PosBuilder<'a, T> {
    pub const fn new() -> Self {
        Self {
            name: "VALUE",
            required: false,
            multiple: false,
            negative: true,
            description: "",
            description_ex: &[],
            group: &ARGUMENTS,
            environment: None,
            enum_items: &[],
            default: None,
            _type: PhantomData,
        }
    }

    pub const fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    pub const fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    #[allow(dead_code)]
    pub const fn description_ex(mut self, description_ex: &'a [&'a str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    #[allow(dead_code)]
    pub const fn group(mut self, group: &'a Group) -> Self {
        self.group = group;
        self
    }

    #[allow(dead_code)]
    pub const fn environment(mut self, environment: &'a str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn default(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    pub const fn done(self) -> Pos<'a, T> {
        TypedArg::new(PosArg {
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            required: self.required,
            multiple: self.multiple,
            negative: self.negative,
            value: Value {
                name: self.name,
                default: self.default,
                enum_items: self.enum_items,
                parse: T::parse_value,
            },
        })
    }
}
