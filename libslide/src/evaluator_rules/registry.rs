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
    ($($(#[doc = $doc:expr])+ $kind:ident: $def:expr)*) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
        /// Built-in rewrite rule names.
        pub enum RuleName {
            $(
                $(#[doc = $doc])+
                $kind,
            )*
        }

        fn get_all_rules() -> HashMap<RuleName, UnbuiltRule> {
            use RuleName::*;
            use UnbuiltRule::*;
            [$(($kind, $def),)*].iter().cloned().collect()
        }
    };
}

// The order matters... (TODO) we should make that more explicit, or be smarter about it.
define_rules! {
    /// Unwraps parantheses.
    UnwrapExplicitParens: S("(_a) -> _a")
    /// Unwraps brackets.
    UnwrapExplicitBrackets: S("[_a] -> _a")
    /// Binary addition.
    Add: F(add)
    /// Binary subtraction.
    Subtract: F(subtract)
    /// Binary multiplication.
    Multiply: F(multiply)
    /// Binary division.
    Divide: F(divide)
    /// Binary modulo.
    Modulo: F(modulo)
    /// Binary exponentiation.
    Exponentiate: F(exponentiate)
    /// Unary posation.
    Posate: F(posate)
    /// Unary negation.
    Negate: F(negate)
    /// The multiplicative identity `a*1=a`.
    MultiplicateIdentity: S("_a * 1 -> _a")
    /// The additive identity `a+0=a`.
    AdditiveIdentity: S("_a + 0 -> _a")
    /// The additive inverse `a+(-a)=0`.
    AdditiveInverse: S("_a - _a -> 0")
    /// The equivalent additive identity `a-0=a`.
    SubtractiveIdentity: S("_a - 0 -> _a")
    /// The commutative axiom with constants.
    ReorderConstants: S("#a + $b -> $b + #a")
    /// The distributive axiom on negation.
    DistributeNegation: M(&[
        "-(_a - _b) -> _b - _a",
        "_a - (_b - _c) -> _a - _b + _c",
    ])
    /// Collapses the addition of a negation to a subtraction.
    FoldNegatedAddition: S("_a + -_b -> _a - _b")
    /// Collapses the multiplication of a reciprocal to a division.
    FoldDivision: M(&[
        "_a * 1 / _b -> _a / _b",
        "_a * (1 / _b) -> _a / _b",
    ])
    /// Exponentiation axioms.
    FoldExponents: M(&[
        "_a * _a -> _a^2",
        "_a * _a^_b -> _a^(_b + 1)",
        "_a^_b * _a -> _a^(_b + 1)",
        "_a^_b * _a^_c -> _a^(_b + _c)",
        "_a / _a^_b -> _a^(1 - _b)",
        "_a^_b / _a -> _a^(_b - 1)",
        "_a^_b / _a^_c -> _a^(_b - _c)",
    ])
    /// Exponentiation identity `a^0=a`.
    ExponentiativeIdentity: S("_a^0 -> _a")
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
            UnbuiltRule::M(v) => sum + 2 * v.len(),
            _ => sum + 1,
        });

        let mut built_rules = Vec::with_capacity(num_rules);
        let mut errors: Vec<Box<dyn Error>> = Vec::new();
        let bootstrapping_rules = Self::get_bootstrapping_rules();
        let bootstrap_blacklist = Self::get_boostrap_blacklist();
        let mut mk_str_rule =
            |built_rules: &mut Vec<Rule>, rule_name: Option<&RuleName>, rule: &'static str| {
                let pm = PatternMap::from_str(rule);
                if let Err(err) = pm.validate() {
                    errors.push(err.into());
                    return;
                }

                if !bootstrap_blacklist.contains(&rule_name.copied()) {
                    let bootstrapped_pm = pm.bootstrap(&bootstrapping_rules);
                    built_rules.push(Rule::PatternMap(bootstrapped_pm));
                }
                built_rules.push(Rule::PatternMap(pm));
            };

        for (rule_name, unbuilt_rule) in all_rules.into_iter() {
            match unbuilt_rule {
                UnbuiltRule::S(rule) => mk_str_rule(&mut built_rules, rule_name, rule),
                UnbuiltRule::M(rules) => {
                    for rule in rules.iter() {
                        mk_str_rule(&mut built_rules, rule_name, rule);
                    }
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
    pub fn remove(&mut self, rule: &RuleName) {
        self.rules.remove(rule);
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
            .flat_map(|r| match r {
                UnbuiltRule::S(s) => vec![Rule::from_pat_str(s)],
                UnbuiltRule::M(m) => m.iter().map(|s| Rule::from_pat_str(s)).collect(),
                UnbuiltRule::F(f) => vec![Rule::from_fn(*f)],
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

/// Errors that result from an attempt to [build a set of rules](RuleSet::build).
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
