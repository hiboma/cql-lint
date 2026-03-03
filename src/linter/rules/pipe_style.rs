use crate::diagnostic::Diagnostic;
use crate::lexer::token::Span;
use crate::linter::rule::Rule;
use crate::parser::ast::Query;

/// W004: パイプ `|` の前後に空白がないケースを検出するルールです。
pub struct PipeStyle;

impl Rule for PipeStyle {
    fn id(&self) -> &'static str {
        "W004"
    }

    fn description(&self) -> &'static str {
        "pipe '|' should be surrounded by spaces"
    }

    fn check(&self, query: &Query, source: &str) -> Vec<Diagnostic> {
        if query.stages.len() < 2 {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let bytes = source.as_bytes();

        // ソース内の `|` を探して前後の空白を確認します
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'"' => {
                    // 文字列リテラルをスキップします
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            i += 2;
                        } else if bytes[i] == b'"' {
                            i += 1;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                }
                b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                    // 単一行コメントをスキップします
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                    // 複数行コメントをスキップします
                    i += 2;
                    while i + 1 < bytes.len() {
                        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                b'|' => {
                    let has_space_before = i > 0
                        && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t' || bytes[i - 1] == b'\n');
                    let has_space_after = i + 1 < bytes.len()
                        && (bytes[i + 1] == b' ' || bytes[i + 1] == b'\t' || bytes[i + 1] == b'\n');

                    if !has_space_before || !has_space_after {
                        diagnostics.push(Diagnostic::info(
                            "W004",
                            "pipe '|' should be surrounded by spaces for readability",
                            Span::new(i, i + 1),
                        ));
                    }
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        diagnostics
    }
}
