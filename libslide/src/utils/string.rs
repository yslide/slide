pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len).collect()
    }
}

/// Indents all lines of a string with `n` spaces.
pub fn indent<T: Into<String>>(s: T, n: usize) -> String {
    let s: String = s.into();
    let indent = " ".repeat(n);
    s.lines()
        .map(|l| format!("{}{}", indent, l))
        .collect::<Vec<_>>()
        .join("\n")
}
