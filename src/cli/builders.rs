use super::ARGUMENTS;
use super::ArgValue;
use super::COMMANDS;
use super::Enum;
use super::EnumItem;
use super::OPTIONS;
use super::ParseRawArg;
use super::args::Args;
use super::args::TypedArg;
use super::types::Command;
use super::types::Group;
use super::types::OptArg;
use super::types::OptArgKind;
use super::types::PosArg;
use std::marker::PhantomData;

pub struct CommandBuilder {
    name: &'static str,
    description: &'static str,
    description_ex: &'static [&'static str],
    version: Option<&'static str>,
    group: &'static Group,
    options: &'static [OptArg],
    positionals: &'static [PosArg],
    commands: &'static [Command],
    run: fn(Args) -> anyhow::Result<()>,
}

impl CommandBuilder {
    pub const fn new() -> Self {
        Self {
            name: "",
            description: "",
            description_ex: &[],
            version: None,
            group: &COMMANDS,
            options: &[],
            positionals: &[],
            commands: &[],
            run: |_| unimplemented!("command not implemented"),
        }
    }

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'static [&'static str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    pub const fn group(mut self, group: &'static Group) -> Self {
        self.group = group;
        self
    }

    pub const fn options(mut self, options: &'static [OptArg]) -> Self {
        self.options = options;
        self
    }

    pub const fn positionals(mut self, positionals: &'static [PosArg]) -> Self {
        self.positionals = positionals;
        self
    }

    pub const fn commands(mut self, commands: &'static [Command]) -> Self {
        self.commands = commands;
        self
    }

    pub const fn run(mut self, run: fn(Args) -> anyhow::Result<()>) -> Self {
        self.run = run;
        self
    }

    pub const fn done(self) -> Command {
        Command {
            name: self.name,
            description: self.description,
            description_ex: self.description_ex,
            version: self.version,
            group: self.group,
            options: self.options,
            positionals: self.positionals,
            commands: self.commands,
            run: self.run,
        }
    }
}

pub struct FlagBuilder {
    id: &'static str,
    short: Option<char>,
    long: Option<&'static str>,
    description: &'static str,
    description_ex: &'static [&'static str],
    group: &'static Group,
    environment: Option<&'static str>,
}

impl FlagBuilder {
    pub const fn new(id: &'static str) -> Self {
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

    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'static [&'static str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn group(mut self, group: &'static Group) -> Self {
        self.group = group;
        self
    }

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn done(self) -> TypedArg<OptArg, bool> {
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

pub struct OptBuilder<T> {
    id: &'static str,
    short: Option<char>,
    long: Option<&'static str>,
    value_name: &'static str,
    description: &'static str,
    description_ex: &'static [&'static str],
    group: &'static Group,
    environment: Option<&'static str>,
    enum_items: &'static [EnumItem],
    default: Option<&'static str>,
    _type: PhantomData<T>,
}

impl<T: ParseRawArg + Enum + 'static> OptBuilder<T> {
    pub const fn new_enum(id: &'static str) -> Self {
        Self::new_with_enum_items(id, T::ENUM_ITEMS)
    }
}

impl<T: ParseRawArg + 'static> OptBuilder<T> {
    pub const fn new(id: &'static str) -> Self {
        Self::new_with_enum_items(id, &[])
    }

    const fn new_with_enum_items(id: &'static str, enum_items: &'static [EnumItem]) -> Self {
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

    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    pub const fn value_name(mut self, value_name: &'static str) -> Self {
        self.value_name = value_name;
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'static [&'static str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn group(mut self, group: &'static Group) -> Self {
        self.group = group;
        self
    }

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    pub const fn done(self) -> TypedArg<OptArg, T> {
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

pub struct PosBuilder<T> {
    id: &'static str,
    name: &'static str,
    required: bool,
    description: &'static str,
    description_ex: &'static [&'static str],
    group: &'static Group,
    environment: Option<&'static str>,
    enum_items: &'static [EnumItem],
    default: Option<&'static str>,
    _type: PhantomData<T>,
}

impl<T: ParseRawArg + 'static> PosBuilder<T> {
    pub const fn new(id: &'static str) -> Self {
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

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn description_ex(mut self, description_ex: &'static [&'static str]) -> Self {
        self.description_ex = description_ex;
        self
    }

    pub const fn group(mut self, group: &'static Group) -> Self {
        self.group = group;
        self
    }

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    pub const fn done(self) -> TypedArg<PosArg, T> {
        TypedArg::new(PosArg {
            id: self.id,
            description: self.description,
            description_ex: self.description_ex,
            group: self.group,
            environment: self.environment,
            required: self.required,
            value: ArgValue {
                name: self.name,
                default: None,
                enum_items: self.enum_items,
                parse: T::parse_raw_arg,
            },
        })
    }
}
