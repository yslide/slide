use crate::evaluator_rules::RuleName;

/// A context for evaluating a slide program.
pub struct EvaluatorContext {
    /// Rules that should not be included in the evaluation of an expression.
    pub(crate) rule_denylist: Vec<RuleName>,

    /// Whether an expression should always be flattened before it is further evaluated.
    pub(crate) always_flatten: bool,
}

impl Default for EvaluatorContext {
    fn default() -> Self {
        Self {
            rule_denylist: vec![],
            always_flatten: true,
        }
    }
}

impl EvaluatorContext {
    /// Set rules to exclude in evaluation.
    pub fn with_denylist<T>(mut self, rule_denylist: T) -> Self
    where
        T: Into<Vec<RuleName>>,
    {
        self.rule_denylist = rule_denylist.into();
        self
    }

    /// Whether expressions should always be flattened during evaluation.
    pub fn always_flatten(mut self, flatten: bool) -> Self {
        self.always_flatten = flatten;
        self
    }
}
