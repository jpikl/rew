use super::ARGUMENTS;
use super::ArgValue;
use super::COMMANDS;
use super::Enum;
use super::EnumItem;
use super::ErrorKind;
use super::OPTIONS;
use super::ParseOsArg;
use super::args::Args;
use super::args::TypedArg;
use super::types::Command;
use super::types::Group;
use super::types::OptArg;
use super::types::OptArgKind;
use super::types::PosArg;
use std::marker::PhantomData;

pub struct CommandBuilder<'a> {
    name: &'a str,
    description: &'a str,
    description_ex: &'a [&'a str],
    version: Option<&'a str>,
    group: &'a Group<'a>,
    options: &'a [OptArg<'a>],
    positionals: &'a [PosArg<'a>],
    subcommands: &'a [Command<'a>],
    run: fn(Args) -> Result<(), ErrorKind<'a>>,
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
            run: |_| unimplemented!("command not implemented"),
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

    pub const fn options(mut self, options: &'a [OptArg]) -> Self {
        self.options = options;
        self
    }

    pub const fn positionals(mut self, positionals: &'a [PosArg]) -> Self {
        self.positionals = positionals;
        self
    }

    pub const fn subcommands(mut self, subcommands: &'a [Command]) -> Self {
        self.subcommands = subcommands;
        self
    }

    pub const fn run(mut self, run: fn(Args) -> Result<(), ErrorKind<'a>>) -> Self {
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
            run: self.run,
        }
    }
}

pub struct FlagBuilder<'a> {
    id: &'a str,
    short: Option<char>,
    long: Option<&'a str>,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
}

impl<'a> FlagBuilder<'a> {
    pub const fn new(id: &'a str) -> Self {
        Self {
            id,
            short: None,
            long: None,
            description: "",
            description_ex: &[],
            group: &OPTIONS,
            environment: None,
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

    pub const fn done(self) -> TypedArg<OptArg<'a>, bool> {
        TypedArg::new(OptArg {
            id: self.id,
            short: self.short,
            long: self.long,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            kind: OptArgKind::Flag,
        })
    }
}

pub struct OptBuilder<'a, T> {
    id: &'a str,
    short: Option<char>,
    long: Option<&'a str>,
    value_name: &'a str,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
    enum_items: &'a [EnumItem<'a>],
    default: Option<&'a str>,
    _type: PhantomData<T>,
}

impl<'a, T: ParseOsArg + Enum<'a> + 'a> OptBuilder<'a, T> {
    pub const fn new_enum(id: &'a str) -> Self {
        Self::new_with_enum_items(id, T::ENUM_ITEMS)
    }
}

impl<'a, T: ParseOsArg + 'a> OptBuilder<'a, T> {
    pub const fn new(id: &'a str) -> Self {
        Self::new_with_enum_items(id, &[])
    }

    const fn new_with_enum_items(id: &'a str, enum_items: &'a [EnumItem]) -> Self {
        Self {
            id,
            short: None,
            long: None,
            value_name: "VALUE",
            description: "",
            description_ex: &[],
            group: &OPTIONS,
            environment: None,
            enum_items,
            default: None,
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

    pub const fn done(self) -> TypedArg<OptArg<'a>, T> {
        TypedArg::new(OptArg {
            id: self.id,
            short: self.short,
            long: self.long,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            kind: OptArgKind::Value(ArgValue {
                name: self.value_name,
                default: self.default,
                enum_items: self.enum_items,
                parse: T::parse_raw_arg,
            }),
        })
    }
}

pub struct PosBuilder<'a, T> {
    id: &'a str,
    name: &'a str,
    required: bool,
    description: &'a str,
    description_ex: &'a [&'a str],
    group: &'a Group<'a>,
    environment: Option<&'a str>,
    enum_items: &'a [EnumItem<'a>],
    default: Option<&'a str>,
    _type: PhantomData<T>,
}

impl<'a, T: ParseOsArg + 'a> PosBuilder<'a, T> {
    pub const fn new(id: &'a str) -> Self {
        Self {
            id,
            name: "VALUE",
            required: false,
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

    pub const fn done(self) -> TypedArg<PosArg<'a>, T> {
        TypedArg::new(PosArg {
            id: self.id,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            required: self.required,
            value: ArgValue {
                name: self.name,
                default: self.default,
                enum_items: self.enum_items,
                parse: T::parse_raw_arg,
            },
        })
    }
}
