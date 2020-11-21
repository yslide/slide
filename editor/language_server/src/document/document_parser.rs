//! Module `document_parser` describes how slide programs should be parsed from a document.

#[derive(Debug)]
pub struct DocumentParser(regex::Regex);

impl DocumentParser {
    pub fn validate(&self) -> Result<(), String> {
        // First capturing group is the entire match.
        // Second capturing group should be the slide program.
        if self.0.captures_len() != 2 {
            Err(format!(
                "must have exactly one explicit capturing group for a slide program; found {}",
                self.0.captures_len() - 1
            ))
        } else {
            Ok(())
        }
    }
}

impl std::cmp::PartialEq for DocumentParser {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str().eq(other.0.as_str())
    }
}

impl From<regex::Regex> for DocumentParser {
    fn from(re: regex::Regex) -> Self {
        Self(re)
    }
}
