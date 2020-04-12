use crate::pattern::EvalContext;
use std::path::Path;

pub struct State {
    local_counter: u32, // TODO separate counter for each dirname
    local_counter_enabled: bool,
    global_counter: u32,
    global_counter_enabled: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            local_counter: 0,
            local_counter_enabled: false,
            global_counter: 0,
            global_counter_enabled: false,
        }
    }

    pub fn set_local_counter_enabled(&mut self, enabled: bool) {
        self.local_counter_enabled = enabled;
    }

    pub fn set_global_counter_enabled(&mut self, enabled: bool) {
        self.local_counter_enabled = enabled;
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
            regex_captures: None,
        }
    }
}
