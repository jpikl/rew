use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    values: HashMap<Option<PathBuf>, u32>,
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
        let key = Path::new(value).parent().map(Path::to_path_buf);
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

    #[test]
    fn config_default() {
        let config = Config::default();
        assert_eq!(config.init, INIT_DEFAULT);
        assert_eq!(config.step, STEP_DEFAULT);
    }

    #[test]
    fn config_from_str() {
        assert_eq!(
            Config::from_str("12"),
            Ok(Config {
                init: 12,
                step: STEP_DEFAULT
            })
        );

        assert_eq!(Config::from_str("12:34"), Ok(Config { init: 12, step: 34 }));
    }

    #[test]
    fn config_from_str_errors() {
        assert_eq!(Config::from_str(""), Err(INIT_ERROR));
        assert_eq!(Config::from_str(":"), Err(INIT_ERROR));
        assert_eq!(Config::from_str(":34"), Err(INIT_ERROR));
        assert_eq!(Config::from_str(":cd"), Err(INIT_ERROR));
        assert_eq!(Config::from_str("12:"), Err(STEP_ERROR));
        assert_eq!(Config::from_str("12:cd"), Err(STEP_ERROR));
        assert_eq!(Config::from_str("ab"), Err(INIT_ERROR));
        assert_eq!(Config::from_str("ab:"), Err(INIT_ERROR));
        assert_eq!(Config::from_str("ab:34"), Err(INIT_ERROR));
        assert_eq!(Config::from_str("ab:cd"), Err(INIT_ERROR));
    }

    #[test]
    fn global_from_config() {
        assert_eq!(
            GlobalGenerator::new(12, 34),
            GlobalGenerator::from(&Config { init: 12, step: 34 })
        );
    }

    #[test]
    fn global_from_zero_per_one() {
        let mut counter = GlobalGenerator::new(0, 1);
        assert_eq!(counter.next(), 0);
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.next(), 2);
    }

    #[test]
    fn global_from_one_per_ten() {
        let mut counter = GlobalGenerator::new(1, 10);
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.next(), 11);
        assert_eq!(counter.next(), 21);
    }

    #[test]
    fn local_from_config() {
        assert_eq!(
            LocalGenerator::new(12, 34),
            LocalGenerator::from(&Config { init: 12, step: 34 })
        );
    }

    #[test]
    fn local_from_zero_per_one() {
        let path_1 = "dir/subdir/file.ext";
        let path_2 = "dir/subdir";
        let path_3 = "dir";
        let mut counter = LocalGenerator::new(0, 1);

        assert_eq!(counter.next(path_1), 0);
        assert_eq!(counter.next(path_1), 1);
        assert_eq!(counter.next(path_1), 2);

        assert_eq!(counter.next(path_2), 0);
        assert_eq!(counter.next(path_3), 0);

        assert_eq!(counter.next(path_2), 1);
        assert_eq!(counter.next(path_3), 1);

        assert_eq!(counter.next(path_2), 2);
        assert_eq!(counter.next(path_3), 2);
    }

    #[test]
    fn local_from_one_per_ten() {
        let path_1 = "dir/subdir/file.ext";
        let path_2 = "dir/subdir";
        let path_3 = "dir";
        let mut counter = LocalGenerator::new(1, 10);

        assert_eq!(counter.next(path_1), 1);
        assert_eq!(counter.next(path_1), 11);
        assert_eq!(counter.next(path_1), 21);

        assert_eq!(counter.next(path_2), 1);
        assert_eq!(counter.next(path_3), 1);

        assert_eq!(counter.next(path_2), 11);
        assert_eq!(counter.next(path_3), 11);

        assert_eq!(counter.next(path_2), 21);
        assert_eq!(counter.next(path_3), 21);
    }
}
