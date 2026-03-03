use crate::diagnostic::Diagnostic;
use crate::linter::rule::Rule;
use crate::parser::ast::*;

/// W005: OR/AND 演算子の優先順位が曖昧な式を検出するルールです。
/// CQL では OR が AND より結合が強いため、`a and b or c` は
/// `a and (b or c)` と解釈されます。これは多くの言語と逆であるため、
/// 括弧なしで AND と OR を混在させている場合に警告します。
pub struct AmbiguousPrecedence;

impl Rule for AmbiguousPrecedence {
    fn id(&self) -> &'static str {
        "W005"
    }

    fn description(&self) -> &'static str {
        "ambiguous AND/OR precedence (OR binds tighter than AND in CQL)"
    }

    fn check(&self, query: &Query, _source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for stage in &query.stages {
            self.check_stage(stage, &mut diagnostics);
        }
        diagnostics
    }
}

impl AmbiguousPrecedence {
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
            // AND の右辺に括弧なしの OR がある場合が要注意です
            FilterExpr::And(left, right) => {
                if self.contains_ungrouped_or(right) || self.contains_ungrouped_or(left) {
                    diagnostics.push(Diagnostic::warning(
                        "W005",
                        "AND/OR used without parentheses; in CQL, OR binds tighter than AND, which differs from most languages. Consider adding explicit parentheses.",
                        span,
                    ));
                }
                self.check_filter(left, span, diagnostics);
                self.check_filter(right, span, diagnostics);
            }
            FilterExpr::Or(left, right) => {
                self.check_filter(left, span, diagnostics);
                self.check_filter(right, span, diagnostics);
            }
            FilterExpr::Not(inner) => {
                self.check_filter(inner, span, diagnostics);
            }
            FilterExpr::FieldShorthand { function, .. } => {
                // FieldShorthand 内の関数呼び出しにはフィルタ式が含まれないため、何もしません
                let _ = function;
            }
            FilterExpr::Grouped(_)
            | FilterExpr::FreeText(_)
            | FilterExpr::FieldFilter { .. }
            | FilterExpr::FunctionCall(_) => {}
        }
    }

    /// 式が括弧で囲まれていない OR を直接含むか判定します。
    fn contains_ungrouped_or(&self, filter: &FilterExpr) -> bool {
        matches!(filter, FilterExpr::Or(..))
    }
}
