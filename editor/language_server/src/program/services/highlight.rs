//! Module `highlight` provides semantic highlight services.

use super::local_response::LocalHighlight;
use crate::program::services::references as refs;
use crate::Program;

use tower_lsp::lsp_types::*;

impl Program {
    /// Returns semantic highlighting ranges relevant to a offset in a document.
    /// If the offset is over an identifier (variable), all references to that variable are returned.
    /// Otherwise, nothing is returned.
    pub fn get_semantic_highlights(&self, offset: usize) -> Option<Vec<LocalHighlight>> {
        // The nice thing is that the references service already does most of the work to get
        // references, so we can just piggyback off that and translate types accordingly.
        let references = self.get_kinded_references(offset)?;
        let references = references
            .into_iter()
            .map(|rk| LocalHighlight {
                span: *rk.span(),
                kind: match rk {
                    refs::ReferenceKind::Definition(_) => DocumentHighlightKind::Write,
                    refs::ReferenceKind::Usage(_) => DocumentHighlightKind::Read,
                },
            })
            .collect();

        Some(references)
    }
}
