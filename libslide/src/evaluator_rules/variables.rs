pub fn is_var_char(c: char) -> bool {
    c.is_alphabetic() || c == '$' || c == '#' || c == '_'
}

#[derive(Debug, PartialEq, Eq)]
/// Represents the pattern a variable name matches.
///
/// `VariablePattern`s support bitwise comparison:
///
/// ```rust, ignore
/// assert!(Any      & Const != 0); // an Any pattern can match a Const pattern
/// assert!(Variable & Const != 0); // a Variable pattern cannot match a Const pattern
/// ```
///
pub enum VariablePattern {
    Variable = 0b01,
    Const = 0b10,
    Any = 0b11,
}

impl VariablePattern {
    pub fn from_name(name: &str) -> Self {
        match name.chars().next().unwrap() {
            '_' => Self::Any,
            '#' => Self::Const,
            _ => Self::Variable,
        }
    }
}

impl std::ops::BitAnd for VariablePattern {
    type Output = u8;

    fn bitand(self, rhs: Self) -> Self::Output {
        self as u8 & rhs as u8
    }
}

#[cfg(test)]
mod tests {
    use super::VariablePattern;

    #[test]
    fn from_name() {
        use VariablePattern::*;
        let cases = vec![
            ("$a", Variable),
            ("a", Variable),
            ("#a", Const),
            ("_a", Any),
        ];
        for (name, variant) in cases.into_iter() {
            assert_eq!(VariablePattern::from_name(&name.to_string()), variant);
        }
    }

    #[test]
    fn bitxor() {
        use VariablePattern::*;
        let cases = vec![
            (Const, Const, false),
            (Const, Variable, true),
            (Const, Any, false),
            (Variable, Const, true),
            (Variable, Variable, false),
            (Variable, Any, false),
            (Any, Const, false),
            (Any, Variable, false),
            (Any, Any, false),
        ];
        for (l, r, expectation) in cases.into_iter() {
            assert_eq!(l & r == 0, expectation)
        }
    }
}
