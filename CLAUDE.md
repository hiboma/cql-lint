# CLAUDE.md - cql-lint

## プロジェクト概要

CrowdStrike LogScale (旧 Humio) のクエリ言語 (CQL) に対する静的解析ツール (linter) です。
クエリ文字列を受け取り、構文エラーやベストプラクティス違反を検出・報告します。

- 言語: Rust (edition 2024)
- 対象: CrowdStrike Query Language (CQL)
- 仕様参照: https://library.humio.com/data-analysis/syntax.html

## ビルド・テスト

```bash
cargo build          # ビルド
cargo test           # 全テスト実行 (124 件)
cargo run -- FILE    # lint 実行
```

## CLI 使い方

```bash
cql-lint <file>...               # ファイルを lint する
echo 'query' | cql-lint          # 標準入力から lint する
cql-lint --format json <file>    # JSON 形式で出力する
cql-lint --disable W002 <file>   # ルールを無効化する
cql-lint --list-rules            # ルール一覧を表示する
cql-lint --verbose <file>        # lint 成功時にメッセージを表示する (-v でも可)
cql-lint --trim <file>           # 末尾空白を除去して整形済みクエリを出力する
cql-lint --trim --write <file>   # 末尾空白を除去してファイルに書き戻す
```

- 終了コード: エラー/警告あり=1, なし=0
- テキスト出力は miette によるソース位置付き表示です

## アーキテクチャ

3 層構造で実装しています。

```
CLI (main.rs)    -- clap + miette による入出力
Linter           -- Rule トレイト + LintEngine による AST 走査
Parser           -- 手書き再帰下降パーサー (Lexer → Token → AST)
```

処理の流れ: `ソース文字列 → Lexer → Vec<Token> → Parser → AST (Query) → LintEngine → Vec<Diagnostic>`

## ファイル構成

```
src/
├── main.rs                              # CLI エントリポイント (clap, miette)
├── lib.rs                               # ライブラリルート
├── diagnostic.rs                        # Diagnostic, Severity 型
├── lexer/
│   ├── mod.rs                           # Lexer 実装 (33 テスト)
│   └── token.rs                         # Token, TokenKind, Span 型
├── parser/
│   ├── mod.rs                           # Parser 実装 (41 テスト)
│   └── ast.rs                           # AST ノード定義
└── linter/
    ├── mod.rs                           # LintEngine (28 テスト)
    ├── rule.rs                          # Rule トレイト定義
    ├── known_functions.rs               # 組み込み関数リスト (140+)
    └── rules/
        ├── mod.rs                       # ルールモジュール一覧
        ├── syntax_error.rs              # E001
        ├── wildcard_only_filter.rs      # W001
        ├── unknown_function.rs          # W002
        ├── missing_args.rs              # W003
        ├── pipe_style.rs               # W004
        ├── ambiguous_precedence.rs      # W005
        └── filter_order.rs             # W006
testdata/                                # テスト用 .logscale ファイル
```

## CQL 構文仕様 (実装済み)

### Lexer が認識するトークン

| カテゴリ | トークン |
|---|---|
| リテラル | 整数, 浮動小数点, 文字列 (`"..."`), 正規表現 (`/pattern/flags`) |
| 識別子 | 通常 (`field`), `@` 付き (`@timestamp`), `#` 付き (`#event_simpleName`), 名前空間付き (`array:contains`) |
| キーワード | `and`, `or`, `not`, `like`, `as`, `case`, `match`, `in`, `true`, `false` (大文字も認識) |
| 演算子 | `=`, `!=`, `:=`, `<`, `<=`, `>`, `>=`, `<=>`, `\|`, `!`, `+`, `-`, `*`, `/` |
| 区切り記号 | `()`, `[]`, `{}`, `,`, `.`, `:` |
| ワイルドカード | `*foo*`, `error*`, `*` |
| コメント | `//` (単一行), `/* */` (複数行) |

### Parser が生成する AST

