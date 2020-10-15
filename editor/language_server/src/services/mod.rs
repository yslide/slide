//! Provides langauge services for a slide language server.

/// Provides hover services for a slide langauge server.
/// See [`get_hover_info`](hover::get_hover_info) for information on returned hover data.
mod hover;
pub use hover::get_hover_info;
