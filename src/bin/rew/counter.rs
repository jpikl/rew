use crate::pattern::path;
use std::collections::HashMap;
use std::str::FromStr;

const INIT_DEFAULT: u32 = 1;
const STEP_DEFAULT: u32 = 1;

const INIT_ERROR: &str = "Invalid init value";
const STEP_ERROR: &str = "Invalid step value";

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Config {
    pub init: u32,
    pub step: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init: INIT_DEFAULT,
            step: STEP_DEFAULT,
        }
    }
}

impl FromStr for Config {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(delimiter_index) = string.find(':') {
            let init_str = &string[0..delimiter_index];
            let init = init_str.parse().map_err(|_| INIT_ERROR)?;

            let step = if delimiter_index < string.len() - 1 {
                let step_str = &string[(delimiter_index + 1)..];
                step_str.parse().map_err(|_| STEP_ERROR)?
            } else {
                return Err(STEP_ERROR);
            };

            Ok(Self { init, step })
        } else {
            let init = string.parse().map_err(|_| INIT_ERROR)?;
            let step = STEP_DEFAULT;

            Ok(Self { init, step })
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct GlobalGenerator {
    value: u32,
    step: u32,
}

impl GlobalGenerator {
    pub fn new(init: u32, step: u32) -> Self {
        Self { value: init, step }
    }

    pub fn next(&mut self) -> u32 {
        let value = self.value;
        self.value += self.step;
        value
    }
}

impl From<&Config> for GlobalGenerator {
    fn from(config: &Config) -> Self {
        Self::new(config.init, config.step)
    }
}

#[derive(PartialEq, Debug)]
pub struct LocalGenerator {
    values: HashMap<String, u32>,
    init: u32,
    step: u32,
}

impl From<&Config> for LocalGenerator {
    fn from(config: &Config) -> Self {
        Self::new(config.init, config.step)
    }
}

impl LocalGenerator {
    pub fn new(init: u32, step: u32) -> Self {
        Self {
            values: HashMap::new(),
            init,
            step,
        }
    }

    pub fn next(&mut self, value: &str) -> u32 {
        let key = match path::get_parent_directory(value.to_string()) {
            Ok(parent) => path::normalize(&parent).unwrap_or_default(),
            Err(_) => String::new(),
        };
        if let Some(value) = self.values.get_mut(&key) {
            *value += self.step;
            *value
        } else {
            self.values.insert(key, self.init);
            self.init
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod config {
        use super::*;

        #[test]
        fn default() {
            let config = Config::default();
            assert_eq!(config.init, INIT_DEFAULT);
            assert_eq!(config.step, STEP_DEFAULT);
        }

        mod from_str {
            use super::*;
            use test_case::test_case;

            #[test_case("", INIT_ERROR; "empty")]
            #[test_case(":", INIT_ERROR; "separator")]
            #[test_case(":34", INIT_ERROR; "separator number")]
            #[test_case(":cd", INIT_ERROR; "separator string")]
            #[test_case("12:", STEP_ERROR; "number separator")]
            #[test_case("12:cd", STEP_ERROR; "number separator string")]
            #[test_case("ab", INIT_ERROR; "string")]
            #[test_case("ab:", INIT_ERROR; "string separator")]
            #[test_case("ab:34", INIT_ERROR; "string separator number")]
            #[test_case("ab:cd", INIT_ERROR; "string separator string")]
            fn err(input: &str, error: &str) {
                assert_eq!(Config::from_str(input), Err(error));
            }

            #[test_case("12", 12, STEP_DEFAULT; "init")]
            #[test_case("12:34", 12, 34; "init and step")]
            fn ok(input: &str, init: u32, step: u32) {
                assert_eq!(Config::from_str(input), Ok(Config { init, step }));
            }
        }
    }

    mod global_generator {
        use super::*;
        use test_case::test_case;

        #[test]
        fn from_config() {
            assert_eq!(
                GlobalGenerator::new(12, 34),
                GlobalGenerator::from(&Config { init: 12, step: 34 })
            );
        }

        #[test_case(0, 1, 0, 0; "0:1 iteration 1")]
        #[test_case(0, 1, 1, 1; "0:1 iteration 2")]
        #[test_case(0, 1, 2, 2; "0:1 iteration 3")]
        #[test_case(1, 10, 0, 1; "1:10 iteration 1")]
        #[test_case(1, 10, 1, 11; "1:10 iteration 2")]
        #[test_case(1, 10, 2, 21; "1:10 iteration 3")]
        fn next(init: u32, step: u32, index: usize, result: u32) {
            let mut counter = GlobalGenerator::new(init, step);
            for _ in 0..index {
                counter.next();
            }
            assert_eq!(counter.next(), result);
        }
    }

    mod local_generator {
        use super::*;
        use test_case::test_case;

        #[test]
        fn from_config() {
            assert_eq!(
                LocalGenerator::new(12, 34),
                LocalGenerator::from(&Config { init: 12, step: 34 })
            );
        }

        #[test_case(0, 1, &[], "a/b/c", 0; "0:1 iteration 1")]
        #[test_case(0, 1, &["a/b/c"], "a/b/c", 1; "0:1 iteration 2")]
        #[test_case(0, 1, &["a/b/c", "a/b/c"], "a/b/c", 2; "0:1 iteration 3")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c"], "a/b", 0; "0:1 iteration 4")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c", "a/b"], "a", 0; "0:1 iteration 5")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a"], "a/b", 1; "0:1 iteration 6")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b"], "a/b", 2; "0:1 iteration 7")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b", "a/b"], "a", 1; "0:1 iteration 8")]
        #[test_case(0, 1, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b", "a/b", "a"], "a", 2; "0:1 iteration 9")]
        #[test_case(1, 10, &[], "a/b/c", 1; "1:10 iteration 1")]
        #[test_case(1, 10, &["a/b/c"], "a/b/c", 11; "1:10 iteration 2")]
        #[test_case(1, 10, &["a/b/c", "a/b/c"], "a/b/c", 21; "1:10 iteration 3")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c"], "a/b", 1; "1:10 iteration 4")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c", "a/b"], "a", 1; "1:10 iteration 5")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a"], "a/b", 11; "1:10 iteration 6")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b"], "a/b", 21; "1:10 iteration 7")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b", "a/b"], "a", 11; "1:10 iteration 8")]
        #[test_case(1, 10, &["a/b/c", "a/b/c", "a/b/c", "a/b", "a", "a/b", "a/b", "a"], "a", 21; "1:10 iteration 9")]
        #[test_case(0, 1, &[], "a/b/c", 0; "normalize dirs 1")]
        #[test_case(0, 1, &["a/b/c"], "./a/b/c", 1; "normalize dirs 2")]
        #[test_case(0, 1, &["a/b/c", "./a/b/c"], "a", 0; "normalize dirs 3")]
        #[test_case(0, 1, &["a/b/c", "./a/b/c", "a"], "a/b/..", 1; "normalize dirs 4")]
        fn next(init: u32, step: u32, prev_paths: &[&str], next_path: &str, result: u32) {
            let mut counter = LocalGenerator::new(init, step);
            for prev_path in prev_paths {
                counter.next(prev_path);
            }
            assert_eq!(counter.next(next_path), result);
        }
    }
}
