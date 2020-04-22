use crate::pattern::range::Range;
use crate::pattern::substitution::Substitution;

mod apply;
mod parse;

#[derive(Debug, PartialEq)]
pub enum Transform {
    Substring(Range),
    SubstringReverse(Range),
    ReplaceFirst(Substitution),
    ReplaceAll(Substitution),
    Trim,
    Lowercase,
    Uppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
    Default(String),
}
