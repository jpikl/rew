use crate::pattern::Pattern;
use std::{cmp, env, process};

mod pattern;

fn main() {
    if let Some(raw_pattern) = env::args().nth(1) {
        match Pattern::parse(&raw_pattern) {
            Ok(pattern) => println!("{:#?}", pattern),
            Err(error) => {
                eprintln!("{}", error.typ,);
                if !raw_pattern.is_empty() {
                    println!(
                        "\n{}\n{}{}",
                        raw_pattern,
                        " ".repeat(error.start),
                        "^".repeat(cmp::max(1, error.end - error.start))
                    );
                }
                process::exit(2);
            }
        }
    } else {
        eprintln!("Expected argument");
        process::exit(1);
    }
}
