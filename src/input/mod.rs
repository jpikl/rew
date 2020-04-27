pub use args::ArgsInput;
use std::io::Result;
use std::path::Path;
pub use stdin::StdinInput;

mod args;
mod stdin;

// TODO make as enum
pub trait Input {
    fn next(&mut self) -> Result<Option<&Path>>;
}
