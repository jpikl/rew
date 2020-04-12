use crate::pattern::parser::PatternItem;
use crate::pattern::variable::Variable;
use crate::pattern::Pattern;

impl Pattern {
    pub fn uses_local_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::LocalCounter)
    }

    pub fn uses_global_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::GlobalCounter)
    }

    pub fn uses_regex_captures(&self) -> bool {
        self.uses_variable(|variable| matches!(variable, Variable::RegexCapture(_)))
    }

    fn uses_variable<F: Fn(&Variable) -> bool>(&self, test: F) -> bool {
        self.items.iter().any(|item| {
            if let PatternItem::Expression { variable, .. } = &item.value {
                test(&variable.value)
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_local_counter() {
        assert_eq!(Pattern::parse("{f}").unwrap().uses_local_counter(), false);
        assert_eq!(Pattern::parse("{c}").unwrap().uses_local_counter(), true);
    }

    #[test]
    fn uses_global_counter() {
        assert_eq!(Pattern::parse("{f}").unwrap().uses_global_counter(), false);
        assert_eq!(Pattern::parse("{C}").unwrap().uses_global_counter(), true);
    }

    #[test]
    fn uses_regex_captures() {
        assert_eq!(Pattern::parse("{f}").unwrap().uses_regex_captures(), false);
        assert_eq!(Pattern::parse("{1}").unwrap().uses_regex_captures(), true);
    }
}
