//! Emit strategies for the libslide grammar IR.

use crate::grammar::*;

use core::fmt;

/// The format in which a slide grammar should be emitted.
#[derive(Copy, Clone)]
pub enum EmitFormat {
    /// Canonical, human-readable form.
    /// For example, `1+1` is output as `1 + 1`.
    Pretty,
    /// S-expression form.
    /// For example, `1+1` is output as `(+ 1 1)`.
    SExpression,
    /// LaTeX output form.
    /// For example, `(1 + 1)` is output as `\left(1 + 1\right)`.
    /// NB: this is not yet implemented.
    Latex,
    /// Slide internal debug form.
    /// NB: this form is not stable, and no assumptions should be made about it.
    Debug,
}

impl From<String> for EmitFormat {
    fn from(ef: String) -> Self {
        match ef.as_ref() {
            "pretty" => EmitFormat::Pretty,
            "s-expression" => EmitFormat::SExpression,
            "latex" => EmitFormat::Latex,
            "debug" => EmitFormat::Debug,
            _ => unreachable!(),
        }
    }
}

bitflags::bitflags! {
    /// Configuration options for emitting a slide grammar.
    #[derive(Default)]
    pub struct EmitConfig: u32 {
        /// Emit divisions as fractions.
        /// Applies to LaTeX emit.
        const FRAC = 1;
        /// Emit assignment operators as ":=".
        /// Applies to pretty emit.
        const DEFINE_ASSIGN = 2;
    }
}

impl From<Vec<String>> for EmitConfig {
    fn from(opts: Vec<String>) -> Self {
        let mut config = EmitConfig::default();
        for opt in opts {
            config |= match opt.as_ref() {
                "frac" => EmitConfig::FRAC,
                "define-assign" => EmitConfig::DEFINE_ASSIGN,
                _ => unreachable!(),
            }
        }
        config
    }
}

/// Implements the emission of a type in an [EmitFormat][EmitFormat].
pub trait Emit
where
    // These are trivially implementable using `emit_pretty` and `emit_debug`. The easiest way to
    // do this is with the `fmt_emit_impl` macro.
    Self: fmt::Display + fmt::Debug,
{
    /// Emit `self` with the given [EmitFormat][EmitFormat].
    ///
    /// NB: This is a multiplexer of the corresponding `emit_` methods present on [Emit][Emit],
    /// except for [EmitFormat::Latex][EmitFormat::Latex], which is emitted via
    /// [emit_wrapped_latex][Emit::emit_wrapped_latex].
    fn emit(&self, form: EmitFormat, config: EmitConfig) -> String {
        match form {
            EmitFormat::Pretty => self.emit_pretty(config),
            EmitFormat::SExpression => self.emit_s_expression(config),
            EmitFormat::Latex => self.emit_wrapped_latex(config),
            EmitFormat::Debug => self.emit_debug(config),
        }
    }

    /// Emit `self` with the [pretty emit format][EmitFormat::Pretty]
    fn emit_pretty(&self, config: EmitConfig) -> String;

    /// Emit `self` with the [debug emit format][EmitFormat::Debug]
    fn emit_debug(&self, _config: EmitConfig) -> String {
        format!("{:#?}", self)
    }

    /// Emit `self` with the [s_expression emit format][EmitFormat::SExpression]
    fn emit_s_expression(&self, config: EmitConfig) -> String;

    /// Emit `self` with the [LaTeX emit format][EmitFormat::Latex]
    fn emit_latex(&self, config: EmitConfig) -> String;

    /// Same as [emit_latex][Emit::emit_latex], but wraps the latex code in inline math mode.
    fn emit_wrapped_latex(&self, config: EmitConfig) -> String {
        format!("${}$", self.emit_latex(config))
    }
}

/// Creates free-standing emit functions for use in other macros, where calling Self::emit_* is
/// inconvinient.
macro_rules! mk_free_emit_fns {
    ($($name:ident;)*) => {$(
        #[inline]
        fn $name(arg: &impl Emit, config: EmitConfig) -> String {
            arg.$name(config)
        }
    )*};
}

mk_free_emit_fns! {
    emit_pretty;
    emit_latex;
}

/// Implements `core::fmt::Display` for a type implementing `Emit`.
/// TODO: Maybe this can be a proc macro?
macro_rules! fmt_emit_impl {
    ($S:path) => {
        impl core::fmt::Display for $S {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.emit_pretty(EmitConfig::default()))
            }
        }
    };
}

macro_rules! normal_wrap {
    (($expr:expr)) => {
        format!("({})", $expr)
    };
    ([$expr:expr]) => {
        format!("[{}]", $expr)
    };
}

