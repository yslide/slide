//! Module `collectors` provides utilities for collecting items in a slide AST.

use crate::grammar::*;
use crate::{InternedStr, Span};

use std::collections::{HashMap, HashSet};

/// Collects unique variable names in an expression.
pub fn collect_var_names(expr: &RcExpr) -> HashSet<InternedStr> {
    let mut collector = VarNameCollector::default();
    collector.visit_expr(expr);
    collector.vars
}
#[derive(Default)]
struct VarNameCollector {
    vars: HashSet<InternedStr>,
}
impl<'a> StmtVisitor<'a> for VarNameCollector {
    fn visit_var(&mut self, var: &'a InternedStr, _span: Span) {
        self.vars.insert(*var);
    }
}

/// Collects variable assignments in a program.
pub fn collect_var_asgns(program: &StmtList) -> HashMap<InternedStr, Vec<&Assignment>> {
    let mut collector = VarAsgnsCollector::default();
    collector.visit(program);
    collector.defs
}
#[derive(Default)]
struct VarAsgnsCollector<'a> {
    defs: HashMap<InternedStr, Vec<&'a Assignment>>,
}
impl<'a> StmtVisitor<'a> for VarAsgnsCollector<'a> {
    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        if let Some(var) = asgn.lhs.get_var() {
            self.defs.entry(var).or_default().push(asgn);
        }
    }
}

/// Collects unique pattern names in an pattern expression.
pub fn collect_pat_names(expr: &RcExprPat) -> HashSet<&str> {
    let mut collector = PatternCollector::default();
    collector.visit(expr);
    collector.pats
}
#[derive(Default)]
struct PatternCollector<'a> {
    pats: HashSet<&'a str>,
}
impl<'a> ExprPatVisitor<'a> for PatternCollector<'a> {
    fn visit_var_pat(&mut self, var_pat: &'a str, _span: Span) {
        self.pats.insert(var_pat);
    }
    fn visit_const_pat(&mut self, const_pat: &'a str, _span: Span) {
        self.pats.insert(const_pat);
    }
    fn visit_any_pat(&mut self, any_pat: &'a str, _span: Span) {
        self.pats.insert(any_pat);
    }
}

#[cfg(test)]
mod test {
    use crate::{parse_expr, parse_expression_pattern, scan};

    #[test]
    fn collect_var_names() {
        let parsed = parse_expr!("a + b + c + a + d / b ^ e");
        let vars = super::collect_var_names(&parsed);

        let mut pats: Vec<_> = vars.into_iter().map(|v| v.to_string()).collect();
        pats.sort();

        assert_eq!(pats, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn collect_pat_names() {
        let parsed = parse_expression_pattern(scan("$a + _b * (#c - [$d]) / $a").tokens).program;
        let pats = super::collect_pat_names(&parsed);

        let mut pats: Vec<_> = pats.into_iter().collect();
        pats.sort_by(|a, b| a.as_bytes()[1].cmp(&b.as_bytes()[1]));

        assert_eq!(pats, vec!["$a", "_b", "#c", "$d"]);
    }
}
