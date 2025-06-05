use crate::cli::SimpleError;
use bstr::BString;
use std::any::Any;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

#[derive(Debug, PartialEq)]
pub struct Value<'a> {
    pub name: &'a str,
    pub default: Option<&'a str>,
    pub enum_items: &'a [EnumItem<'a>],
    pub parse: ParseValueFn<OsStr>,
}

impl Value<'_> {
    pub fn default(&self) -> Option<Box<dyn Any>> {
        self.default
            .map(|value| (self.parse)(OsStr::new(value).into()).expect("unparsable default value"))
    }
}

pub trait Enum<'a> {
    const ENUM_ITEMS: &'a [EnumItem<'a>];
}

#[derive(Debug, PartialEq)]
pub struct EnumItem<'a> {
    pub name: &'a str,
    pub description: &'a [&'a str],
}

#[macro_export]
macro_rules! impl_enum {
    ($type:path, {$($value:ident: $name:literal),*$(,)?}) => {
        impl_enum!($type, {$($value: {name: $name, description: []}),*});
    };

    ($type:path, {$($value:ident: {name: $name:literal, description: [$($description:literal),*$(,)?]$(,)?}),*$(,)?}) => {
        impl<'a> $crate::cli::Enum<'a> for $type {
            const ENUM_ITEMS: &'a [$crate::cli::EnumItem<'a>] = &[$( $crate::cli::EnumItem {name: $name, description: &[$($description),*]},)*];
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$value => write!(f, "{}", $name),)*
                }
            }
        }

        impl $crate::cli::ParseValue<str> for $type {
            fn parse_value(raw_value: Cow<str>) -> $crate::cli::ParseValueResult<String> {
                match raw_value.as_ref() {
                    $($name => Ok(Box::new(Self::$value)),)*
                    _ => Err((raw_value.clone().into(), Self::unexpected_value_err().into())),
                }
            }
        }

        impl $type {
            fn unexpected_value_err() -> impl std::error::Error {
                use $crate::cli::Enum;
                let mut err = String::from("Expected one of [");
                for (i, item) in Self::ENUM_ITEMS.iter().enumerate() {
                    if i > 0 {
                        err.push_str(", ");
                    }
                    err.push_str($crate::cli::QUOTE_START);
                    err.push_str(item.name);
                    err.push_str($crate::cli::QUOTE_END);
                }
                err.push(']');
                $crate::cli::SimpleError::new(err)
            }
        }
    };
}

pub type ParseValueResult<T> = Result<Box<dyn Any>, (T, Box<dyn std::error::Error>)>;
pub type ParseValueFn<T> = fn(Cow<T>) -> ParseValueResult<<T as ToOwned>::Owned>;

pub trait ParseValue<T: ?Sized + ToOwned> {
    fn parse_value(raw_value: Cow<T>) -> ParseValueResult<T::Owned>;
}

impl ParseValue<OsStr> for OsString {
    fn parse_value(raw_value: Cow<OsStr>) -> ParseValueResult<OsString> {
        Ok(Box::new(raw_value.into_owned()))
    }
}

impl ParseValue<OsStr> for BString {
    fn parse_value(raw_value: Cow<OsStr>) -> ParseValueResult<OsString> {
        Ok(Box::new(BString::new(raw_value.into_owned().into_encoded_bytes())))
    }
}

impl ParseValue<str> for String {
    fn parse_value(raw_value: Cow<str>) -> ParseValueResult<String> {
        Ok(Box::new(raw_value.into_owned()))
    }
}

impl<T: ParseValue<str>> ParseValue<OsStr> for T {
    fn parse_value(raw_value: Cow<OsStr>) -> ParseValueResult<OsString> {
        match raw_value {
            Cow::Borrowed(raw_value) => match raw_value.to_str() {
                Some(value) => result_into_os(T::parse_value(value.into())),
                None => invalid_utf_err(raw_value.into()),
            },
            Cow::Owned(raw_value) => match raw_value.into_string() {
                Ok(value) => result_into_os(T::parse_value(value.into())),
                Err(raw_value) => invalid_utf_err(raw_value),
            },
        }
    }
}

fn result_into_os(res: ParseValueResult<String>) -> ParseValueResult<OsString> {
    match res {
        Ok(value) => Ok(value),
        Err((value, err)) => Err((value.into(), err)),
    }
}

fn invalid_utf_err(value: OsString) -> ParseValueResult<OsString> {
    Err((value, SimpleError::new("value is not valid UTF-8 string").into()))
}

#[macro_export]
macro_rules! impl_parse_value {
    ($type:path) => {
        impl $crate::cli::ParseValue<str> for $type {
            fn parse_value(raw_value: std::borrow::Cow<str>) -> $crate::cli::ParseValueResult<String> {
                match raw_value.parse::<$type>() {
                    Ok(value) => Ok(Box::new(value)),
                    Err(err) => Err((raw_value.into(), err.into())),
                }
            }
        }
    };
}

impl_parse_value!(bool);
impl_parse_value!(char);
impl_parse_value!(i8);
impl_parse_value!(i16);
impl_parse_value!(i32);
impl_parse_value!(i64);
impl_parse_value!(i128);
impl_parse_value!(isize);
impl_parse_value!(u8);
impl_parse_value!(u16);
impl_parse_value!(u32);
impl_parse_value!(u64);
impl_parse_value!(u128);
impl_parse_value!(usize);
impl_parse_value!(f32);
impl_parse_value!(f64);
