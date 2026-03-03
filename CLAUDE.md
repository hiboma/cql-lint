# CLAUDE.md - cql-lint

## Project Overview

A static analysis tool (linter) for CrowdStrike LogScale (formerly Humio) Query Language (CQL).
It takes query strings as input and detects syntax errors and best practice violations.

- Language: Rust (edition 2024)
- Target: CrowdStrike Query Language (CQL)
- Specification: https://library.humio.com/data-analysis/syntax.html

## Build & Test

```bash
cargo build          # Build
cargo test           # Run all tests (124 cases)
cargo run -- FILE    # Run lint
```

## CLI Usage

```bash
cql-lint <file>...               # Lint files
echo 'query' | cql-lint          # Lint from stdin
cql-lint --format json <file>    # Output in JSON format
cql-lint --disable W002 <file>   # Disable a rule
cql-lint --list-rules            # List all rules
cql-lint --verbose <file>        # Show message on lint success (-v also works)
cql-lint --trim <file>           # Remove trailing whitespace and print formatted query
cql-lint --trim --write <file>   # Remove trailing whitespace and write back to file
```

- Exit code: 1 if errors/warnings found, 0 otherwise
- Text output uses miette for source-location-annotated display

## Architecture

The project is implemented in a 3-layer structure.

```
CLI (main.rs)    -- I/O via clap + miette
Linter           -- AST traversal via Rule trait + LintEngine
Parser           -- Hand-written recursive descent parser (Lexer -> Token -> AST)
```

Processing pipeline: `Source string -> Lexer -> Vec<Token> -> Parser -> AST (Query) -> LintEngine -> Vec<Diagnostic>`

## File Structure

```
src/
├── main.rs                              # CLI entry point (clap, miette)
├── lib.rs                               # Library root
├── diagnostic.rs                        # Diagnostic, Severity types
├── lexer/
│   ├── mod.rs                           # Lexer implementation (33 tests)
│   └── token.rs                         # Token, TokenKind, Span types
├── parser/
│   ├── mod.rs                           # Parser implementation (41 tests)
│   └── ast.rs                           # AST node definitions
└── linter/
    ├── mod.rs                           # LintEngine (28 tests)
    ├── rule.rs                          # Rule trait definition
    ├── known_functions.rs               # Built-in function list (140+)
    └── rules/
        ├── mod.rs                       # Rule module index
        ├── syntax_error.rs              # E001
        ├── wildcard_only_filter.rs      # W001
        ├── unknown_function.rs          # W002
        ├── missing_args.rs              # W003
        ├── pipe_style.rs               # W004
        ├── ambiguous_precedence.rs      # W005
        └── filter_order.rs             # W006
testdata/                                # Test .logscale files
```

## CQL Syntax Specification (Implemented)

### Tokens Recognized by Lexer

| Category | Tokens |
|---|---|
| Literals | Integer, Float, String (`"..."`), Regex (`/pattern/flags`) |
| Identifiers | Plain (`field`), `@`-prefixed (`@timestamp`), `#`-prefixed (`#event_simpleName`), Namespaced (`array:contains`) |
| Keywords | `and`, `or`, `not`, `like`, `as`, `case`, `match`, `in`, `true`, `false` (case-insensitive) |
| Operators | `=`, `!=`, `:=`, `<`, `<=`, `>`, `>=`, `<=>`, `\|`, `!`, `+`, `-`, `*`, `/` |
| Delimiters | `()`, `[]`, `{}`, `,`, `.`, `:` |
| Wildcards | `*foo*`, `error*`, `*` |
| Comments | `//` (single-line), `/* */` (multi-line) |

### AST Produced by Parser

- `Query` -- List of pipeline stages
- `PipelineStage` -- `StageKind` (Filter / FunctionCall / Assignment)
- `FilterExpr` -- FreeText, FieldFilter, And, Or, Not, Grouped, FunctionCall
- `FunctionCall` -- Function name + argument list (positional args, named args)
- `Expr` -- Number, String, Bool, Field, Regex, Wildcard, Array, FunctionCall, BinaryOp, UnaryOp, SubQuery, CompareExpr
- `Assignment` -- Field name + expression

### CQL-Specific Notes

