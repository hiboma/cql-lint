use crate::diagnostic::Diagnostic;
use crate::linter::known_functions::is_known_function;
use crate::linter::rule::Rule;
use crate::parser::ast::*;

/// W002: 未知の関数名の使用を検出するルールです。
pub struct UnknownFunction;

impl Rule for UnknownFunction {
    fn id(&self) -> &'static str {
        "W002"
    }

    fn description(&self) -> &'static str {
        "unknown function name"
    }

    fn check(&self, query: &Query, _source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for stage in &query.stages {
            self.check_stage(stage, &mut diagnostics);
        }
        diagnostics
    }
}

impl UnknownFunction {
    fn check_stage(&self, stage: &PipelineStage, diagnostics: &mut Vec<Diagnostic>) {
        match &stage.kind {
            StageKind::FunctionCall(fc) => {
                self.check_function_call(fc, diagnostics);
            }
            StageKind::Filter(filter) => {
                self.check_filter(filter, diagnostics);
            }
            StageKind::Assignment(assignment) => {
                self.check_expr(&assignment.value, diagnostics);
            }
            StageKind::CaseStatement(case_stmt) => {
                for branch in &case_stmt.branches {
                    for s in &branch.pipeline {
                        self.check_stage(s, diagnostics);
                    }
                }
            }
            StageKind::MatchStatement(match_stmt) => {
                for branch in &match_stmt.branches {
                    self.check_expr(&branch.pattern, diagnostics);
                    for s in &branch.pipeline {
                        self.check_stage(s, diagnostics);
                    }
                }
            }
        }
    }

    fn check_function_call(&self, fc: &FunctionCall, diagnostics: &mut Vec<Diagnostic>) {
        if !is_known_function(&fc.name) {
            diagnostics.push(Diagnostic::warning(
                "W002",
                format!("unknown function '{}'", fc.name),
                fc.span,
            ));
        }

        // 引数内の式も検査します
        for arg in &fc.arguments {
            match arg {
                Argument::Positional(expr) => self.check_expr(expr, diagnostics),
                Argument::Named { value, .. } => self.check_expr(value, diagnostics),
            }
        }
    }

    fn check_filter(&self, filter: &FilterExpr, diagnostics: &mut Vec<Diagnostic>) {
        match filter {
            FilterExpr::And(left, right) | FilterExpr::Or(left, right) => {
                self.check_filter(left, diagnostics);
                self.check_filter(right, diagnostics);
            }
            FilterExpr::Not(inner) | FilterExpr::Grouped(inner) => {
                self.check_filter(inner, diagnostics);
            }
            FilterExpr::FunctionCall(fc) => {
                self.check_function_call(fc, diagnostics);
            }
            FilterExpr::FieldShorthand { function, .. } => {
                self.check_function_call(function, diagnostics);
            }
            FilterExpr::FreeText(_) | FilterExpr::FieldFilter { .. } => {}
        }
    }

    fn check_expr(&self, expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
        match expr {
            Expr::FunctionCall(fc) => {
                self.check_function_call(fc, diagnostics);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_expr(left, diagnostics);
                self.check_expr(right, diagnostics);
            }
            Expr::UnaryOp { operand, .. } => {
                self.check_expr(operand, diagnostics);
            }
            Expr::Array(elements) => {
                for elem in elements {
                    self.check_expr(elem, diagnostics);
                }
            }
            Expr::SubQuery(query) => {
                for stage in &query.stages {
                    self.check_stage(stage, diagnostics);
                }
            }
            Expr::CompareExpr { left, right, .. } => {
                self.check_expr(left, diagnostics);
                self.check_expr(right, diagnostics);
            }
            Expr::SavedQueryCall { arguments, .. } => {
                for arg in arguments {
                    match arg {
                        Argument::Positional(expr) => self.check_expr(expr, diagnostics),
                        Argument::Named { value, .. } => self.check_expr(value, diagnostics),
                    }
                }
            }
            Expr::IndexAccess { object, index } => {
                self.check_expr(object, diagnostics);
                self.check_expr(index, diagnostics);
            }
            Expr::Number(_)
            | Expr::String(_)
            | Expr::Bool(_)
            | Expr::Field(_)
            | Expr::Regex { .. }
            | Expr::Wildcard(_)
            | Expr::Parameter(_) => {}
        }
    }
}
