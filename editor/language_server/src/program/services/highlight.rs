//! Module `highlight` provides semantic highlight services.

use crate::program::services::references as refs;
use crate::shims::to_range;
use crate::Program;

use tower_lsp::lsp_types::*;

impl Program {
    /// Returns semantic highlighting ranges relevant to a position in a document.
    /// If the position is over an identifier (variable), all references to that variable are returned.
    /// Otherwise, nothing is returned.
    pub fn get_semantic_highlights(&self, position: usize) -> Option<Vec<DocumentHighlight>> {
        let source = &self.source;
        // The nice thing is that the references service already does most of the work to get
        // references, so we can just piggyback off that and translate types accordingly.
        let references = self.get_kinded_references(position)?;
        let references = references
            .into_iter()
            .map(|rk| DocumentHighlight {
                range: to_range(rk.span(), source),
                kind: Some(match rk {
                    refs::ReferenceKind::Definition(_) => DocumentHighlightKind::Write,
                    refs::ReferenceKind::Usage(_) => DocumentHighlightKind::Read,
                }),
            })
            .collect();

        Some(references)
    }
}
