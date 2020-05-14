mod fn_rules;

use super::rule::*;
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
           UnwrapExplicitParens: S("(_a) -> _a")
         UnwrapExplicitBrackets: S("[_a] -> _a")
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
            // Building a string rule actually generates two versions:
            // 1. The "raw" form of the string rule
            // 2. A version of the (1) boostrapped with a set of rules, possibly including (1)
            //    itself.
            UnbuiltRule::S(_) => sum + 2,
            _ => sum + 1,
        });

        let mut built_rules = Vec::with_capacity(num_rules);
        let bootstrapping_rules = Self::get_bootstrapping_rules();
        for unbuilt_rule in self.rules.values() {
            match unbuilt_rule {
                UnbuiltRule::S(s) => {
                    let pm = PatternMap::from_str(s);
                    let bootstrapped_pm = pm.bootstrap(&bootstrapping_rules);
                    built_rules.push(Rule::PatternMap(pm));
                    built_rules.push(Rule::PatternMap(bootstrapped_pm));
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

    /// Retrieves a set of rules to be used in bootstrapping other rules.
    fn get_bootstrapping_rules() -> Vec<Rule> {
        let bootstrapping_rules = [
            RuleName::UnwrapExplicitParens,
            RuleName::UnwrapExplicitBrackets,
        ];
        let rule_set = get_all_rules();

        bootstrapping_rules
            .iter()
            .map(|r| rule_set.get(r).unwrap())
            .map(|r| match r {
                UnbuiltRule::S(s) => Rule::PatternMap(PatternMap::from_str(s)),
                UnbuiltRule::F(f) => Rule::from_fn(*f),
            })
            .collect()
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
