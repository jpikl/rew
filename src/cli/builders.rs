use super::args::Args;
use super::args::ParseArg;
use super::args::TypedArg;
use super::types::Command;
use super::types::CommandGroup;
use super::types::OptArg;
use super::types::OptKind;
use super::types::PosArg;
use std::marker::PhantomData;

pub struct CommandBuilder {
    name: &'static str,
    description: &'static str,
    version: &'static str,
    group: Option<&'static CommandGroup>,
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
            version: "",
            group: None,
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

    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    pub const fn group(mut self, group: &'static CommandGroup) -> Self {
        self.group = Some(group);
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
    short: char,
    long: &'static str,
    description: &'static str,
    environment: Option<&'static str>,
}

impl FlagBuilder {
    pub const fn new() -> Self {
        Self {
            short: '\0',
            long: "",
            description: "",
            environment: None,
        }
    }

    pub const fn short(mut self, short: char) -> Self {
        self.short = short;
        self
    }

    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = long;
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn done(self) -> TypedArg<OptArg, bool> {
        TypedArg::new(OptArg {
            short: self.short,
            long: self.long,
            description: self.description,
            environment: self.environment,
            kind: OptKind::Flag,
        })
    }
}

pub struct OptBuilder<Type> {
    short: char,
    long: &'static str,
    value_name: &'static str,
    description: &'static str,
    environment: Option<&'static str>,
    default: Option<&'static str>,
    _type: PhantomData<Type>,
}

impl<Type: ParseArg + 'static> OptBuilder<Type> {
    pub const fn new() -> Self {
        Self {
            short: '\0',
            long: "",
            value_name: "",
            description: "",
            environment: None,
            default: None,
            _type: PhantomData,
        }
    }

    pub const fn short(mut self, short: char) -> Self {
        self.short = short;
        self
    }

    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = long;
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

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    pub const fn done(self) -> TypedArg<OptArg, Type> {
        TypedArg::new(OptArg {
            short: self.short,
            long: self.long,
            description: self.description,
            environment: self.environment,
            kind: OptKind::Value {
                name: self.value_name,
                default: self.default,
                parse: Type::parse_arg,
            },
        })
    }
}

pub struct PosBuilder<Type> {
    name: &'static str,
    required: bool,
    description: &'static str,
    environment: Option<&'static str>,
    _type: PhantomData<Type>,
}

impl<Type: ParseArg + 'static> PosBuilder<Type> {
    pub const fn new() -> Self {
        Self {
            name: "",
            required: false,
            description: "",
            environment: None,
            _type: PhantomData,
        }
    }

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub const fn environment(mut self, environment: &'static str) -> Self {
        self.environment = Some(environment);
        self
    }

    pub const fn done(self) -> TypedArg<PosArg, Type> {
        TypedArg::new(PosArg {
            name: self.name,
            required: self.required,
            description: self.description,
            environment: self.environment,
            parse: Type::parse_arg,
        })
    }
}
