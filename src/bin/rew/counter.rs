use std::collections::HashMap;
use std::str::FromStr;

use num_traits::PrimInt;

use crate::pattern::path;

const INIT_ERROR: &str = "Invalid init value";
const STEP_ERROR: &str = "Invalid step value";

pub trait Value: PrimInt + FromStr {}

impl<T: PrimInt + FromStr> Value for T {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Config<T: Value> {
    pub init: T,
    pub step: T,
}

impl<T: Value> Default for Config<T> {
    fn default() -> Self {
        Self {
            init: T::one(),
            step: T::one(),
        }
    }
}

impl<T: Value> FromStr for Config<T> {
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
            let step = Config::default().step;

            Ok(Self { init, step })
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct GlobalGenerator<T> {
    value: T,
    step: T,
}

impl<T: Value> GlobalGenerator<T> {
    pub fn new(init: T, step: T) -> Self {
        Self { value: init, step }
    }

    pub fn next(&mut self) -> T {
        let value = self.value;
        self.value = self.value.add(self.step);
        value
    }
}

impl<T: Value> From<&Config<T>> for GlobalGenerator<T> {
    fn from(config: &Config<T>) -> Self {
        Self::new(config.init, config.step)
    }
}

#[derive(PartialEq, Debug)]
pub struct LocalGenerator<T: Value> {
    values: HashMap<String, T>,
    init: T,
    step: T,
}

impl<T: Value> From<&Config<T>> for LocalGenerator<T> {
    fn from(config: &Config<T>) -> Self {
        Self::new(config.init, config.step)
    }
}

impl<T: Value> LocalGenerator<T> {
    pub fn new(init: T, step: T) -> Self {
        Self {
            values: HashMap::new(),
            init,
            step,
        }
    }

    pub fn next(&mut self, value: &str) -> T {
        let key = match path::get_parent_directory(value.to_string()) {
            Ok(parent) => path::normalize(&parent).unwrap_or_default(),
            Err(_) => String::new(),
        };
        if let Some(value) = self.values.get_mut(&key) {
            *value = value.add(self.step);
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
    type Value = u32;

    mod config {
        use super::*;

        #[test]
        fn default() {
            let config = Config::<Value>::default();
            assert_eq!(config.init, 1);
            assert_eq!(config.step, 1);
        }

        mod from_str {
            use test_case::test_case;

            use super::*;

            #[test_case("",      INIT_ERROR ; "empty")]
            #[test_case(":",     INIT_ERROR ; "separator")]
            #[test_case(":34",   INIT_ERROR ; "separator number")]
            #[test_case(":cd",   INIT_ERROR ; "separator string")]
            #[test_case("12:",   STEP_ERROR ; "number separator")]
            #[test_case("12:cd", STEP_ERROR ; "number separator string")]
            #[test_case("ab",    INIT_ERROR ; "string")]
            #[test_case("ab:",   INIT_ERROR ; "string separator")]
            #[test_case("ab:34", INIT_ERROR ; "string separator number")]
            #[test_case("ab:cd", INIT_ERROR ; "string separator string")]
            fn err(input: &str, error: &str) {
                assert_eq!(Config::<Value>::from_str(input), Err(error));
            }

            #[test_case("12",    12, 1  ; "init")]
            #[test_case("12:34", 12, 34 ; "init and step")]
            fn ok(input: &str, init: Value, step: Value) {
                assert_eq!(Config::from_str(input), Ok(Config { init, step }));
            }
        }
    }

    mod global_generator {
        use test_case::test_case;

        use super::*;

        #[test]
        fn from_config() {
            assert_eq!(
                GlobalGenerator::new(12, 34),
                GlobalGenerator::from(&Config { init: 12, step: 34 })
            );
        }

        #[test_case(0, 1,  0, 0  ; "0:1 iteration 1")]
        #[test_case(0, 1,  1, 1  ; "0:1 iteration 2")]
        #[test_case(0, 1,  2, 2  ; "0:1 iteration 3")]
        #[test_case(1, 10, 0, 1  ; "1:10 iteration 1")]
        #[test_case(1, 10, 1, 11 ; "1:10 iteration 2")]
        #[test_case(1, 10, 2, 21 ; "1:10 iteration 3")]
        fn next(init: Value, step: Value, index: usize, result: Value) {
            let mut counter = GlobalGenerator::new(init, step);
            for _ in 0..index {
                counter.next();
            }
            assert_eq!(counter.next(), result);
        }
    }

    mod local_generator {
        use test_case::test_case;

        use super::*;

        const A: &str = "a";
        const AX: &str = "a/b/..";
        const B: &str = "a/b";
        const C: &str = "a/b/c";
        const CX: &str = "./a/b/c";

        #[test]
        fn from_config() {
            assert_eq!(
                LocalGenerator::new(12, 34),
                LocalGenerator::from(&Config { init: 12, step: 34 })
            );
        }

        #[test_case(0, 1,  &[],                       C,  0  ; "0:1 iteration 1")]
        #[test_case(0, 1,  &[C],                      C,  1  ; "0:1 iteration 2")]
        #[test_case(0, 1,  &[C, C],                   C,  2  ; "0:1 iteration 3")]
        #[test_case(0, 1,  &[C, C, C],                B,  0  ; "0:1 iteration 4")]
        #[test_case(0, 1,  &[C, C, C, B],             A,  0  ; "0:1 iteration 5")]
        #[test_case(0, 1,  &[C, C, C, B, A],          B,  1  ; "0:1 iteration 6")]
        #[test_case(0, 1,  &[C, C, C, B, A, B],       B,  2  ; "0:1 iteration 7")]
        #[test_case(0, 1,  &[C, C, C, B, A, B, B],    A,  1  ; "0:1 iteration 8")]
        #[test_case(0, 1,  &[C, C, C, B, A, B, B, A], A,  2  ; "0:1 iteration 9")]
        #[test_case(1, 10, &[],                       C,  1  ; "1:10 iteration 1")]
        #[test_case(1, 10, &[C],                      C,  11 ; "1:10 iteration 2")]
        #[test_case(1, 10, &[C, C],                   C,  21 ; "1:10 iteration 3")]
        #[test_case(1, 10, &[C, C, C],                B,  1  ; "1:10 iteration 4")]
        #[test_case(1, 10, &[C, C, C, B],             A,  1  ; "1:10 iteration 5")]
        #[test_case(1, 10, &[C, C, C, B, A],          B,  11 ; "1:10 iteration 6")]
        #[test_case(1, 10, &[C, C, C, B, A, B],       B,  21 ; "1:10 iteration 7")]
        #[test_case(1, 10, &[C, C, C, B, A, B, B],    A,  11 ; "1:10 iteration 8")]
        #[test_case(1, 10, &[C, C, C, B, A, B, B, A], A,  21 ; "1:10 iteration 9")]
        #[test_case(0, 1, &[],                        C,  0  ; "normalize dirs 1")]
        #[test_case(0, 1, &[C],                       CX, 1  ; "normalize dirs 2")]
        #[test_case(0, 1, &[C, CX],                   A,  0  ; "normalize dirs 3")]
        #[test_case(0, 1, &[C, CX, A],                AX, 1  ; "normalize dirs 4")]
        fn next(init: Value, step: Value, prev_paths: &[&str], next_path: &str, result: Value) {
            let mut counter = LocalGenerator::new(init, step);
            for prev_path in prev_paths {
                counter.next(prev_path);
            }
            assert_eq!(counter.next(next_path), result);
        }
    }
}
