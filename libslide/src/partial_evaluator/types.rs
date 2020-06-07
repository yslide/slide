use crate::evaluator_rules::RuleName;

pub struct EvaluatorContext {
    /// Rules that should not be included in the evaluation of an expression.
    pub(crate) rule_blacklist: Vec<RuleName>,

    /// Whether an expression should always be flattened before it is further evaluated.
    pub(crate) always_flatten: bool,
}

impl Default for EvaluatorContext {
    fn default() -> Self {
        Self {
            rule_blacklist: vec![],
            always_flatten: true,
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

    pub fn always_flatten(mut self, flatten: bool) -> Self {
        self.always_flatten = flatten;
        self
    }
}
