pub mod known_functions;
pub mod rule;
pub mod rules;

use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::parser::Parser;
use rule::Rule;

/// Lint エンジンです。登録されたルールを AST に対して実行します。
pub struct LintEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl LintEngine {
    /// デフォルトのルールセットで LintEngine を作成します。
    pub fn new() -> Self {
        let rules: Vec<Box<dyn Rule>> = vec![
            Box::new(rules::syntax_error::SyntaxError),
            Box::new(rules::wildcard_only_filter::WildcardOnlyFilter),
            Box::new(rules::unknown_function::UnknownFunction),
            Box::new(rules::missing_args::MissingArgs),
            Box::new(rules::pipe_style::PipeStyle),
            Box::new(rules::ambiguous_precedence::AmbiguousPrecedence),
            Box::new(rules::filter_order::FilterOrder),
        ];
        Self { rules }
    }

    /// クエリ文字列に対して lint を実行し、診断メッセージのリストを返します。
    pub fn lint(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Lexer
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // Parser
        let parser = Parser::new(tokens);
        let (query, parse_errors) = parser.parse();

        // E001: 構文エラーを診断に変換します
        for err in &parse_errors {
            diagnostics.push(Diagnostic::error("E001", &err.message, err.span));
        }

        // AST が取得できた場合、各ルールを実行します
        if let Some(ref query) = query {
            for rule in &self.rules {
                // E001 は parse_errors から直接追加済みなのでスキップします
                if rule.id() == "E001" {
                    if parse_errors.is_empty() {
                        diagnostics.extend(rule.check(query, source));
                    }
                    continue;
                }
                diagnostics.extend(rule.check(query, source));
            }
        }

        // span の開始位置でソートします
        diagnostics.sort_by_key(|d| d.span.start);
        diagnostics
    }

    /// 登録されているルールの一覧を返します。
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }
}