macro_rules! latex_wrap {
    (($expr:expr)) => {
        format!("\\left({}\\right)", $expr)
    };
    ([$expr:expr]) => {
        format!("\\left[{}\\right]", $expr)
    };
}

fmt_emit_impl!(Stmt);
impl Emit for Stmt {
    fn emit_pretty(&self, config: EmitConfig) -> String {
        match self {
            Self::Expr(expr) => expr.emit_pretty(config),
            Self::Assignment(asgn) => asgn.emit_pretty(config),
        }
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        match self {
            Self::Expr(expr) => expr.emit_s_expression(config),
            Self::Assignment(Assignment { var, rhs }) => {
                format!("(= {} {})", var, rhs.emit_s_expression(config))
            }
        }
    }

    fn emit_latex(&self, config: EmitConfig) -> String {
        match self {
            Self::Expr(expr) => expr.emit_latex(config),
            Self::Assignment(asgn) => asgn.emit_latex(config),
        }
    }
}

fmt_emit_impl!(Assignment);
impl Emit for Assignment {
    fn emit_pretty(&self, config: EmitConfig) -> String {
        let assign = if config.contains(EmitConfig::DEFINE_ASSIGN) {
            ":="
        } else {
            "="
        };
        format!("{} {} {}", self.var, assign, self.rhs.emit_pretty(config))
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        format!("(= {} {})", self.var, self.rhs.emit_s_expression(config))
    }

    fn emit_latex(&self, config: EmitConfig) -> String {
        format!("{} = {}", self.var, self.rhs.emit_latex(config))
    }
}

fmt_emit_impl!(Expr);
impl Emit for Expr {
    fn emit_pretty(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(num) => num.to_string(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_pretty(config),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_pretty(config),
            Self::Parend(expr) => normal_wrap!((expr.emit_pretty(config))),
            Self::Bracketed(expr) => normal_wrap!([expr.emit_pretty(config)]),
        }
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_s_expression(config),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_s_expression(config),
            Self::Parend(inner) => normal_wrap!((inner.emit_s_expression(config))),
            Self::Bracketed(inner) => normal_wrap!([inner.emit_s_expression(config)]),
        }
    }

    fn emit_latex(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(num) => match num.to_string().as_ref() {
                "inf" => "\\infty",
                other => other,
            }
            .to_owned(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_latex(config),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_latex(config),
            Self::Parend(expr) => latex_wrap!((expr.emit_latex(config))),
            Self::Bracketed(expr) => latex_wrap!([expr.emit_latex(config)]),
        }
    }
}

fmt_emit_impl!(BinaryOperator);
impl Emit for BinaryOperator {
    fn emit_pretty(&self, _config: EmitConfig) -> String {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Exp => "^",
        }
        .to_owned()
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        self.emit_pretty(config)
    }

    fn emit_latex(&self, _config: EmitConfig) -> String {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Mod => "\\mod",
            Self::Exp => "^",
        }
        .to_owned()
    }
}

macro_rules! format_binary_operand {
    ($E:ident, $parent_expr:ident, $operand:expr, $is_right_operand:expr, $emit:ident, $wrap:ident, $config:ident) => {
        match $operand.as_ref() {
            // We want to format items like
            //    v--------- child op
            //         v---- parent op
            // (3 + 5) ^ 2 [1]
            //  3 + 5  + 2
            //  3 - 5  + 2
            //  3 * 5  + 2
            // and
            //   v---------- parent op
            //        v----- child op
            // 2 +  3 + 5
            // 2 - (3 + 5)
            // 2 * (3 + 5)
            //
            // So the idea here is as follows:
            // - if the child op precedence is less than the parent op, we must always parenthesize
            //   it ([1])
            // - if the op precedences are equivalent, then
            //   - if the child is on the LHS, we can always unwrap it
            //   - if the child is on the RHS, we parenthesize it unless the parent op is
            //     associative
            //
            // I think this is enough, but maybe we're overlooking left/right associativity?
            $E::BinaryExpr(child) => {
                if child.op.precedence() < $parent_expr.op.precedence()
                    || ($is_right_operand
                        && child.op.precedence() == $parent_expr.op.precedence()
                        && !$parent_expr.op.is_associative())
                {
                    $wrap!(($emit(child, $config)))
                } else {
                    $emit(child, $config)
                }
            }
            expr => $emit(expr, $config),
        }
    };
}

