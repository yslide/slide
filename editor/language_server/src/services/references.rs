use crate::ast;
use crate::shims::{to_offset, to_range};
use crate::Program;

use libslide::*;
use tower_lsp::lsp_types::*;
use visit::StmtVisitor;

/// Returns references relevant to a position in a document.
/// - If the position is over an identifier (variable),
///   - if `include_declaration` is true, all references are returned.
///   - otherwise only non-declaration references are returned.
/// - Otherwise, nothing is returned.
pub(crate) fn get_references(
    position: Position,
    include_declaration: bool,
    program: &Program,
) -> Option<Vec<Location>> {
    let uri = program.document_uri.as_ref();
    let source = program.source.as_ref();
    let references = get_kinded_references(position, program)?;
    let references = references
        .into_iter()
        .filter_map(|rk| match rk {
            ReferenceKind::Definition(_) if !include_declaration => None,
            _ => Some(Location::new((*uri).clone(), to_range(rk.span(), &source))),
        })
        .collect();

    Some(references)
}

/// The kind of a reference.
pub enum ReferenceKind {
    /// The reference is a definition of the item this is a reference of.
    /// For example, "a" in "a = 1" is a definition reference of "a".
    Definition(Span),
    /// The reference is a usage of the item this is a reference of, in an expression.
    /// For example, "a" in "b + a + 1" is a usage reference of "a".
    Usage(Span),
}

impl ReferenceKind {
    pub fn span(&self) -> &Span {
        match self {
            Self::Definition(sp) => sp,
            Self::Usage(sp) => sp,
        }
    }
}

pub(crate) fn get_kinded_references(
    position: Position,
    program: &Program,
) -> Option<Vec<ReferenceKind>> {
    let program_ast = &program.original_ast();
    let position = to_offset(&position, &program.source);
    let tightest_expr = ast::get_tightest_expr(position, program_ast)?;
    let seeking = tightest_expr.get_var()?;

    let mut reference_finder = ReferenceFinder {
        seeking,
        is_declaration: false,
        refs: vec![],
    };
    reference_finder.visit_stmt_list(program_ast);

    Some(reference_finder.refs)
}

struct ReferenceFinder {
    seeking: InternedStr,
    is_declaration: bool,
    refs: Vec<ReferenceKind>,
}
impl<'a> StmtVisitor<'a> for ReferenceFinder {
    fn visit_var(&mut self, var: &'a InternedStr, span: Span) {
        if *var == self.seeking {
            self.refs.push(match self.is_declaration {
                true => ReferenceKind::Definition(span),
                false => ReferenceKind::Usage(span),
            });
        }
    }

    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        fn visit_asgn_side(visitor: &mut ReferenceFinder, side: &RcExpr) {
            // Mark the expression to visit as being a declaration iff it consists of a single
            // variable.
            visitor.is_declaration = side.is_var();
            visit::descend_expr(visitor, side);
            visitor.is_declaration = false;
        }

        visit_asgn_side(self, &asgn.lhs);
        visit_asgn_side(self, &asgn.rhs);
    }
}
