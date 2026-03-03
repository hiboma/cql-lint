use crate::diagnostic::Diagnostic;
use crate::linter::known_functions::is_aggregate_function;
use crate::linter::rule::Rule;
use crate::parser::ast::*;

/// W006: 集約関数の後にフィルタがある場合に警告するルールです。
/// フィルタは集約の前に配置した方がパフォーマンスが向上します。
pub struct FilterOrder;

impl Rule for FilterOrder {
    fn id(&self) -> &'static str {
        "W006"
    }

    fn description(&self) -> &'static str {
        "filter after aggregate may reduce performance"
    }

    fn check(&self, query: &Query, _source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen_aggregate = false;

        for stage in &query.stages {
            match &stage.kind {
                StageKind::FunctionCall(fc) => {
                    if is_aggregate_function(&fc.name) {
                        seen_aggregate = true;
                    }
                }
                StageKind::Filter(_) => {
                    if seen_aggregate {
                        diagnostics.push(Diagnostic::info(
                            "W006",
                            "filter placed after aggregate function; consider moving filters before aggregation for better performance",
                            stage.span,
                        ));
                    }
                }
                StageKind::Assignment(_)
                | StageKind::CaseStatement(_)
                | StageKind::MatchStatement(_) => {}
            }
        }

        diagnostics
    }
}
