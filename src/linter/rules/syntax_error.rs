use crate::diagnostic::Diagnostic;
use crate::linter::rule::Rule;
use crate::parser::ast::Query;

/// E001: 構文エラーを検出するルールです。
/// 実際のエラー検出は Parser が行い、LintEngine が ParseError を Diagnostic に変換します。
/// このルールは追加の構文チェック用に予約されています。
pub struct SyntaxError;

impl Rule for SyntaxError {
    fn id(&self) -> &'static str {
        "E001"
    }

    fn description(&self) -> &'static str {
        "syntax error"
    }

    fn check(&self, _query: &Query, _source: &str) -> Vec<Diagnostic> {
        // パースエラーは LintEngine が直接処理するため、ここでは空を返します
        Vec::new()
    }
}