- `Query` -- パイプラインステージのリスト
- `PipelineStage` -- `StageKind` (Filter / FunctionCall / Assignment)
- `FilterExpr` -- FreeText, FieldFilter, And, Or, Not, Grouped, FunctionCall
- `FunctionCall` -- 関数名 + 引数リスト (位置引数, 名前付き引数)
- `Expr` -- Number, String, Bool, Field, Regex, Wildcard, Array, FunctionCall, BinaryOp, UnaryOp, SubQuery, CompareExpr
- `Assignment` -- フィールド名 + 式

### CQL 固有の注意点

- **OR は AND より結合が強い**: `a and b or c` は `a and (b or c)` と解釈されます
- **暗黙の AND**: `src="a" ip="b"` は `src="a" AND ip="b"` と同等です
- **`as` は名前付き引数名として使用可能**: `rename(field=X, as=Y)`
- **`in`, `match` はキーワードかつ関数名**: `in(field=X, values=[...])` のように使います
- **`#` 付きタグフィールド**: `#event_simpleName` のようにイベントタイプを参照します
- **サブクエリ**: `join({...})` のように `{}` 内にパイプラインを記述します
- **引数内の比較式**: `test(field != value)` のように関数引数内で比較演算子を使います

## Lint ルール一覧

| ID | 重大度 | カテゴリ | 説明 |
|---|---|---|---|
| `E001` | error | syntax | 構文エラー (パース失敗) |
| `W001` | warning | performance | `*` のみのフリーテキストフィルタ (全件マッチ) |
| `W002` | warning | correctness | 未知の関数名の使用 |
| `W003` | warning | correctness | 関数の必須引数が不足 |
| `W004` | info | style | パイプ `\|` の前後に空白がない |
| `W005` | warning | correctness | AND/OR 優先順位が曖昧な式 (括弧なしの混在) |
| `W006` | info | performance | 集約関数の後にフィルタを配置 |

## 新しいルールの追加方法

1. `src/linter/rules/` に新しいファイルを作成します
2. `Rule` トレイトを実装します (`id`, `description`, `check`)
3. `src/linter/rules/mod.rs` にモジュールを追加します
4. `src/linter/mod.rs` の `LintEngine::new()` にルールを登録します
5. `src/linter/mod.rs` の `#[cfg(test)]` にテストを追加します

```rust
// Rule トレイト
pub trait Rule {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, query: &Query, source: &str) -> Vec<Diagnostic>;
}
```

## 依存ライブラリ

| ライブラリ | 用途 |
|---|---|
| `clap` (4) | CLI 引数解析 |
| `miette` (7) | ソース位置付きエラー表示 |
| `serde` + `serde_json` (1) | JSON 出力, Serialize 導出 |
| `thiserror` (2) | エラー型定義 |
| `insta` (1, dev) | スナップショットテスト |

## Homebrew 配布

### 構成

- `Formula/cql-lint.rb` -- Formula テンプレート (このリポジトリに同梱)
- `github.com/hiboma/homebrew-tap` -- Homebrew tap リポジトリ (Formula の公開先)

### リリース時の手動更新手順

1. タグ push で GitHub Release が作成されるのを待ちます
2. リリースページから macOS / Linux の tar.gz をダウンロードし sha256 を取得します

```bash
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-aarch64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-x86_64-apple-darwin.tar.gz | shasum -a 256
curl -sL https://github.com/hiboma/cql-lint/releases/download/v0.1.0/cql-lint-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | shasum -a 256
```

3. `Formula/cql-lint.rb` の `version` と `sha256` を更新します
4. 更新した Formula を `hiboma/homebrew-tap` リポジトリにコピーして push します

### 自動化する場合

release.yml に homebrew-releaser アクションを追加すると、タグ push 時に tap リポジトリの Formula を自動更新できます。

1. GitHub の Settings > Developer settings > Personal access tokens で `repo` スコープの PAT を作成します
2. cql-lint リポジトリの Settings > Secrets に `HOMEBREW_TAP_TOKEN` として登録します
3. release.yml の `release` ジョブの後に以下のジョブを追加します

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

## コーディング規約

- ファイル末尾に改行を入れます (POSIX 仕様)
- テストは各モジュール内の `#[cfg(test)] mod tests` に記述します
- AST の走査はルールごとに再帰的に行います (`check_stage` → `check_filter` → `check_expr`)
- 新しい AST ノードを追加した場合、全ルールの `check_expr` / `check_filter` に分岐を追加します

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
