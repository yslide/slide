use crate::evaluator_rules::RuleName;

pub struct EvaluatorContext {
    pub(crate) rule_blacklist: Vec<RuleName>,
}

impl Default for EvaluatorContext {
    fn default() -> Self {
        Self {
            rule_blacklist: vec![],
        }
    }
}

impl EvaluatorContext {
    pub fn with_blacklist<T>(mut self, rule_blacklist: T) -> Self
    where
        T: Into<Vec<RuleName>>,
    {
        self.rule_blacklist = rule_blacklist.into();
        self
    }
}
