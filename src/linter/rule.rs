use crate::diagnostic::Diagnostic;
use crate::parser::ast::Query;

/// Lint ルールのトレイトです。
/// 各ルールはこのトレイトを実装し、AST を検査して診断メッセージを返します。
pub trait Rule {
    /// ルール ID を返します (例: "W002")。
    fn id(&self) -> &'static str;

    /// ルールの説明を返します。
    fn description(&self) -> &'static str;

    /// AST を検査し、検出した問題を診断メッセージとして返します。
    fn check(&self, query: &Query, source: &str) -> Vec<Diagnostic>;
}
