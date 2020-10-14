pub mod mem;
pub mod sys;

use std::io::{BufRead, Write};
use termcolor::WriteColor;

pub trait Input: BufRead {}
pub trait Output: Write + WriteColor {}

impl<T: BufRead> Input for T {}
impl<T: Write + WriteColor> Output for T {}

pub trait Io<'a> {
    type StdinLock: Input;
    type StdoutLock: Output;
    type StderrLock: Output;

    fn stdin(&'a self) -> Self::StdinLock;
    fn stdout(&'a self) -> Self::StdoutLock;
    fn stderr(&'a self) -> Self::StderrLock;
}
