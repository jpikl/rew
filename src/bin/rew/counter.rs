use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct GlobalGenerator {
    value: u32,
    step: u32,
}

impl GlobalGenerator {
    pub fn new(inital: u32, step: u32) -> Self {
        Self {
            value: inital,
            step,
        }
    }

    pub fn next(&mut self) -> u32 {
        let value = self.value;
        self.value += self.step;
        value
    }
}

pub struct LocalGenerator {
    values: HashMap<Option<PathBuf>, u32>,
    inital: u32,
    step: u32,
}

impl LocalGenerator {
    pub fn new(inital: u32, step: u32) -> Self {
        Self {
            values: HashMap::new(),
            inital,
            step,
        }
    }

    pub fn next(&mut self, value: &str) -> u32 {
        let key = Path::new(value).parent().map(Path::to_path_buf);
        if let Some(value) = self.values.get_mut(&key) {
            *value += self.step;
            *value
        } else {
            self.values.insert(key, self.inital);
            self.inital
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
