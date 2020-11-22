//! Module `services` describes language services provied by a [`Program`](crate::Program) to
//! answer server queries.

// The following modules contribute to the API interface of Program, and do not expose any free
// functions publicly.
mod definitions;
mod highlight;
mod hover;
mod references;

// The following modules contribute free functions to the services API.
pub mod diagnostics;

/// Module `local_response` describes the API for responses of language queries at the level of a
/// program, and implementations of [`ToAbsoluteResponse`](crate::ToAbsoluteResponse) to raise
/// those responses in the context of a [`Document`](crate::Document).
pub mod local_response;
