//! Module `services` describes language services provied by a [`Program`](crate::Program) to
//! answer server queries.

// The following modules contribute to the API interface of Program, and do not expose any free
// functions publicly.
mod actions;
mod annotations;
mod completions;
mod definitions;
mod folding_ranges;
mod format;
mod highlight;
mod hover;
mod references;
mod rename;
mod selection_ranges;
mod symbols;

// The following modules contribute free functions to the services API.
pub mod diagnostics;

/// Module `response` describes the API for responses of language queries at the level of a
/// program. These can then be marshaled into a response at the level of an entire
/// [`Document`](crate::document_registry::Document) for answering queries with the
/// LSP API.
pub mod response;
