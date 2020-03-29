#[derive(Debug, PartialEq)]
pub struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}

#[derive(Debug, PartialEq)]
pub enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    LocalCounter,
    GlobalCounter,
    CaptureGroup(usize),
    Uuid,
}

#[derive(Debug, PartialEq)]
pub enum Transform {
    Character(usize),
    SubstrFrom(usize),
    SubstrTo(usize),
    SubstrFromTo(usize, usize),
    EndCharacter(usize),
    EndSubstrFrom(usize),
    EndSubstrTo(usize),
    EndSubstrFromTo(usize, usize),
    ReplaceFirst(String, String),
    ReplaceAll(String, String),
    Trim,
    Lowercase,
    Uppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    value: T,
    source_position: usize,
    source_length: usize,
}
