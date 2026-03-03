<h1 align="center">🐤🪵⚖️<br>cql-lint</h1>

<p align="center">
<strong>Beta</strong>: This tool is in beta. Syntax coverage and lint rules are still limited. Expect breaking changes.
</p>

A linter for [CrowdStrike LogScale](https://www.crowdstrike.com/platform/next-gen-siem/logscale/) query language (CQL).
It detects syntax errors, potential bugs, and style issues in CQL queries.

## Features

- Syntax validation of CQL queries
- Detection of unknown function names (140+ built-in functions supported)
- Warnings for missing required arguments
- Style checks (pipe formatting, operator precedence)
- Performance hints (wildcard-only filters, filter ordering)
- JSON output format for CI integration
- Auto-fix: trailing whitespace trimming with `--trim`

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap hiboma/tap
brew install cql-lint
```

### Download from releases

Download a pre-built binary from the [Releases](https://github.com/hiboma/cql-lint/releases) page.

Available platforms:

- Linux x86_64
- macOS x86_64 / ARM (Apple Silicon)
- Windows x86_64

### Build from source

Requires Rust 1.85 or later.

```bash
git clone https://github.com/hiboma/cql-lint.git
cd cql-lint
cargo build --release
# Binary is at target/release/cql-lint
```

## Usage

```bash
# Lint files
cql-lint query.logscale

# Lint from stdin
echo '#event_simpleName=ProcessRollup2 | count()' | cql-lint

# JSON output
cql-lint --format json query.logscale

# Disable specific rules
cql-lint --disable W002 query.logscale

# List all rules
cql-lint --list-rules

# List supported functions
cql-lint --list-functions

# Trim trailing whitespace and print
cql-lint --trim query.logscale

# Trim and write back to file
cql-lint --trim --write query.logscale

# Show success message
cql-lint --verbose query.logscale
```

### Exit codes

| Code | Meaning |
|------|---------|
| `0`  | No issues found |
| `1`  | Errors or warnings found |
| `2`  | Failed to read input |

## Lint Rules

| ID   | Severity | Category    | Description |
|------|----------|-------------|-------------|
| E001 | error    | syntax      | Syntax error (parse failure) |
| W001 | warning  | performance | Wildcard-only free text filter (`*` matches everything) |
| W002 | warning  | correctness | Unknown function name |
| W003 | warning  | correctness | Missing required arguments |
| W004 | info     | style       | No whitespace around pipe `\|` |
| W005 | warning  | correctness | Ambiguous AND/OR precedence (mixed without parentheses) |
| W006 | info     | performance | Filter placed after aggregation function |

## CQL Reference

This tool targets the query language documented at:
https://library.humio.com/data-analysis/syntax.html

### Notable CQL behaviors

- `OR` binds more tightly than `AND`: `a AND b OR c` is parsed as `a AND (b OR c)`
- Implicit AND: `src="a" ip="b"` is equivalent to `src="a" AND ip="b"`
- Tag fields use `#` prefix: `#event_simpleName`
- Sub-queries use `{}`: `join({...})`

## License

[MIT](LICENSE)
