mod fn_rules;

use super::rule::*;
use super::unbuilt_rule::UnbuiltRule;
use crate::utils::indent;
use fn_rules::*;

use core::fmt;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;

macro_rules! define_rules {
    ($($kind:ident: $def:expr)*) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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
           UnwrapExplicitParens: S("(_a) -> _a")
         UnwrapExplicitBrackets: S("[_a] -> _a")
                            Add: F(add)
                       Subtract: F(subtract)
                       Multiply: F(multiply)
                         Divide: F(divide)
                         Modulo: F(modulo)
                   Exponentiate: F(exponentiate)
                         Posate: F(posate)
                         Negate: F(negate)
           MultiplicateIdentity: S("_a * 1 -> _a")
               AdditiveIdentity: S("_a + 0 -> _a")
                AdditiveInverse: S("_a - _a -> 0")
            SubtractiveIdentity: S("_a - 0 -> _a")
               ReorderConstants: S("#a + $b -> $b + #a")
             DistributeNegation: S("-(_a - _b) -> _b - _a")
            FoldNegatedAddition: S("_a + -_b -> _a - _b")
}

impl PartialOrd for RuleName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuleName {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

/// Set of unbuilt rules.
pub struct RuleSet {
    rules: HashMap<RuleName, UnbuiltRule>,
    custom_rules: Vec<UnbuiltRule>,
}

impl Default for RuleSet {
    /// Constructs the default rule set.
    fn default() -> Self {
        Self {
            rules: get_all_rules(),
            custom_rules: Vec::new(),
        }
    }
}

impl RuleSet {
    /// Creates a list of `Rules`s from the unbuilt rule set.
    pub fn build(&self) -> Result<Vec<Rule>, BuildRuleErrors> {
        // Order rules deterministically -- first order by name, then add custom rules.
        let mut unbuilt_named_rules: Vec<(&RuleName, &UnbuiltRule)> = self.rules.iter().collect();
        unbuilt_named_rules.sort_by(|&(a, _), &(b, _)| a.cmp(b));
        let all_rules: Vec<_> = unbuilt_named_rules
            .into_iter()
            .map(|(rn, rule)| (Some(rn), rule))
            .chain(self.custom_rules.iter().map(|r| (None, r)))
            .collect();

        let num_rules = all_rules.iter().clone().fold(0, |sum, (_, ur)| match ur {
            // Building a string rule actually generates two versions:
            // 1. The "raw" form of the string rule
            // 2. A version of the (1) boostrapped with a set of rules, possibly including (1)
            //    itself.
            UnbuiltRule::S(_) => sum + 2,
            _ => sum + 1,
        });

        let mut built_rules = Vec::with_capacity(num_rules);
        let mut errors: Vec<Box<dyn Error>> = Vec::new();
        let bootstrapping_rules = Self::get_bootstrapping_rules();
        let bootstrap_blacklist = Self::get_boostrap_blacklist();
        for (rule_name, unbuilt_rule) in all_rules.into_iter() {
            match unbuilt_rule {
                UnbuiltRule::S(s) => {
                    let pm = PatternMap::from_str(s);
                    if let Some(err) = pm.validate() {
                        errors.push(err.into());
                        continue;
                    }

                    if !bootstrap_blacklist.contains(&rule_name.copied()) {
                        let bootstrapped_pm = pm.bootstrap(&bootstrapping_rules);
                        built_rules.push(Rule::PatternMap(bootstrapped_pm));
                    }

                    built_rules.push(Rule::PatternMap(pm));
                }
                UnbuiltRule::F(f) => built_rules.push(Rule::from_fn(*f)),
            }
        }

        if !errors.is_empty() {
            return Err(BuildRuleErrors { errors });
        }

        Ok(built_rules)
    }

    /// Remove a named rule from the rule set.
    pub fn remove(&mut self, rule: RuleName) {
        self.rules.remove(&rule);
    }

    /// Insert a custom unbuilt rule into the rule set.
    #[allow(unused)] // Used in testing. TODO: enable
    fn insert_custom<T: Into<UnbuiltRule>>(&mut self, rule: T) {
        self.custom_rules.push(rule.into());
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

    /// Retrieves a set of rules to be excluded from being bootstrapped.
    fn get_boostrap_blacklist() -> HashSet<Option<RuleName>> {
        vec![
            Some(RuleName::UnwrapExplicitParens),
            Some(RuleName::UnwrapExplicitBrackets),
        ]
        .into_iter()
        .collect()
    }
}

#[derive(Debug)]
pub struct BuildRuleErrors {
    errors: Vec<Box<dyn Error>>,
}

impl fmt::Display for BuildRuleErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errors = self
            .errors
            .iter()
            .enumerate()
            .map(|(i, r)| format!("({}) {}", i + 1, r.to_string()))
            .map(|s| indent(s, 4))
            .collect::<Vec<_>>()
            .join("\n");

        write!(
            f,
            "Failed to build rules with {} errors.\n{}",
            self.errors.len(),
            errors
        )
    }
}

impl Error for BuildRuleErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rules() {
        let rule_set = RuleSet::default();
        let built_rules = rule_set.build().unwrap();

        assert!(built_rules
            .into_iter()
            .any(|s| s.to_string() == "_a + 0 -> _a"));
    }

    #[test]
    fn fail_build_rules() {
        let mut rule_set = RuleSet::default();
        rule_set.insert_custom("_a -> _b");
        rule_set.insert_custom("$a -> $a - _c");
        let err = rule_set.build().expect_err("");

        assert_eq!(
            err.to_string(),
            r##"Failed to build rules with 2 errors.
    (1) Could not resolve pattern map
        "_a -> _b"
    Specifically, source "_a" is missing pattern(s) "_b" present in target "_b"
    (2) Could not resolve pattern map
        "$a -> $a - _c"
    Specifically, source "$a" is missing pattern(s) "_c" present in target "$a - _c""##
        );
    }
}
