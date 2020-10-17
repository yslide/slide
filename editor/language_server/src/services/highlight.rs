use crate::services::references as refs;
use crate::shims::to_range;
use crate::ProgramInfo;

use tower_lsp::lsp_types::*;

/// Returns semantic highlighting ranges relevant to a position in a document.
/// If the position is over an identifier (variable), all references to that variable are returned.
/// Otherwise, nothing is returned.
pub(crate) fn get_semantic_highlights(
    position: Position,
    program_info: &ProgramInfo,
) -> Option<Vec<DocumentHighlight>> {
    let ProgramInfo { source, .. } = program_info;
    // The nice thing is that the references service already does most of the work to get
    // references, so we can just piggyback off that and translate types accordingly.
    let references = refs::get_kinded_references(position, program_info)?;
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
