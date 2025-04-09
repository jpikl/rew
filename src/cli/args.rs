use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::marker::PhantomData;

pub trait Arg {
    fn id(&self) -> &str;
}

pub struct TypedArg<A, T> {
    pub arg: A,
    _type: PhantomData<T>,
}

impl<A, T> TypedArg<A, T> {
    pub const fn new(arg: A) -> Self {
        Self {
            arg,
            _type: PhantomData,
        }
    }
}

impl<A: Display, T> Display for TypedArg<A, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.arg)
    }
}

pub struct Args {
    values: HashMap<String, Box<dyn Any>>,
}

impl Args {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set<A: Arg>(&mut self, arg: &A, value: Box<dyn Any>) {
        self.values.insert(arg.id().to_owned(), value);
    }

    pub fn get<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> T {
        match self.values.get(arg.arg.id()) {
            Some(value) => value
                .downcast_ref::<T>()
                .expect("mismatched arg type")
                .clone(),
            None => T::default(),
        }
    }

    pub fn get_owned<A: Arg, T: Default + 'static>(&mut self, arg: &TypedArg<A, T>) -> T {
        match self.values.remove(arg.arg.id()) {
            Some(value) => *value.downcast::<T>().expect("mismatched arg type"),
            None => T::default(),
        }
    }
}

pub type ParseArgResult = Result<Box<dyn Any>, String>;
pub type ParseOsArgFn = fn(Cow<OsStr>) -> ParseArgResult;

pub trait ParseArg {
    fn parse_arg(raw_value: Cow<str>) -> ParseArgResult;
}

pub trait ParseOsArg {
    fn parse_os_arg(raw_value: Cow<OsStr>) -> ParseArgResult;
}

impl ParseOsArg for OsString {
    fn parse_os_arg(raw_value: Cow<OsStr>) -> ParseArgResult {
        Ok(Box::new(raw_value.into_owned()))
    }
}

impl ParseArg for String {
    fn parse_arg(raw_value: Cow<str>) -> ParseArgResult {
        Ok(Box::new(raw_value.into_owned()))
    }
}

impl<A: ParseArg> ParseOsArg for A {
    fn parse_os_arg(raw_value: Cow<OsStr>) -> ParseArgResult {
        match raw_value {
            Cow::Borrowed(raw_value) => {
                if let Some(value) = raw_value.to_str() {
                    return A::parse_arg(Cow::Borrowed(value));
                }
            }
            Cow::Owned(raw_value) => {
                if let Ok(value) = raw_value.into_string() {
                    return A::parse_arg(Cow::Owned(value));
                }
            }
        }
        Err("value is not valid UTF-8 string".into())
    }
}

#[macro_export]
macro_rules! default_parse_arg {
    ($type:path) => {
        impl $crate::cli::ParseArg for $type {
            fn parse_arg(raw_value: std::borrow::Cow<str>) -> $crate::cli::ParseArgResult {
                match raw_value.parse::<$type>() {
                    Ok(value) => Ok(Box::new(value)),
                    Err(err) => Err(err.to_string()),
                }
            }
        }
    };
}

default_parse_arg!(bool);
default_parse_arg!(char);
default_parse_arg!(i8);
default_parse_arg!(i16);
default_parse_arg!(i32);
default_parse_arg!(i64);
default_parse_arg!(i128);
default_parse_arg!(isize);
default_parse_arg!(u8);
default_parse_arg!(u16);
default_parse_arg!(u32);
default_parse_arg!(u64);
default_parse_arg!(u128);
default_parse_arg!(usize);
default_parse_arg!(f32);
default_parse_arg!(f64);

pub trait Enum {
    fn enum_items() -> &'static [EnumItem];
}

pub struct EnumItem {
    pub name: &'static str,
    pub description: &'static [&'static str],
}

#[macro_export]
macro_rules! impl_enum {
    ($type:path, {$($value:ident: {name: $name:literal, description: [$($description:literal),*,], }),*,}) => {
        impl $crate::cli::Enum for $type {
            fn enum_items() -> &'static [EnumItem] {
                &[$( EnumItem {name: $name, description: &[$($description),*]}, )*]
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$value => write!(f, "{}", $name),)*
                }
            }
        }

        impl ParseArg for $type {
            fn parse_arg(raw_value: Cow<str>) -> ParseArgResult {
                match raw_value.as_ref() {
                    $($name => Ok(Box::new(Self::$value)),)*
                    _ => Err(format!("Invalid enum value: {}", raw_value)),
                }
            }
        }
    };
}
