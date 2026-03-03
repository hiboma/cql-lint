use std::io::Read;
use std::process;

use clap::Parser;
use miette::{LabeledSpan, Severity as MietteSeverity, miette};

use cql_lint::diagnostic::{Diagnostic, Severity};
use cql_lint::linter::LintEngine;
use cql_lint::linter::known_functions::KNOWN_FUNCTION_ENTRIES;

#[derive(Parser)]
#[command(
    name = "cql-lint",
    version,
    about = "A linter for CrowdStrike LogScale query language"
)]
struct Cli {
    /// lint 対象のファイルパス (指定しない場合は標準入力から読み込みます)
    files: Vec<String>,

    /// 出力フォーマット
    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    format: String,

    /// 無効にするルール ID (カンマ区切り)
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,

    /// 登録されているルールの一覧を表示します
    #[arg(long)]
    list_rules: bool,

    /// サポートしている関数の一覧を表示します
    #[arg(long)]
    list_functions: bool,

    /// 末尾の空白文字・余分な改行を除去して整形済みクエリを出力します
    #[arg(long)]
    trim: bool,

    /// --trim と併用し、整形結果をファイルに直接書き戻します
    #[arg(long, requires = "trim")]
    write: bool,

    /// lint 成功時にメッセージを表示します
    #[arg(long, short)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.list_rules {
        print_rules();
        return;
    }

    if cli.list_functions {
        print_functions();
        return;
    }

    let engine = LintEngine::new();
    let mut has_errors = false;

    let mut file_count: usize = 0;

    if cli.files.is_empty() {
        // 標準入力から読み込みます
        let mut source = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut source) {
            eprintln!("error: failed to read stdin: {}", e);
            process::exit(2);
        }
        file_count = 1;
        if cli.trim {
            let trimmed = trim_source(&source);
            let diagnostics = run_lint(&engine, &trimmed, &cli.disable);
            if !diagnostics.is_empty() {
                has_errors = true;
                print_diagnostics(&diagnostics, &trimmed, "<stdin>", &cli.format);
            } else {
                print!("{}", trimmed);
            }
        } else {
            let diagnostics = run_lint(&engine, &source, &cli.disable);
            if !diagnostics.is_empty() {
                has_errors = true;
                print_diagnostics(&diagnostics, &source, "<stdin>", &cli.format);
            }
        }
    } else {
        for path in &cli.files {
            match std::fs::read_to_string(path) {
                Ok(source) => {
                    file_count += 1;
                    if cli.trim {
                        let trimmed = trim_source(&source);
                        let diagnostics = run_lint(&engine, &trimmed, &cli.disable);
                        if !diagnostics.is_empty() {
                            has_errors = true;
                            print_diagnostics(&diagnostics, &trimmed, path, &cli.format);
                        } else if cli.write {
                            if let Err(e) = std::fs::write(path, &trimmed) {
                                eprintln!("error: failed to write '{}': {}", path, e);
                                has_errors = true;
                            }
                        } else {
                            print!("{}", trimmed);
                        }
                    } else {
                        let diagnostics = run_lint(&engine, &source, &cli.disable);
                        if !diagnostics.is_empty() {
                            has_errors = true;
                            print_diagnostics(&diagnostics, &source, path, &cli.format);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to read '{}': {}", path, e);
                    has_errors = true;
                }
            }
        }
    }

    if has_errors {
        process::exit(1);
    }

    if cli.verbose {
        println!("ok: {} file(s) checked, no issues found.", file_count);
    }
}

fn run_lint(engine: &LintEngine, source: &str, disable: &[String]) -> Vec<Diagnostic> {
    engine
        .lint(source)
        .into_iter()
        .filter(|d| !disable.iter().any(|id| id == &d.rule_id))
        .collect()
}

fn format_rules() -> String {
    let engine = LintEngine::new();
    engine
        .rules()
        .iter()
        .map(|rule| format!("{}: {}", rule.id(), rule.description()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_rules() {
    println!("{}", format_rules());
}

fn format_functions() -> String {
    let mut lines = Vec::new();
    let mut current_category = "";
    for entry in KNOWN_FUNCTION_ENTRIES {
        if entry.category != current_category {
            if !current_category.is_empty() {
                lines.push(String::new());
            }
            lines.push(format!("[{}]", entry.category));
            current_category = entry.category;
        }
        lines.push(format!("  {}", entry.name));
    }
    lines.join("\n")
}

fn print_functions() {
    println!("{}", format_functions());
}

fn print_diagnostics(diagnostics: &[Diagnostic], source: &str, filename: &str, format: &str) {
    match format {
        "json" => print_json(diagnostics, filename),
        _ => print_text(diagnostics, source, filename),
    }
}

#[derive(serde::Serialize)]
struct JsonOutput<'a> {
    file: &'a str,
    diagnostics: &'a [Diagnostic],
}

fn format_json(diagnostics: &[Diagnostic], filename: &str) -> Option<String> {
    let output = JsonOutput {
        file: filename,
        diagnostics,
    };
    serde_json::to_string_pretty(&output).ok()
}

fn print_json(diagnostics: &[Diagnostic], filename: &str) {
    if let Some(json) = format_json(diagnostics, filename) {
        println!("{}", json);
    }
}

fn print_text(diagnostics: &[Diagnostic], source: &str, filename: &str) {
    for d in diagnostics {
        let (line, col) = offset_to_line_col(source, d.span.start);
        let miette_severity = match d.severity {
            Severity::Error => MietteSeverity::Error,
            Severity::Warning => MietteSeverity::Warning,
            Severity::Info => MietteSeverity::Advice,
        };

        let span_len = if d.span.end > d.span.start {
            d.span.end - d.span.start
        } else {
            1
        };

        let report = miette!(
            severity = miette_severity,
            labels = vec![LabeledSpan::at(
                d.span.start..d.span.start + span_len,
                &d.rule_id
            )],
            "{}:{}:{}: [{}] {}",
            filename,
            line,
            col,
            d.rule_id,
            d.message
        )
        .with_source_code(source.to_string());

        eprintln!("{:?}", report);
    }
}

/// 各行の末尾の空白文字を除去し、ファイル末尾を改行 1 つで終端します。
/// トップレベル (サブクエリ外) の行頭空白を除去します。
/// コード部分とインラインコメントの間の余分な空白も 1 つに詰めます。
fn trim_source(source: &str) -> String {
    let mut brace_depth: usize = 0;
    let mut result: String = source
        .lines()
        .map(|line| {
            let trimmed = line.trim_end();
            // トップレベルの場合のみ行頭空白を除去します
            let trimmed = if brace_depth == 0 {
                trimmed.trim_start()
            } else {
                trimmed
            };
            // この行の括弧を数えて深さを更新します
            // 文字列リテラル内の括弧は無視します
            brace_depth = update_nesting_depth(trimmed, brace_depth);
            collapse_before_comment(trimmed)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 末尾に改行が 1 つだけ入るようにします (POSIX 準拠)
    if !result.is_empty() {
        result.push('\n');
    }

    result
}

/// 文字列リテラルを考慮しながら行内の括弧 (`{}`, `[]`, `()`) の深さを更新します。
fn update_nesting_depth(line: &str, depth: usize) -> usize {
    let bytes = line.as_bytes();
    let mut d = depth;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
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
                // コメント以降は括弧を数えません
                break;
            }
            b'{' | b'[' | b'(' => {
                d += 1;
                i += 1;
            }
            b'}' | b']' | b')' => {
                d = d.saturating_sub(1);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    d
}

/// コード部分とインラインコメント (`//`) の間の空白を 1 つに詰めます。
/// 文字列リテラル内の `//` は無視します。
fn collapse_before_comment(line: &str) -> String {
    // 行全体がコメントで始まる場合はそのまま返します
    let stripped = line.trim_start();
    if stripped.starts_with("//") {
        return line.to_string();
    }

    // 文字列リテラルを考慮しながら `//` の位置を探します
    let bytes = line.as_bytes();
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
                // インラインコメントを検出しました
                let code_part = line[..i].trim_end();
                let comment_part = &line[i..];
                if code_part.is_empty() {
                    return line.to_string();
                }
                return format!("{} {}", code_part, comment_part);
            }
            _ => {
                i += 1;
            }
        }
    }

    line.to_string()
}

/// バイトオフセットを行番号・列番号 (1-based) に変換します。
fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_line_col_start() {
        assert_eq!(offset_to_line_col("hello", 0), (1, 1));
    }

    #[test]
    fn test_offset_to_line_col_middle() {
        assert_eq!(offset_to_line_col("hello", 3), (1, 4));
    }

    #[test]
    fn test_offset_to_line_col_newline() {
        assert_eq!(offset_to_line_col("hello\nworld", 6), (2, 1));
    }

    #[test]
    fn test_offset_to_line_col_second_line() {
        assert_eq!(offset_to_line_col("hello\nworld", 8), (2, 3));
    }

    #[test]
    fn test_trim_source_trailing_spaces() {
        assert_eq!(trim_source("hello   \nworld  \n"), "hello\nworld\n");
    }

    #[test]
    fn test_trim_source_trailing_tabs() {
        assert_eq!(trim_source("hello\t\t\nworld\n"), "hello\nworld\n");
    }

    #[test]
    fn test_trim_source_multiple_trailing_newlines() {
        assert_eq!(trim_source("hello\n\n\n\n"), "hello\n\n\n\n");
    }

    #[test]
    fn test_trim_source_no_trailing_newline() {
        assert_eq!(trim_source("hello"), "hello\n");
    }

    #[test]
    fn test_trim_source_empty() {
        assert_eq!(trim_source(""), "");
    }

    #[test]
    fn test_trim_source_mixed_whitespace() {
        assert_eq!(
            trim_source("status = 200   \t\n| count()  \n"),
            "status = 200\n| count()\n"
        );
    }

    #[test]
    fn test_trim_source_inline_comment_extra_spaces() {
        assert_eq!(
            trim_source("| hoge        // comment\n"),
            "| hoge // comment\n"
        );
    }

    #[test]
    fn test_trim_source_inline_comment_already_single_space() {
        assert_eq!(trim_source("| hoge // comment\n"), "| hoge // comment\n");
    }

    #[test]
    fn test_trim_source_full_line_comment_preserved() {
        assert_eq!(
            trim_source("// this is a comment\n"),
            "// this is a comment\n"
        );
    }

    #[test]
    fn test_trim_source_indented_full_line_comment_at_toplevel() {
        // トップレベルの行頭空白は除去されます
        assert_eq!(
            trim_source("  // indented comment\n"),
            "// indented comment\n"
        );
    }

    #[test]
    fn test_trim_source_comment_in_string_not_collapsed() {
        // 文字列内の // はコメントとして扱いません
        assert_eq!(
            trim_source(r#"url = "http://example.com"        // comment"#),
            "url = \"http://example.com\" // comment\n"
        );
    }

    #[test]
    fn test_trim_source_no_comment() {
        assert_eq!(trim_source("| count()   \n"), "| count()\n");
    }

    #[test]
    fn test_trim_source_leading_spaces_at_toplevel() {
        assert_eq!(
            trim_source("  status = 200\n  | count()\n"),
            "status = 200\n| count()\n"
        );
    }

    #[test]
    fn test_trim_source_subquery_indent_preserved() {
        let input = "| join({\n    count()\n    | table()\n  }, field=key)\n";
        let expected = "| join({\n    count()\n    | table()\n  }, field=key)\n";
        assert_eq!(trim_source(input), expected);
    }

    #[test]
    fn test_trim_source_nested_subquery_indent_preserved() {
        let input = "| join({\n    inner({\n        count()\n    })\n  })\n";
        let expected = "| join({\n    inner({\n        count()\n    })\n  })\n";
        assert_eq!(trim_source(input), expected);
    }

    #[test]
    fn test_trim_source_multiline_array_indent_preserved() {
        let input = "| table(fields=[\n    @timestamp,\n    aip\n  ])\n";
        let expected = "| table(fields=[\n    @timestamp,\n    aip\n  ])\n";
        assert_eq!(trim_source(input), expected);
    }

    #[test]
    fn test_trim_source_multiline_function_args_indent_preserved() {
        let input = "| format(\n    format=\"%s\",\n    field=[a, b]\n  )\n";
        let expected = "| format(\n    format=\"%s\",\n    field=[a, b]\n  )\n";
        assert_eq!(trim_source(input), expected);
    }

    // ---- update_nesting_depth ----

    #[test]
    fn test_update_nesting_depth_open_brace() {
        assert_eq!(update_nesting_depth("{", 0), 1);
    }

    #[test]
    fn test_update_nesting_depth_close_brace() {
        assert_eq!(update_nesting_depth("}", 1), 0);
    }

    #[test]
    fn test_update_nesting_depth_saturating_sub() {
        assert_eq!(update_nesting_depth("}", 0), 0);
    }

    #[test]
    fn test_update_nesting_depth_string_literal() {
        assert_eq!(update_nesting_depth(r#""{""#, 0), 0);
    }

    #[test]
    fn test_update_nesting_depth_escaped_quote_in_string() {
        assert_eq!(update_nesting_depth(r#""\"{"#, 0), 0);
    }

    #[test]
    fn test_update_nesting_depth_comment() {
        assert_eq!(update_nesting_depth("code // {", 0), 0);
    }

    // ---- collapse_before_comment ----

    #[test]
    fn test_collapse_before_comment_no_comment() {
        assert_eq!(collapse_before_comment("| count()"), "| count()");
    }

    #[test]
    fn test_collapse_before_comment_full_line() {
        assert_eq!(
            collapse_before_comment("// full line comment"),
            "// full line comment"
        );
    }

    #[test]
    fn test_collapse_before_comment_string_with_slashes() {
        assert_eq!(
            collapse_before_comment(r#"url = "http://example.com"     // comment"#),
            r#"url = "http://example.com" // comment"#
        );
    }

    #[test]
    fn test_collapse_before_comment_escaped_string() {
        assert_eq!(
            collapse_before_comment(r#"field = "val\"ue"     // comment"#),
            r#"field = "val\"ue" // comment"#
        );
    }

    // ---- run_lint ----

    #[test]
    fn test_run_lint_no_disable() {
        let engine = LintEngine::new();
        let diagnostics = run_lint(&engine, "unknownFunc()", &[]);
        assert!(diagnostics.iter().any(|d| d.rule_id == "W002"));
    }

    #[test]
    fn test_run_lint_disable_rule() {
        let engine = LintEngine::new();
        let diagnostics = run_lint(&engine, "unknownFunc()", &["W002".to_string()]);
        assert!(diagnostics.iter().all(|d| d.rule_id != "W002"));
    }

    #[test]
    fn test_run_lint_disable_multiple_rules() {
        let engine = LintEngine::new();
        let diagnostics = run_lint(
            &engine,
            "unknownFunc()",
            &["W002".to_string(), "W003".to_string()],
        );
        assert!(diagnostics.is_empty());
    }

    // ---- format_json ----

    #[test]
    fn test_format_json_with_diagnostics() {
        let engine = LintEngine::new();
        let diagnostics = engine.lint("unknownFunc()");
        let json = format_json(&diagnostics, "test.logscale").unwrap();
        assert!(json.contains("test.logscale"));
        assert!(json.contains("W002"));
    }

    #[test]
    fn test_format_json_empty_diagnostics() {
        let json = format_json(&[], "test.logscale").unwrap();
        assert!(json.contains("\"diagnostics\": []"));
    }

    // ---- format_rules / format_functions ----

    #[test]
    fn test_format_rules_contains_all_rule_ids() {
        let output = format_rules();
        assert!(output.contains("E001"));
        assert!(output.contains("W001"));
        assert!(output.contains("W002"));
        assert!(output.contains("W003"));
        assert!(output.contains("W004"));
        assert!(output.contains("W005"));
        assert!(output.contains("W006"));
    }

    #[test]
    fn test_format_functions_contains_categories() {
        let output = format_functions();
        assert!(output.contains("[Aggregate]"));
        assert!(output.contains("count"));
    }
}
