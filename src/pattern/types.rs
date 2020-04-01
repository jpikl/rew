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

#[derive(Debug, PartialEq)]
pub struct Range {
    pub offset: usize,
    pub length: usize, // Zero length means unlimited
}

#[derive(Debug, PartialEq)]
pub struct Substitution {
    pub value: String,
    pub replacement: String,
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    value: T,
    source_position: usize,
    source_length: usize,
}
