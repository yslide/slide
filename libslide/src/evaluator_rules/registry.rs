mod fn_rules;

use super::rule::Rule;
use super::unbuilt_rule::UnbuiltRule;

use fn_rules::*;

macro_rules! define_rules {
    ($($kind:ident | $def:expr)*) => {
        // TODO: will allow lint in a future iteration when we support removing rules.
        #[allow(dead_code)]
        pub enum RuleName {
            $($kind,)*
        }

        use UnbuiltRule::*;
        static DEFAULT_RULESET: &[UnbuiltRule] = &[
            $($def,)*
        ];
    };
}

define_rules! {
    Add                         | F(add)
    Subtract                    | F(subtract)
    Multiply                    | F(multiply)
    Divide                      | F(divide)
    Modulo                      | F(modulo)
    Exponentiate                | F(exponentiate)
    Posate                      | F(posate)
    Negate                      | F(negate)
    AdditiveIdentity            | S("_a + 0 -> _a")
    ReorderConstants            | S("#a + $b -> $b + #a")
    DistributeNegation          | S("-(_a - _b) -> _b - _a")
    UnwrapParens                | M(&["($a) -> $a", "(#a) -> #a"])
    UnwrapBraces                | M(&["[$a] -> $a", "[#a] -> #a"])
}

/// Set of unbuilt rules.
pub struct RuleSet {
    rules: Vec<UnbuiltRule>,
}

impl Default for RuleSet {
    /// Constructs the default rule set.
    fn default() -> Self {
        Self {
            rules: DEFAULT_RULESET.to_vec(),
        }
    }
}

impl RuleSet {
    /// Creates a list of `Rules`s from the unbuilt rule set.
    pub fn build(&self) -> Vec<Rule> {
        let num_rules = self.rules.iter().fold(0, |sum, ur| match ur {
            UnbuiltRule::M(m) => sum + m.len(),
            _ => sum + 1,
        });

        let mut built_rules = Vec::with_capacity(num_rules);
        for unbuilt_rule in self.rules.iter() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rules() {
        let rule_set = RuleSet::default();
        let built_rules = rule_set.build();
        let var_plus_zero = &built_rules[RuleName::AdditiveIdentity as usize];

        assert_eq!(var_plus_zero.to_string(), "_a + 0 -> _a");
    }
}
