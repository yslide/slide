//! Module `selection_ranges` provides services for determining ranges in a program around an offset
//! a user may be interested in selecting.

use super::response::*;
use crate::ast::*;
use crate::Program;

impl Program {
    /// Returns ranges around an offset a user may be interested in selection. `None` iff there are
    /// no selection ranges around the cursor; otherwise, there is guaranteed to be at least one
    /// cursor.
    pub fn selection_ranges(&self, offset: usize) -> Option<ProgramSelectionRanges> {
        let ast = self.original_ast();
        let ranges: Vec<_> = get_item_path_to_offset(offset, &ast)
            .into_iter()
            .map(|item| item.span())
            .collect();
        if ranges.is_empty() {
            None
        } else {
            Some(ProgramSelectionRanges(ranges))
        }
    }
}
