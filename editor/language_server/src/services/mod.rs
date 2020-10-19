//! Provides langauge services for a slide language server.

/// Provides completion services for a slide langauge server.
/// See [`get_completions`](completions::get_completions) for more information.
mod completions;
pub(crate) use completions::get_completions;

/// Provides definitions services for a slide langauge server.
/// See [`get_definitions`](definitions::get_definitions) for more information.
mod definitions;
pub(crate) use definitions::get_definitions;

/// Provides hover services for a slide langauge server.
/// See [`get_hover_info`](hover::get_hover_info) for information on returned hover data.
mod hover;
pub(crate) use hover::get_hover_info;

/// Provides references for a program.
/// See [`get_references`](references::get_references) for more information.
mod references;
pub(crate) use references::get_references;

/// Provides semantic highlight services.
/// See [`get_semantic_highlights`](highlight::get_semantic_highlights) for more information.
mod highlight;
pub(crate) use highlight::get_semantic_highlights;
