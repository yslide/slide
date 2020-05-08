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
    None = 0,
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
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u8 & rhs as u8).into()
    }
}

impl From<u8> for VariablePattern {
    fn from(n: u8) -> Self {
        use VariablePattern::*;
        match n {
            _ if n == None as u8 => None,
            _ if n == Variable as u8 => Variable,
            _ if n == Const as u8 => Const,
            _ if n == Any as u8 => Any,
            _ => unreachable!(),
        }
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
            (Const, Const, Const),
            (Const, Variable, None),
            (Const, Any, Const),
            (Variable, Const, None),
            (Variable, Variable, Variable),
            (Variable, Any, Variable),
            (Any, Const, Const),
            (Any, Variable, Variable),
            (Any, Any, Any),
        ];
        for (l, r, res) in cases.into_iter() {
            assert!(l & r == res)
        }
    }
}