impl Default for LintEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;

    #[test]
    fn test_lint_valid_query() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 | count()");
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_lint_syntax_error() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("count(");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].rule_id, "E001");
        assert_eq!(diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_lint_unknown_function() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("fooBarBaz()");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].rule_id, "W002");
        assert_eq!(diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_lint_known_function() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("groupBy(field=src)");
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_lint_pipeline_with_unknown_function() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 404 | myCustomFunc(x)");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "W002");
    }

    #[test]
    fn test_lint_multiple_issues() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 404 | unknownA() | unknownB()");
        let w002_count = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002_count, 2);
    }

    #[test]
    fn test_lint_empty_query() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_lint_negated_unknown_function() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("!notARealFunc(ip)");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "W002");
    }

    #[test]
    fn test_lint_nested_function_call_unknown() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("result := unknownFunc(x)");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "W002");
    }

    #[test]
    fn test_lint_in_function() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("in(field, values=[a, b, c])");
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            diagnostics
        );
    }

    // ---- W001: wildcard-only filter ----

    #[test]
    fn test_w001_wildcard_only() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("*");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 1);
    }

    #[test]
    fn test_w001_wildcard_not_triggered() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("*error*");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 0);
    }

    // ---- W003: missing args ----

    #[test]
    fn test_w003_groupby_no_args() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("groupBy()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_groupby_with_args() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("groupBy(field=src)");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 0);
    }

    #[test]
    fn test_w003_count_no_args_ok() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("count()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 0);
    }

    #[test]
    fn test_w003_sum_no_args() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("sum()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    // ---- W004: pipe style ----

    #[test]
    fn test_w004_pipe_no_spaces() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status=200|count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 1);
    }

    #[test]
    fn test_w004_pipe_with_spaces() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 | count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_missing_left_space() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status=200| count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 1);
    }

    // ---- W005: ambiguous precedence ----

    #[test]
    fn test_w005_and_or_no_parens() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("x=1 and y=2 or z=3");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert_eq!(w005, 1);
    }

    #[test]
    fn test_w005_and_or_with_parens() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("x=1 and (y=2 or z=3)");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert_eq!(w005, 0);
    }

    #[test]
    fn test_w005_only_and_no_warning() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("x=1 and y=2 and z=3");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert_eq!(w005, 0);
    }

    // ---- W006: filter order ----

    #[test]
    fn test_w006_filter_after_aggregate() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("count() | status = 200");
        let w006 = diagnostics.iter().filter(|d| d.rule_id == "W006").count();
        assert_eq!(w006, 1);
    }

    #[test]
    fn test_w006_filter_before_aggregate_ok() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 | count()");
        let w006 = diagnostics.iter().filter(|d| d.rule_id == "W006").count();
        assert_eq!(w006, 0);
    }

    #[test]
    fn test_w006_groupby_then_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("groupBy(field=src) | status = 200");
        let w006 = diagnostics.iter().filter(|d| d.rule_id == "W006").count();
        assert_eq!(w006, 1);
    }

    // ---- W002: unknown function (additional coverage) ----

    #[test]
    fn test_w002_in_case_statement() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("case { status=200 | unknownFunc() ; * | count() }");
        let w002: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == "W002").collect();
        assert_eq!(w002.len(), 1);
        assert!(w002[0].message.contains("unknownFunc"));
    }

    #[test]
    fn test_w002_in_match_statement() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint(r#"status match { "200" => unknownFunc() ; * => count() }"#);
        let w002: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == "W002").collect();
        assert_eq!(w002.len(), 1);
        assert!(w002[0].message.contains("unknownFunc"));
    }

    #[test]
    fn test_w002_in_or_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("unknownFunc(x) or count()");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_field_shorthand() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("field =~ unknownFunc(pattern=x)");
        let w002: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == "W002").collect();
        assert_eq!(w002.len(), 1);
        assert!(w002[0].message.contains("unknownFunc"));
    }

    #[test]
    fn test_w002_in_binary_op() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("result := unknownFunc(x) + 1");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_unary_op() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("result := -unknownFunc(x)");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_array_expr() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("table(fields=[unknownFunc()])");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_subquery() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("join({unknownFunc()}, field=key)");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_compare_expr() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("test(unknownFunc(x) != 0)");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    #[test]
    fn test_w002_in_named_arg() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("table(fields=unknownFunc())");
        let w002 = diagnostics.iter().filter(|d| d.rule_id == "W002").count();
        assert_eq!(w002, 1);
    }

    // ---- W001: wildcard-only filter (additional coverage) ----

    #[test]
    fn test_w001_in_not_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("not *");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 1);
    }

    #[test]
    fn test_w001_in_grouped_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("(*)");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 1);
    }

    #[test]
    fn test_w001_in_or_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("* or status=200");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 1);
    }

    #[test]
    fn test_w001_in_and_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("* and status=200");
        let w001 = diagnostics.iter().filter(|d| d.rule_id == "W001").count();
        assert_eq!(w001, 1);
    }

    // ---- W003: missing args (additional coverage) ----

    #[test]
    fn test_w003_in_case_statement() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("case { status=200 | groupBy() ; * | count() }");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_match_statement() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint(r#"status match { "200" => groupBy() ; * => count() }"#);
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_and_filter() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("groupBy() and count()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_field_shorthand() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("field =~ match()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_binary_op() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("result := sum() + 1");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_unary_op() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("result := -sum()");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_array_expr() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("table(fields=[sum()])");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_subquery() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("join({groupBy()}, field=key)");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_compare_expr() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("test(sum() != 0)");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    #[test]
    fn test_w003_in_named_arg() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("table(fields=groupBy())");
        let w003 = diagnostics.iter().filter(|d| d.rule_id == "W003").count();
        assert_eq!(w003, 1);
    }

    // ---- W004: pipe style (additional coverage) ----

    #[test]
    fn test_w004_pipe_in_string_ignored() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint(r#"field = "val|ue" | count()"#);
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_escape_in_string() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint(r#"field = "val\"ue" | count()"#);
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_in_line_comment_ignored() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 | count() // note|pipe");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_in_block_comment_ignored() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 /* a|b */ | count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_single_stage_skipped() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_missing_right_space() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 |count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 1);
    }

    #[test]
    fn test_w004_pipe_after_newline() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200\n| count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_before_newline() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200 |\ncount()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    #[test]
    fn test_w004_pipe_with_tab() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("status = 200\t| count()");
        let w004 = diagnostics.iter().filter(|d| d.rule_id == "W004").count();
        assert_eq!(w004, 0);
    }

    // ---- W005: ambiguous precedence (additional coverage) ----

    #[test]
    fn test_w005_only_or_no_warning() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("x=1 or y=2 or z=3");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert_eq!(w005, 0);
    }

    #[test]
    fn test_w005_and_or_mixed_multiple() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("a=1 and b=2 or c=3 and d=4");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert!(w005 >= 1);
    }

    #[test]
    fn test_w005_nested_or_in_and() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("(a=1 or b=2) and c=3 or d=4");
        let w005 = diagnostics.iter().filter(|d| d.rule_id == "W005").count();
        assert!(w005 >= 1);
    }

    #[test]
    fn test_lint_multibyte_error_message() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("これはテスト | count()");
        let e001: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == "E001").collect();
        assert!(!e001.is_empty());
        // バイト単位のエラーメッセージではなく、日本語テキストが含まれることを確認します
        for d in &e001 {
            assert!(
                !d.message.contains("'ã'"),
                "error message should not contain raw byte char: {}",
                d.message
            );
        }
    }

    #[test]
    fn test_lint_multibyte_recovery_detects_both_errors() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("あ | count() | い");
        let e001: Vec<_> = diagnostics.iter().filter(|d| d.rule_id == "E001").collect();
        assert!(
            e001.len() >= 2,
            "should detect both errors, got: {:?}",
            e001
        );
    }
}
