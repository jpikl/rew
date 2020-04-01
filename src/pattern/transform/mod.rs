use crate::pattern::range::Range;
use crate::pattern::substitution::Substitution;

mod eval;
mod parse;

#[derive(Debug, PartialEq)]
pub enum Transform {
    Substring(Range),
    SubstringFromEnd(Range),
    ReplaceFirst(Substitution),
    ReplaceAll(Substitution),
    Trim,
    Lowercase,
    Uppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(Vec<char>),
    RightPad(Vec<char>),
}
