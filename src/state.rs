use crate::pattern::EvalContext;
use std::path::Path;

pub struct State {
    local_counter: u32, // TODO separate counter for each dirname
    global_counter: u32,
}

impl State {
    pub fn new() -> Self {
        Self {
            local_counter: 0,
            global_counter: 0,
        }
    }

    pub fn get_eval_context<'a>(&'a mut self, path: &'a Path) -> StateEvalContext {
        StateEvalContext {
            path,
            local_counter: &mut self.local_counter,
            local_counter_incremented: false,
            global_counter: &mut self.global_counter,
            global_counter_incremented: false,
        }
    }
}

pub struct StateEvalContext<'a> {
    path: &'a Path,
    local_counter: &'a mut u32,
    local_counter_incremented: bool,
    global_counter: &'a mut u32,
    global_counter_incremented: bool,
}

impl<'a> EvalContext for StateEvalContext<'a> {
    fn path(&self) -> &Path {
        self.path
    }

    fn local_counter(&mut self) -> u32 {
        if !self.local_counter_incremented {
            *self.local_counter += 1;
            self.local_counter_incremented = true;
        }
        *self.local_counter
    }

    fn global_counter(&mut self) -> u32 {
        if !self.global_counter_incremented {
            *self.global_counter += 1;
            self.global_counter_incremented = true;
        }
        *self.global_counter
    }

    fn capture_group(&self, _index: usize) -> Option<&str> {
        None
    }
}
