use crate::diagnostic::Diagnostic;
use crate::linter::rule::Rule;
use crate::parser::ast::*;

/// W003: 関数の必須引数が不足している場合に警告するルールです。
/// 一部の主要な関数について、最低限必要な引数の数を検査します。
pub struct MissingArgs;

/// 関数名と最小引数数の定義です。
static REQUIRED_ARGS: &[(&str, usize)] = &[
    ("groupBy", 1),
    ("top", 1),
    ("percentile", 1),
    ("rename", 1),
    ("replace", 1),
    ("regex", 1),
    ("cidr", 1),
    ("in", 1),
    ("split", 1),
    ("splitString", 1),
    ("format", 1),
    ("concat", 1),
    ("parseTimestamp", 1),
    ("bucket", 1),
    ("timeChart", 1),
    ("match", 1),
    ("sum", 1),
    ("avg", 1),
    ("max", 1),
    ("min", 1),
    ("wildcard", 1),
    ("test", 1),
    // Step 2 で追加した関数の必須引数
    ("callFunction", 1),
    ("partition", 1),
    ("series", 1),
    ("array:reduceColumn", 1),
    ("array:reduceRow", 1),
    ("array:rename", 1),
    ("matchAsArray", 1),
    ("text:endsWith", 1),
    ("text:startsWith", 1),
    ("parseInt", 1),
    ("reverseDns", 1),
    ("shannonEntropy", 1),
    ("parseCEF", 1),
    ("parseLEEF", 1),
    ("parseUri", 1),
    ("neighbor", 1),
    ("stripAnsiCodes", 1),
    ("text:editDistance", 1),
    ("text:editDistanceAsArray", 1),
    ("text:length", 1),
    ("text:positionOf", 1),
    ("text:substring", 2),
    ("text:trim", 1),
    ("lowercase", 1),
    ("uppercase", 1),
    ("geography:distance", 2),
    ("geohash", 1),
    ("tokenHash", 1),
    ("setTimeInterval", 1),
];

impl Rule for MissingArgs {
    fn id(&self) -> &'static str {
        "W003"
    }

    fn description(&self) -> &'static str {
        "function is missing required arguments"
    }

    fn check(&self, query: &Query, _source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for stage in &query.stages {
            self.check_stage(stage, &mut diagnostics);
        }
        diagnostics
    }
}

impl MissingArgs {
    fn check_stage(&self, stage: &PipelineStage, diagnostics: &mut Vec<Diagnostic>) {
        match &stage.kind {
            StageKind::FunctionCall(fc) => self.check_function_call(fc, diagnostics),
            StageKind::Filter(filter) => self.check_filter(filter, diagnostics),
            StageKind::Assignment(assignment) => self.check_expr(&assignment.value, diagnostics),
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
        for &(name, min_args) in REQUIRED_ARGS {
            if fc.name.eq_ignore_ascii_case(name) && fc.arguments.len() < min_args {
                diagnostics.push(Diagnostic::warning(
                    "W003",
                    format!(
                        "function '{}' requires at least {} argument(s), but {} provided",
                        fc.name,
                        min_args,
                        fc.arguments.len()
                    ),
                    fc.span,
                ));
                break;
            }
        }

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
            FilterExpr::FunctionCall(fc) => self.check_function_call(fc, diagnostics),
            FilterExpr::FieldShorthand { function, .. } => {
                self.check_function_call(function, diagnostics);
            }
            FilterExpr::FreeText(_) | FilterExpr::FieldFilter { .. } => {}
        }
    }

    fn check_expr(&self, expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
        match expr {
            Expr::FunctionCall(fc) => self.check_function_call(fc, diagnostics),
            Expr::BinaryOp { left, right, .. } => {
                self.check_expr(left, diagnostics);
                self.check_expr(right, diagnostics);
            }
            Expr::UnaryOp { operand, .. } => self.check_expr(operand, diagnostics),
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
            Expr::Comparison { left, right, .. } => {
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
