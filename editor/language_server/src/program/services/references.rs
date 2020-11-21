//! Module `references` provides references for a [`Program`](crate::Program).

use crate::ast;
use crate::shims::to_range;
use crate::Program;

use libslide::*;
use tower_lsp::lsp_types::*;
use visit::StmtVisitor;

impl Program {
    /// Returns references relevant to a offset in a document.
    /// - If the offset is over an identifier (variable),
    ///   - if `include_declaration` is true, all references are returned.
    ///   - otherwise only non-declaration references are returned.
    /// - Otherwise, nothing is returned.
    pub fn get_references(
        &self,
        offset: usize,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        let uri = self.document_uri.as_ref();
        let source = self.source.as_ref();
        let references = self.get_kinded_references(offset)?;
        let references = references
            .into_iter()
            .filter_map(|rk| match rk {
                ReferenceKind::Definition(_) if !include_declaration => None,
                _ => Some(Location::new((*uri).clone(), to_range(rk.span(), &source))),
            })
            .collect();

        Some(references)
    }

    pub fn get_kinded_references(&self, offset: usize) -> Option<Vec<ReferenceKind>> {
        let program_ast = &self.original_ast();
        let tightest_expr = ast::get_tightest_expr(offset, program_ast)?;
        let seeking = tightest_expr.get_var()?;

        let mut reference_finder = ReferenceFinder {
            seeking,
            is_declaration: false,
            refs: vec![],
        };
        reference_finder.visit_stmt_list(program_ast);

        Some(reference_finder.refs)
    }
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
