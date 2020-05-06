#[derive(Copy, Clone)]
pub struct ScannerOptions {
    _is_var_char: fn(char) -> bool,
}

impl Default for ScannerOptions {
    fn default() -> Self {
        Self {
            _is_var_char: char::is_alphabetic,
        }
    }
}

impl ScannerOptions {
    pub fn set_is_var_char(mut self, f: fn(char) -> bool) -> Self {
        self._is_var_char = f;
        self
    }

    /// Whether a character is part of a variable.
    pub fn is_var_char(self, c: char) -> bool {
        (self._is_var_char)(c)
    }
}
