use crate::diagnostic::Diagnostic;
use crate::linter::rule::Rule;
use crate::parser::ast::*;

/// W001: `*` のみのフリーテキストフィルタを検出するルールです。
/// `*` は全てのイベントにマッチするため、意図的でない場合は不要です。
pub struct WildcardOnlyFilter;

impl Rule for WildcardOnlyFilter {
    fn id(&self) -> &'static str {
        "W001"
    }

    fn description(&self) -> &'static str {
        "wildcard-only filter matches all events"
    }

    fn check(&self, query: &Query, _source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for stage in &query.stages {
            self.check_stage(stage, &mut diagnostics);
        }
        diagnostics
    }
}

impl WildcardOnlyFilter {
    fn check_stage(&self, stage: &PipelineStage, diagnostics: &mut Vec<Diagnostic>) {
        match &stage.kind {
            StageKind::Filter(filter) => {
                self.check_filter(filter, stage.span, diagnostics);
            }
            StageKind::CaseStatement(cs) => {
                for branch in &cs.branches {
                    for s in &branch.pipeline {
                        self.check_stage(s, diagnostics);
                    }
                }
            }
            StageKind::MatchStatement(ms) => {
                for branch in &ms.branches {
                    for s in &branch.pipeline {
                        self.check_stage(s, diagnostics);
                    }
                }
            }
            StageKind::FunctionCall(_) | StageKind::Assignment(_) => {}
        }
    }

    fn check_filter(
        &self,
        filter: &FilterExpr,
        span: crate::lexer::token::Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match filter {
            FilterExpr::FreeText(text) if text == "*" => {
                diagnostics.push(Diagnostic::warning(
                    "W001",
                    "'*' filter matches all events and may be unnecessary",
                    span,
                ));
            }
            FilterExpr::And(left, right) | FilterExpr::Or(left, right) => {
                self.check_filter(left, span, diagnostics);
                self.check_filter(right, span, diagnostics);
            }
            FilterExpr::Not(inner) | FilterExpr::Grouped(inner) => {
                self.check_filter(inner, span, diagnostics);
            }
            FilterExpr::FieldShorthand { .. }
            | FilterExpr::FunctionCall(_)
            | FilterExpr::FieldFilter { .. } => {}
            FilterExpr::FreeText(_) => {}
        }
    }
}
