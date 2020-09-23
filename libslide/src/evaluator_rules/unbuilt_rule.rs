use crate::grammar::RcExpr;

/// An unbuilt rule, generally used to express a rule in a human-readable form.
#[derive(Copy, Clone)]
pub enum UnbuiltRule {
    /// An expression-mapping rule.
    ///
    /// A string rule is of the form
    ///
    /// ```text
    /// "<expr> -> <expr>"
    /// ```
    ///
    /// Where <expr> is any expression pattern. An expression pattern is similar to any other
    /// expression, differing only in its pattern matching variables. The form of pattern matching
    /// variables and the expressions they match are as follows:
    ///   
    /// | pattern | matches        |
    /// |:------- |:-------------- |
    /// | #<name> | A constant     |
    /// | $<name> | A variable     |
    /// | _<name> | Any expression |
    ///
    /// To apply a rule, the lhs of the rule is pattern matched on the target expression. If the
    /// matching is sucessful, the rhs of the rule is expanded with the results of the matching.
    ///
    /// For example, the rule
    ///   
    /// ```text
    /// "$a + 0 -> $a"
    /// ```
    ///
    /// Applied on the expression `"x + 0"` would yield `"x"`.
    ///
    /// Note that mapping rules are built as, matched with, and applied on expression parse trees
    /// rather than the string representations of expressions. This ensures rule application is
    /// always exact and deterministic.
    S(&'static str),

    /// Multiple string rules. This should be used by rules that are only fully expressed by
    /// multiple similar transformations.
    M(&'static [&'static str]),

    /// A function rule.
    F(fn(RcExpr) -> Option<RcExpr>),
}

impl From<&'static str> for UnbuiltRule {
    fn from(s: &'static str) -> Self {
        Self::S(s)
    }
}