macro_rules! display_binary_expr {
    ($iexpr:ident, $expr:ident) => {
        fmt_emit_impl!(BinaryExpr<$iexpr>);
        impl Emit for BinaryExpr<$iexpr> {
            fn emit_pretty(&self, config: EmitConfig) -> String {
                format!(
                    "{} {} {}",
                    format_binary_operand!(
                        $expr,
                        self,
                        &self.lhs,
                        false,
                        emit_pretty,
                        normal_wrap,
                        config
                    ),
                    self.op.emit_pretty(config),
                    format_binary_operand!(
                        $expr,
                        self,
                        &self.rhs,
                        true,
                        emit_pretty,
                        normal_wrap,
                        config
                    ),
                )
            }

            fn emit_s_expression(&self, config: EmitConfig) -> String {
                format!(
                    "({} {} {})",
                    self.op.emit_s_expression(config),
                    self.lhs.emit_s_expression(config),
                    self.rhs.emit_s_expression(config),
                )
            }

            fn emit_latex(&self, config: EmitConfig) -> String {
                let lhs = format_binary_operand!(
                    $expr, self, &self.lhs, false, emit_latex, latex_wrap, config
                );
                let op = self.op.emit_latex(config);
                let rhs = format_binary_operand!(
                    $expr, self, &self.rhs, true, emit_latex, latex_wrap, config
                );
                match self.op {
                    BinaryOperator::Exp => format!("{}^{{{}}}", lhs, rhs),
                    BinaryOperator::Div if config.contains(EmitConfig::FRAC) => {
                        format!("\\frac{{{}}}{{{}}}", lhs, rhs)
                    }
                    _ => format!("{} {} {}", lhs, op, rhs),
                }
            }
        }
    };
}
display_binary_expr!(InternedExpr, Expr);
display_binary_expr!(InternedExprPat, ExprPat);

fmt_emit_impl!(UnaryOperator);
impl Emit for UnaryOperator {
    fn emit_pretty(&self, _config: EmitConfig) -> String {
        match self {
            Self::SignPositive => "+",
            Self::SignNegative => "-",
        }
        .to_owned()
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        self.emit_pretty(config)
    }

    fn emit_latex(&self, config: EmitConfig) -> String {
        self.emit_pretty(config)
    }
}

macro_rules! display_unary_expr {
    ($iexpr:ident, $expr:ident) => {
        fmt_emit_impl!(UnaryExpr<$iexpr>);
        impl Emit for UnaryExpr<$iexpr> {
            fn emit_pretty(&self, config: EmitConfig) -> String {
                let format_arg = |arg: &$iexpr| match arg.as_ref() {
                    $expr::BinaryExpr(l) => normal_wrap!((l.emit_pretty(config))),
                    expr => expr.emit_pretty(config),
                };
                format!("{}{}", self.op.emit_pretty(config), format_arg(&self.rhs))
            }

            fn emit_s_expression(&self, config: EmitConfig) -> String {
                format!(
                    "({} {})",
                    self.op.emit_s_expression(config),
                    self.rhs.emit_s_expression(config),
                )
            }

            fn emit_latex(&self, config: EmitConfig) -> String {
                let format_arg = |arg: &$iexpr| match arg.as_ref() {
                    $expr::BinaryExpr(l) => latex_wrap!((l.emit_latex(config))),
                    expr => expr.emit_latex(config),
                };
                format!("{}{}", self.op.emit_latex(config), format_arg(&self.rhs))
            }
        }
    };
}
display_unary_expr!(InternedExpr, Expr);
display_unary_expr!(InternedExprPat, ExprPat);

fmt_emit_impl!(ExprPat);
impl Emit for ExprPat {
    fn emit_pretty(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(num) => num.to_string(),
            Self::VarPat(var) | Self::ConstPat(var) | Self::AnyPat(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_pretty(config),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_pretty(config),
            Self::Parend(expr) => normal_wrap!((expr.emit_pretty(config))),
            Self::Bracketed(expr) => normal_wrap!([expr.emit_pretty(config)]),
        }
    }

    fn emit_s_expression(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::VarPat(pat) | Self::ConstPat(pat) | Self::AnyPat(pat) => pat.to_string(),
            Self::BinaryExpr(binary) => binary.emit_s_expression(config),
            Self::UnaryExpr(unary) => unary.emit_s_expression(config),
            Self::Parend(inner) => normal_wrap!((inner.emit_s_expression(config))),
            Self::Bracketed(inner) => normal_wrap!([inner.emit_s_expression(config)]),
        }
    }

    fn emit_latex(&self, config: EmitConfig) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::VarPat(pat) | Self::ConstPat(pat) | Self::AnyPat(pat) => {
                // $a, #a, _a all need to be escaped as \$a, \#a, \_a.
                format!("\\{}", pat.to_string())
            }
            Self::BinaryExpr(binary_expr) => binary_expr.emit_latex(config),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_latex(config),
            Self::Parend(inner) => latex_wrap!((inner.emit_latex(config))),
            Self::Bracketed(inner) => latex_wrap!([inner.emit_latex(config)]),
        }
    }
}
