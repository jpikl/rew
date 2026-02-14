use crate::cli2::parse::ParseError;
use crate::cli2::parse::Parser;
use crate::cli2::parse::RawArgs;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Id(pub i32);

impl Id {
    pub const fn zero() -> Id {
        Id(0)
    }
}

trait ParseTrue: Default {
    fn parse_true() -> Self {
        Self::default()
    }
}

impl ParseTrue for bool {
    fn parse_true() -> Self {
        true
    }
}

impl ParseTrue for u8 {}
impl ParseTrue for u16 {}
impl ParseTrue for u32 {}
impl ParseTrue for u64 {}
impl ParseTrue for u128 {}
impl ParseTrue for usize {}
impl ParseTrue for i8 {}
impl ParseTrue for i16 {}
impl ParseTrue for i32 {}
impl ParseTrue for i64 {}
impl ParseTrue for i128 {}
impl ParseTrue for isize {}
impl ParseTrue for char {}
impl ParseTrue for String {}
impl ParseTrue for OsString {}

pub trait Parse: Default {
    fn parse(val: OsString) -> Result<Self, (Box<dyn std::error::Error>, OsString)>;
}

impl Parse for OsString {
    fn parse(val: OsString) -> Result<Self, (Box<dyn std::error::Error>, OsString)> {
        Ok(val)
    }
}

impl Parse for String {
    fn parse(val: OsString) -> Result<Self, (Box<dyn std::error::Error>, OsString)> {
        match val.into_string() {
            Ok(str) => Ok(str),
            Err(val) => Err(("Value is not valid UTF-8".into(), val)),
        }
    }
}

macro_rules! impl_parse {
    ($type:path) => {
        impl Parse for $type {
            fn parse(raw: OsString) -> Result<Self, (Box<dyn std::error::Error>, OsString)> {
                match raw.to_str() {
                    Some(str) => match <$type as std::str::FromStr>::from_str(str) {
                        Ok(val) => Ok(val),
                        Err(err) => Err((err.into(), raw)),
                    },
                    None => Err(("Value is not valid UTF-8".into(), raw)),
                }
            }
        }
    };
}

impl_parse!(bool);
impl_parse!(u8);
impl_parse!(u16);
impl_parse!(u32);
impl_parse!(u64);
impl_parse!(u128);
impl_parse!(usize);
impl_parse!(i8);
impl_parse!(i16);
impl_parse!(i32);
impl_parse!(i64);
impl_parse!(i128);
impl_parse!(isize);
impl_parse!(char);

pub trait Value: Parse + ParseTrue + Display + Debug {}

impl<T: Parse + ParseTrue + Display + Debug> Value for T {}

pub struct Placeholder<T: 'static> {
    arg: &'static TypedArg<T>,
    value: Option<T>,
}

impl<T: Value> Placeholder<T> {
    fn arg(&self) -> &'static dyn Arg {
        self.arg as &(dyn Arg)
    }

    pub fn value(&mut self) -> Result<T, ParseError> {
        if let Some(value) = self.value.take() {
            return Ok(value);
        }
        if self.arg.required {
            if self.arg.is_option() {
                return Err(ParseError::MissingOptionValue(self.arg()));
            } else {
                return Err(ParseError::MissingArgument(self.arg()));
            }
        }
        if let Some(env_key) = self.arg.environment {
            if let Some(env_val) = std::env::var_os(env_key) {
                return match T::parse(env_val) {
                    Ok(val) => Ok(val),
                    Err((err, val)) => {
                        if self.arg.is_option() {
                            Err(ParseError::InvalidOptionValue(self.arg(), val, err))
                        } else {
                            Err(ParseError::InvalidArgumentValue(self.arg(), val, err))
                        }
                    }
                };
            }
            if self.arg.is_option() {
                return Err(ParseError::MissingOptionValue(self.arg()));
            } else {
                return Err(ParseError::MissingArgument(self.arg()));
            }
        }
        Ok(T::default())
    }
}

pub trait ArgSetter {
    fn id(&self) -> Id;
    fn set(&mut self);
    fn set_from(&mut self, val: OsString) -> Result<(), ParseError>;
}

impl<T: Value> ArgSetter for Placeholder<T> {
    fn id(&self) -> Id {
        self.arg.id
    }

    fn set(&mut self) {
        self.value.replace(T::parse_true());
    }

