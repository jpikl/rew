use crate::cli2::Arg;
use crate::cli2::ArgValue;
use crate::cli2::COMMANDS;
use crate::cli2::Command;
use crate::cli2::Group;
use crate::cli2::Id;
use crate::cli2::OPTIONS;
use crate::cli2::POSITIONALS;
use crate::cli2::TypedArg;
use crate::cli2::Value;
use std::marker::PhantomData;

pub struct CommandBuilder {
    id: Id,
    name: &'static str,
    description: &'static [&'static str],
    version: Option<&'static str>,
    group: &'static Group,
    args: &'static [&'static dyn Arg],
}

impl CommandBuilder {
    pub const fn new() -> Self {
        Self {
            id: Id::zero(),
            name: "",
            description: &[],
            version: None,
            group: &COMMANDS,
            args: &[],
        }
    }

    pub const fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub const fn description(mut self, description: &'static [&'static str]) -> Self {
        self.description = description;
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

    pub const fn args(mut self, args: &'static [&'static dyn Arg]) -> Self {
        self.args = args;
        self
    }

    pub const fn done(self) -> Command {
        Command {
            id: self.id,
            name: self.name,
            description: self.description,
            version: self.version,
            group: self.group,
            args: self.args,
        }
    }
}

pub struct FlagBuilder {
    id: Id,
    short: Option<char>,
    long: Option<&'static str>,
    description: &'static [&'static str],
    group: &'static Group,
    environment: Option<&'static str>,
}

impl FlagBuilder {
    pub const fn new() -> Self {
        Self {
            id: Id::zero(),
            short: None,
            long: None,
            description: &[],
            group: &OPTIONS,
            environment: None,
        }
    }

    pub const fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub const fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    pub const fn description(mut self, description: &'static [&'static str]) -> Self {
        self.description = description;
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

    pub const fn done(self) -> TypedArg<bool> {
        TypedArg {
            _type: PhantomData {},
            id: self.id,
            short: self.short,
            long: self.long,
            description: self.description,
            group: self.group,
            environment: self.environment,
            multiple: false,
            required: false,
            value: None,
        }
    }
}

pub struct OptBuilder<T> {
    id: Id,
    short: Option<char>,
    long: Option<&'static str>,
    value_name: &'static str,
    description: &'static [&'static str],
    group: &'static Group,
    environment: Option<&'static str>,
    _type: PhantomData<T>,
}

impl<T: Value> OptBuilder<T> {
    pub const fn new() -> Self {
        Self {
            id: Id::zero(),
            short: None,
            long: None,
            value_name: "VALUE",
            description: &[],
            group: &OPTIONS,
            environment: None,
            _type: PhantomData,
        }
    }

    pub const fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
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

    pub const fn description(mut self, description: &'static [&'static str]) -> Self {
        self.description = description;
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

    pub const fn done(self) -> TypedArg<T> {
        TypedArg {
            _type: PhantomData {},
            id: self.id,
            short: self.short,
            long: self.long,
            description: self.description,
            group: self.group,
            environment: self.environment,
            multiple: false,
            required: false,
            value: Some(ArgValue {
                name: self.value_name,
                default: None,
                enum_items: &[],
                accept: None,
            }),
        }
    }
}

pub struct PosBuilder<T> {
    id: Id,
    name: &'static str,
    description: &'static [&'static str],
    group: &'static Group,
    required: bool,
    multiple: bool,
    _type: PhantomData<T>,
}

impl<T: Value> PosBuilder<T> {
    pub const fn new() -> Self {
        Self {
            id: Id::zero(),
            name: "VALUE",
            description: &[],
            group: &POSITIONALS,
            required: false,
            multiple: false,
            _type: PhantomData,
        }
    }

    pub const fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub const fn description(mut self, description: &'static [&'static str]) -> Self {
        self.description = description;
        self
    }

    pub const fn group(mut self, group: &'static Group) -> Self {
        self.group = group;
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

    pub const fn done(self) -> TypedArg<T> {
        TypedArg {
            _type: PhantomData {},
            id: self.id,
            short: None,
            long: None,
            description: self.description,
            group: self.group,
            environment: None,
            multiple: self.multiple,
            required: self.required,
            value: Some(ArgValue {
                name: self.name,
                default: None,
                enum_items: &[],
                accept: None,
            }),
        }
    }
}
