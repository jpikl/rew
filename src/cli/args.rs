use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

pub trait Arg {
    fn id(&self) -> &str;
    fn default_value(&self) -> Option<Box<dyn Any>>;
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
        self.arg.fmt(f)
    }
}

impl<A: Arg, T: Default + Clone + 'static> TypedArg<A, T> {
    pub fn unbox(&self, value: Option<&Box<dyn Any>>) -> T {
        match value {
            Some(value) => value.downcast_ref::<T>().expect("mismatched arg type").clone(),
            None => self.default_value(),
        }
    }

    pub fn unbox_opt(&self, value: Option<&Box<dyn Any>>) -> Option<T> {
        value.map(|value| value.downcast_ref::<T>().expect("mismatched arg type").clone())
    }

    pub fn unbox_ref<'a>(&self, value: Option<&'a Box<dyn Any>>) -> Cow<'a, T> {
        match value {
            Some(value) => Cow::Borrowed(value.downcast_ref::<T>().expect("mismatched arg type")),
            None => Cow::Owned(self.default_value()),
        }
    }

    pub fn unbox_opt_ref<'a>(&self, value: Option<&'a Box<dyn Any>>) -> Option<&'a T> {
        value.map(|value| value.downcast_ref::<T>().expect("mismatched arg type"))
    }

    pub fn default_value(&self) -> T {
        match self.arg.default_value() {
            Some(value) => *value.downcast::<T>().expect("mismatched arg type"),
            None => T::default(),
        }
    }
}

#[derive(Debug)]
pub struct ArgUsage {
    pub arg_id: String,
    pub source: ArgSource,
    pub value: Box<dyn Any>,
}

#[derive(Debug)]
pub enum ArgSource {
    ShortOption,
    LongOption,
    Positional,
    Environment,
}

impl ArgUsage {
    pub fn new<A: Arg>(arg: &A, kind: ArgSource, value: Box<dyn Any>) -> Self {
        Self {
            arg_id: arg.id().to_string(),
            source: kind,
            value,
        }
    }
}

#[derive(Debug)]
pub struct Args {
    usages: Vec<ArgUsage>,
}

impl Args {
    pub fn new(usages: Vec<ArgUsage>) -> Self {
        Self { usages }
    }

    pub fn usages(&self) -> &[ArgUsage] {
        &self.usages
    }

    pub fn has<A: Arg>(&self, arg: &A) -> bool {
        self.usages.iter().any(|usage| usage.arg_id == arg.id())
    }

    pub fn get<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> T {
        arg.unbox(self.get_value(&arg.arg))
    }

    pub fn get_opt<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> Option<T> {
        arg.unbox_opt(self.get_value(&arg.arg))
    }

    pub fn get_ref<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> Cow<T> {
        arg.unbox_ref(self.get_value(&arg.arg))
    }

    #[allow(dead_code)]
    pub fn get_opt_ref<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> Option<&T> {
        arg.unbox_opt_ref(self.get_value(&arg.arg))
    }

    fn get_value<A: Arg>(&self, arg: &A) -> Option<&Box<dyn Any>> {
        self.usages
            .iter()
            .rev()
            .find(|usage| usage.arg_id == arg.id())
            .map(|usage| &usage.value)
    }

    #[allow(dead_code)]
    pub fn iter<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> impl Iterator<Item = T> {
        self.iter_values(&arg.arg).map(|value| arg.unbox(Some(value)))
    }

    pub fn iter_ref<A: Arg, T: Default + Clone + 'static>(&self, arg: &TypedArg<A, T>) -> impl Iterator<Item = Cow<T>> {
        self.iter_values(&arg.arg).map(|value| arg.unbox_ref(Some(value)))
    }

    fn iter_values<A: Arg>(&self, arg: &A) -> impl Iterator<Item = &Box<dyn Any>> {
        self.usages
            .iter()
            .filter(|usage| usage.arg_id == arg.id())
            .map(|usage| &usage.value)
    }
}