    fn set_from(&mut self, val: OsString) -> Result<(), ParseError> {
        match T::parse(val) {
            Ok(val) => {
                self.value.replace(val);
                Ok(())
            }
            Err((err, val)) => {
                if self.arg.is_option() {
                    Err(ParseError::InvalidOptionValue(self.arg(), val, err))
                } else {
                    Err(ParseError::InvalidArgumentValue(self.arg(), val, err))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedArg<T> {
    pub _type: PhantomData<T>,
    pub id: Id,
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub description: &'static [&'static str],
    pub group: &'static Group,
    pub environment: Option<&'static str>,
    pub value: Option<ArgValue>,
    pub required: bool,
    pub multiple: bool,
}

impl<T> TypedArg<T> {
    pub fn placeholder(&'static self) -> Placeholder<T> {
        Placeholder { arg: self, value: None }
    }
}

pub trait Arg: Display + Debug {
    fn id(&self) -> Id;
    fn short(&self) -> Option<char>;
    fn long(&self) -> Option<&str>;
    fn description(&self) -> &[&str];
    fn group(&self) -> &Group;
    fn environment(&self) -> Option<&str>;
    fn value(&self) -> Option<&ArgValue>;
    fn multple(&self) -> bool;

    fn is_option(&self) -> bool {
        self.short().is_some() || self.long().is_some()
    }

    fn is_positional(&self) -> bool {
        !self.is_option() && self.value().is_some()
    }
}

impl<T> Display for TypedArg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(short) = self.short {
            write!(f, "-{}", short)?;
        }
        if self.short.is_some() && self.long.is_some() {
            write!(f, ", ")?;
        }
        if let Some(long) = self.long {
            write!(f, "--{}", long)?;
        }
        if self.short.is_some() && self.long.is_some() {
            write!(f, " ")?;
        }
        if let Some(value) = &self.value {
            write!(f, "<{}>", value.name)?;
        }
        Ok(())
    }
}

impl<T: Debug> Arg for TypedArg<T> {
    fn id(&self) -> Id {
        self.id
    }

    fn short(&self) -> Option<char> {
        self.short
    }

    fn long(&self) -> Option<&str> {
        self.long
    }

    fn description(&self) -> &[&str] {
        self.description
    }

    fn group(&self) -> &Group {
        self.group
    }

    fn environment(&self) -> Option<&str> {
        self.environment
    }

    fn value(&self) -> Option<&ArgValue> {
        self.value.as_ref()
    }

    fn multple(&self) -> bool {
        self.multiple
    }
}

#[derive(Clone, Debug)]
pub struct ArgValue {
    pub name: &'static str,
    pub default: Option<&'static str>,
    pub enum_items: &'static [EnumItem],
    pub accept: Option<fn(val: Cow<'_, OsStr>) -> bool>,
}

#[derive(Clone, Debug)]
pub struct EnumItem {
    pub name: &'static str,
    pub description: &'static [&'static str],
}

#[derive(Clone, Debug)]
pub struct Group {
    pub name: &'static str,
    pub description: &'static [&'static str],
}

pub const OPTIONS: Group = Group {
    name: "Options",
    description: &[],
};

pub const POSITIONALS: Group = Group {
    name: "Arguments",
    description: &[],
};

pub const COMMANDS: Group = Group {
    name: "Commands",
    description: &[],
};

#[derive(Clone, Debug)]
pub struct Command {
    pub id: Id,
    pub name: &'static str,
    pub description: &'static [&'static str],
    pub version: Option<&'static str>,
    pub group: &'static Group,
    pub args: &'static [&'static dyn Arg],
}

#[derive(Clone, Debug)]
pub struct CommandAlias {
    pub name: &'static str,
    pub command: &'static Command,
    pub args: &'static [&'static str],
}

impl Command {
    pub fn parse(&'static self, args: RawArgs) -> Parser {
        Parser::new(self, args)
    }
}

#[macro_export]
macro_rules! define_ids {
    ($start:expr, $step:expr, { $($id:ident),* $(,)? }) => {
        define_ids!(@impl $start, $step, 0, $($id),*);
    };

    (@impl $start:expr, $step:expr, $count:expr) => {};

    (@impl $start:expr, $step:expr, $count:expr, $id:ident $(, $($rest:ident),*)?) => {
        pub const $id: $crate::cli2::Id = $crate::cli2::Id($start.0 + $count * $step);
        define_ids!(@impl $start, $step, $count + 1 $(, $($rest),*)?);
    };
}
