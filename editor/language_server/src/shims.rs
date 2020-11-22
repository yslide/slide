//! Module `shims` converts between slide and LSP types.

use tower_lsp::lsp_types::*;

// https://docs.rs/wast/25.0.2/src/wast/ast/token.rs.html#24-36
// TODO: batch this or provide a offset mapping.
pub fn to_position(offset: usize, source: &str) -> Position {
    let mut cur = 0;
    // Use split_terminator instead of lines so that if there is a `\r`,
    // it is included in the offset calculation. The `+1` values below
    // account for the `\n`.
    for (i, line) in source.split_terminator('\n').enumerate() {
        if cur + line.len() + 1 > offset {
            return Position::new(i as u64, (offset - cur) as u64);
        }
        cur += line.len() + 1;
    }
    Position::new(source.lines().count() as u64, 0)
}

pub fn to_offset(position: &Position, source: &str) -> usize {
    // Use split_terminator instead of lines so that if there is a `\r`,
    // it is included in the offset calculation. The `+1` values below
    // account for the `\n`.
    source
        .split_terminator('\n')
        .take(position.line as usize)
        .fold(0, |acc, line| acc + line.len() + 1)
        + position.character as usize
}
