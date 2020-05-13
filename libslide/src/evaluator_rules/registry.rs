mod fn_rules;

use super::rule::Rule;
use super::unbuilt_rule::UnbuiltRule;
use fn_rules::*;

use std::collections::HashMap;

macro_rules! define_rules {
    ($($kind:ident: $def:expr)*) => {
        #[derive(Clone, Copy, Hash, PartialEq, Eq)]
        pub enum RuleName {
            $($kind,)*
        }

        fn get_all_rules() -> HashMap<RuleName, UnbuiltRule> {
            use RuleName::*;
            use UnbuiltRule::*;
            [$(($kind, $def),)*].iter().cloned().collect()
        }
    };
}

define_rules! {
                            Add: F(add)
                       Subtract: F(subtract)
                       Multiply: F(multiply)
                         Divide: F(divide)
                         Modulo: F(modulo)
                   Exponentiate: F(exponentiate)
                         Posate: F(posate)
                         Negate: F(negate)
               AdditiveIdentity: S("_a + 0 -> _a")
               ReorderConstants: S("#a + $b -> $b + #a")
             DistributeNegation: S("-(_a - _b) -> _b - _a")
                   UnwrapParens: M(&["($a) -> $a", "(#a) -> #a"])
                   UnwrapBraces: M(&["[$a] -> $a", "[#a] -> #a"])
}

/// Set of unbuilt rules.
pub struct RuleSet {
    rules: HashMap<RuleName, UnbuiltRule>,
}

impl Default for RuleSet {
    /// Constructs the default rule set.
    fn default() -> Self {
        Self {
            rules: get_all_rules(),
        }
    }
}

impl RuleSet {
    /// Creates a list of `Rules`s from the unbuilt rule set.
    pub fn build(&self) -> Vec<Rule> {
        let num_rules = self.rules.values().fold(0, |sum, ur| match ur {
            UnbuiltRule::M(m) => sum + m.len(),
            _ => sum + 1,
        });

        let mut built_rules = Vec::with_capacity(num_rules);
        for unbuilt_rule in self.rules.values() {
            match unbuilt_rule {
                UnbuiltRule::S(s) => built_rules.push(Rule::from_str(s)),
                UnbuiltRule::M(multiple) => {
                    for s in multiple.iter() {
                        built_rules.push(Rule::from_str(s));
                    }
                }
                UnbuiltRule::F(f) => built_rules.push(Rule::from_fn(*f)),
            }
        }
        built_rules
    }

    /// Remove a rule from the rule set.
    pub fn remove(&mut self, rule: RuleName) {
        self.rules.remove(&rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rules() {
        let rule_set = RuleSet::default();
        let built_rules = rule_set.build();

        assert!(built_rules.iter().any(|r| r.to_string() == "_a + 0 -> _a"));
    }
}
