use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

pub trait Arg {
    fn key(&self) -> Cow<str>;
}

pub struct TypedArg<Arg, Type> {
    pub arg: Arg,
    _type: PhantomData<Type>,
}

impl<Arg, Type> TypedArg<Arg, Type> {
    pub const fn new(arg: Arg) -> Self {
        Self {
            arg,
            _type: PhantomData,
        }
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
        self.values.insert(arg.key().into_owned(), value);
    }

    pub fn get<A: Arg, T: ParseArg + Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> T {
        match self.values.get(arg.arg.key().as_ref()) {
            Some(value) => value
                .downcast_ref::<T>()
                .expect("mismatched arg type")
                .clone(),
            None => T::default(),
        }
    }

    pub fn get_owned<A: Arg, T: ParseArg + Default + 'static>(
        &mut self,
        arg: &TypedArg<A, T>,
    ) -> T {
        match self.values.remove(arg.arg.key().as_ref()) {
            Some(value) => *value.downcast::<T>().expect("mismatched arg type"),
            None => T::default(),
        }
    }
}

pub type ParseArgResult = Result<Box<dyn Any>, String>;
pub type ParseArgFn = fn(Cow<OsStr>) -> ParseArgResult;

pub trait ParseArg {
    fn parse_arg(raw_value: Cow<OsStr>) -> ParseArgResult;
}

trait DefaultParseArg {}

impl DefaultParseArg for bool {}
impl DefaultParseArg for char {}
impl DefaultParseArg for i8 {}
impl DefaultParseArg for i16 {}
impl DefaultParseArg for i32 {}
impl DefaultParseArg for i64 {}
impl DefaultParseArg for i128 {}
impl DefaultParseArg for isize {}
impl DefaultParseArg for u8 {}
impl DefaultParseArg for u16 {}
impl DefaultParseArg for u32 {}
impl DefaultParseArg for u64 {}
impl DefaultParseArg for u128 {}
impl DefaultParseArg for usize {}
impl DefaultParseArg for f32 {}
impl DefaultParseArg for f64 {}

impl ParseArg for OsString {
    fn parse_arg(raw_value: Cow<OsStr>) -> ParseArgResult {
        Ok(Box::new(raw_value.into_owned()))
    }
}

impl ParseArg for String {
    fn parse_arg(raw_value: Cow<OsStr>) -> ParseArgResult {
        match raw_value.into_owned().into_string() {
            Ok(value) => Ok(Box::new(value)),
            Err(_) => Err("value is not valid UTF-8 string".into()),
        }
    }
}

impl<T: FromStr<Err = impl Display> + DefaultParseArg + 'static> ParseArg for T {
    fn parse_arg(raw_value: Cow<OsStr>) -> ParseArgResult {
        if let Some(value) = raw_value.to_str() {
            match value.parse::<T>() {
                Ok(value) => Ok(Box::new(value)),
                Err(err) => Err(err.to_string()),
            }
        } else {
            Err("value is not valid UTF-8 string".into())
        }
    }
}