- **OR binds tighter than AND**: `a and b or c` is interpreted as `a and (b or c)`
- **Implicit AND**: `src="a" ip="b"` is equivalent to `src="a" AND ip="b"`
- **`as` can be used as a named argument**: `rename(field=X, as=Y)`
- **`in` and `match` are both keywords and function names**: Used as `in(field=X, values=[...])`
- **`#`-prefixed tag fields**: Reference event types like `#event_simpleName`
- **Subqueries**: Write pipelines inside `{}`, e.g., `join({...})`
- **Comparison expressions in arguments**: Comparison operators can be used inside function arguments, e.g., `test(field != value)`

## Lint Rules

| ID | Severity | Category | Description |
|---|---|---|---|
| `E001` | error | syntax | Syntax error (parse failure) |
| `W001` | warning | performance | Free-text filter with only `*` (matches everything) |
| `W002` | warning | correctness | Unknown function name |
| `W003` | warning | correctness | Missing required function arguments |
| `W004` | info | style | No whitespace around pipe `\|` |
| `W005` | warning | correctness | Ambiguous AND/OR precedence (mixed without parentheses) |
| `W006` | info | performance | Filter placed after aggregation function |

## Adding a New Rule

1. Create a new file in `src/linter/rules/`
2. Implement the `Rule` trait (`id`, `description`, `check`)
3. Add the module to `src/linter/rules/mod.rs`
4. Register the rule in `LintEngine::new()` in `src/linter/mod.rs`
5. Add tests in `#[cfg(test)]` in `src/linter/mod.rs`

```rust
// Rule trait
pub trait Rule {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, query: &Query, source: &str) -> Vec<Diagnostic>;
}
```

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` (4) | CLI argument parsing |
| `miette` (7) | Source-location-annotated error display |
| `serde` + `serde_json` (1) | JSON output, Serialize derive |
| `thiserror` (2) | Error type definitions |
| `insta` (1, dev) | Snapshot testing |

## Homebrew Distribution

### Components

- `Formula/cql-lint.rb` -- Formula template (included in this repository)
- `github.com/hiboma/homebrew-tap` -- Homebrew tap repository (where the Formula is published)

### Manual Update Steps on Release

1. Wait for a GitHub Release to be created by a tag push
2. Download the macOS / Linux tar.gz from the release page and get the sha256

```bash
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-aarch64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-x86_64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | shasum -a 256
```

3. Update `version` and `sha256` in `Formula/cql-lint.rb`
4. Copy the updated Formula to the `hiboma/homebrew-tap` repository and push

### Automation

Adding a homebrew-releaser action to release.yml enables automatic Formula updates in the tap repository on tag push.

1. Create a PAT with `repo` scope at GitHub Settings > Developer settings > Personal access tokens
2. Register it as `HOMEBREW_TAP_TOKEN` in cql-lint repository Settings > Secrets
3. Add the following job after the `release` job in release.yml

```yaml
  homebrew:
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: Justintime50/homebrew-releaser@v1
        with:
          homebrew_owner: hiboma
          homebrew_tap: homebrew-tap
          formula_folder: Formula
          github_token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          commit_owner: hiboma
          commit_email: hiboma@users.noreply.github.com
          install: bin.install "cql-lint"
          test: |
            output = shell_output("echo 'count()' | #{bin}/cql-lint --verbose")
            assert_match "no issues found", output
          target_darwin_amd64: true
          target_darwin_arm64: true
          target_linux_amd64: true
          target_linux_arm64: false
```

## Coding Conventions

- Files must end with a newline (POSIX compliance)
- Tests are written in `#[cfg(test)] mod tests` within each module
- AST traversal is performed recursively per rule (`check_stage` -> `check_filter` -> `check_expr`)
- When adding a new AST node, add branches to `check_expr` / `check_filter` in all rules

## GitHub Actions Security Policy

Reference: https://seclists.org/oss-sec/2026/q1/246

### Triggers

- Do not use `pull_request_target`. Use `pull_request` instead.
- Do not run privileged operations on PRs from forks.

### Permissions

- Explicitly set `permissions` at the workflow top level to restrict defaults.
- Grant only the minimum required permissions per job.
- Limit `contents: write` to jobs that need it (e.g., release jobs).

### Action Pinning

- Pin third-party actions by commit SHA. Include the tag as an inline comment.
- Use `pinact` to manage hash updates.
- Example: `uses: actions/checkout@<commit-sha> # v4`

### Shell Scripts

- Do not interpolate user-controlled inputs directly in inline shell scripts.
- Pass values like `${{ github.event.pull_request.title }}` through environment variables instead.
