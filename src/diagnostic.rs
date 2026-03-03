use serde::Serialize;

use crate::lexer::token::Span;

/// 診断メッセージの重大度を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 構文エラーなど、クエリの実行を阻害する問題です。
    Error,
    /// ベストプラクティス違反や潜在的な問題です。
    Warning,
    /// スタイルに関する提案です。
    Info,
}

/// 診断メッセージを表します。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    /// ルール ID (例: "E001", "W002")
    pub rule_id: String,
    /// 重大度
    pub severity: Severity,
    /// メッセージ
    pub message: String,
    /// ソースコード上の位置
    pub span: Span,
}

impl Diagnostic {
    pub fn error(rule_id: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity: Severity::Error,
            message: message.into(),
            span,
        }
    }

    pub fn warning(rule_id: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity: Severity::Warning,
            message: message.into(),
            span,
        }
    }

    pub fn info(rule_id: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity: Severity::Info,
            message: message.into(),
            span,
        }
    }
}
