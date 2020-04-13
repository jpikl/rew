use crate::pattern::EvalContext;
use regex::Regex;
use std::path::Path;

pub enum RegexTarget {
    Path,
    Filename,
}

pub struct State {
    local_counter: u32, // TODO separate counter for each dirname
    local_counter_enabled: bool,
    global_counter: u32,
    global_counter_enabled: bool,
    regex: Option<Regex>,
    regex_target: RegexTarget,
}

impl State {
    pub fn new() -> Self {
        Self {
            local_counter: 0,
            local_counter_enabled: false,
            global_counter: 0,
            global_counter_enabled: false,
            regex: None,
            regex_target: RegexTarget::Filename,
        }
    }

    pub fn set_local_counter_enabled(&mut self, enabled: bool) {
        self.local_counter_enabled = enabled;
    }

    pub fn set_global_counter_enabled(&mut self, enabled: bool) {
        self.local_counter_enabled = enabled;
    }

    pub fn set_regex(&mut self, regex: Option<Regex>) {
        self.regex = regex;
    }

    pub fn set_regex_target(&mut self, target: RegexTarget) {
        self.regex_target = target;
    }

    pub fn get_eval_context<'a>(&'a mut self, path: &'a Path) -> EvalContext {
        if self.local_counter_enabled {
            self.local_counter += 1
        }
        if self.global_counter_enabled {
            self.global_counter += 1
        }
        EvalContext {
            path,
            local_counter: self.local_counter,
            global_counter: self.global_counter,
            regex_captures: self.get_regex_captures(path),
        }
    }

    fn get_regex_captures<'a>(&self, path: &'a Path) -> Option<regex::Captures<'a>> {
        if let Some(regex) = &self.regex {
            let target = match self.regex_target {
                RegexTarget::Path => Some(path.as_os_str()),
                RegexTarget::Filename => path.file_name(),
            };
            if let Some(value) = target {
                // TODO handle utf error
                regex.captures(&value.to_str().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}
