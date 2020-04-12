mod eval;
mod parse;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    FullDirname,
    ParentDirname,
    FullPath,
    LocalCounter,
    GlobalCounter,
    RegexCapture(usize),
    Uuid,
}
