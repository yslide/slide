//! Module `source_map` represents a [`Document`](super::Document)'s text source and provides
//! method to convert between offsets and positions in the source.

use parking_lot::RwLock;
use std::collections::HashMap;
use tower_lsp::lsp_types::Position;

/// Describes a source text, providing mappings between byte offsets and line/column positions in
/// the source text.
pub struct SourceMap {
    /// The lines in the source. Each line is represented by the byte offset of the start of the
    /// line and the length of the line.
    lines: Vec<(
        /* offset of line start */ usize,
        /* line length */ usize,
    )>,
    /// A cache of line/column positions -> byte offset mappings.
    cache_position2offset: RwLock<HashMap<WrappedPosition, usize>>,
    /// A cache of byte offset -> line/column positions mappings.
    cache_offset2position: RwLock<HashMap<usize, WrappedPosition>>,
}

impl SourceMap {
    /// Creates a new `SourceFile`.
    pub fn new(source: &str) -> Self {
        let mut offset = 0;
        let mut lines: Vec<_> = LinesWithEndings::from(source)
            .into_iter()
            .map(|line| {
                let line_offset_and_width = (offset, line.len() - 1);
                offset += line.len();
                line_offset_and_width
            })
            .collect();
        if let Some(l) = lines.last_mut() {
            l.1 += 1;
        }

        Self {
            lines,
            cache_offset2position: Default::default(),
            cache_position2offset: Default::default(),
        }
    }

    /// Returns the byte offset corresponding to a line/column position in the source.
    pub fn to_offset(&self, position: Position) -> usize {
        let position = WrappedPosition::from(position);
        if !self.cache_position2offset.read().contains_key(&position) {
            let (line_start_offset, _) = self.lines[position.line];
            let offset = line_start_offset + position.co;
            self.cache_position2offset.write().insert(position, offset);
        }

        *self.cache_position2offset.read().get(&position).unwrap()
    }

    /// Returns the line/column position corresponding to a byte offset in the source.
    pub fn to_position(&self, offset: usize) -> Position {
        if !self.cache_offset2position.read().contains_key(&offset) {
            let line = self
                .lines
                .binary_search_by(|&(line_start_offset, line_len)| {
                    if offset < line_start_offset {
                        std::cmp::Ordering::Greater
                    } else if offset > line_start_offset + line_len {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .expect("Offset not found!");
            let (line_start_offset, _) = self.lines[line];
            let position = WrappedPosition::new(line, offset - line_start_offset);
            self.cache_offset2position.write().insert(offset, position);
        }

        (*self.cache_offset2position.read().get(&offset).unwrap()).into()
    }
}

/// Iterator yielding every line in a string. The line includes the newline character.
// TODO: use `str#split_inclusive` after it's made stable (https://github.com/rust-lang/rust/issues/72360)
pub struct LinesWithEndings<'a> {
    input: &'a str,
}

impl<'a> LinesWithEndings<'a> {
    pub fn from(input: &'a str) -> LinesWithEndings<'a> {
        LinesWithEndings { input }
    }
}

impl<'a> Iterator for LinesWithEndings<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.input.is_empty() {
            return None;
        }
        let split = self
            .input
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or_else(|| self.input.len());
        let (line, rest) = self.input.split_at(split);
        self.input = rest;
        Some(line)
    }
}

/// Shim for the LSP `Position` interface for use in a `SourceMap`.
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct WrappedPosition {
    line: usize,
    co: usize,
}

impl WrappedPosition {
    fn new(line: usize, co: usize) -> Self {
        Self { line, co }
    }
}

impl From<WrappedPosition> for Position {
    fn from(pos: WrappedPosition) -> Position {
        Position::new(pos.line as u64, pos.co as u64)
    }
}

impl From<Position> for WrappedPosition {
    fn from(pos: Position) -> WrappedPosition {
        WrappedPosition::new(pos.line as usize, pos.character as usize)
    }
}

#[cfg(test)]
mod test {
    use super::SourceMap;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn source_map() {
        let source = r"Hello
this
is some

text";
        let source_map = SourceMap::new(source);
        for &(ch, offset, (line, co)) in &[
            ("H", 0, (0, 0)),
            ("e", 1, (0, 1)),
            ("l", 2, (0, 2)),
            ("l", 3, (0, 3)),
            ("o", 4, (0, 4)),
            ("\n", 5, (0, 5)),
            ("t", 6, (1, 0)),
            ("h", 7, (1, 1)),
            ("i", 8, (1, 2)),
            ("s", 9, (1, 3)),
            ("\n", 10, (1, 4)),
            ("i", 11, (2, 0)),
            ("s", 12, (2, 1)),
            (" ", 13, (2, 2)),
            ("s", 14, (2, 3)),
            ("o", 15, (2, 4)),
            ("m", 16, (2, 5)),
            ("e", 17, (2, 6)),
            ("\n", 18, (2, 7)),
            ("\n", 19, (3, 0)),
            ("t", 20, (4, 0)),
            ("e", 21, (4, 1)),
            ("x", 22, (4, 2)),
            ("t", 23, (4, 3)),
        ] {
            let position = Position::new(line, co);
            assert_eq!(&source[offset..offset + 1], ch);
            assert_eq!(source_map.to_position(offset), position);
            assert_eq!(source_map.to_offset(position), offset);
        }
    }
}
