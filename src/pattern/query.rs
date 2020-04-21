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
    use crate::pattern::lexer::Parsed;

    #[test]
    fn uses_none() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::Filename),
            transforms: Vec::new(),
        })];
        let pattern = Pattern::new(items);
        assert_eq!(pattern.uses_local_counter(), false);
        assert_eq!(pattern.uses_global_counter(), false);
        assert_eq!(pattern.uses_regex_captures(), false);
    }

    #[test]
    fn uses_local_counter() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::LocalCounter),
            transforms: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_local_counter(), true);
    }

    #[test]
    fn uses_global_counter() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::GlobalCounter),
            transforms: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_global_counter(), true);
    }

    #[test]
    fn uses_global_regex_captures() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::RegexCapture(1)),
            transforms: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_regex_captures(), true);
    }
}
