pub mod ast;

use crate::lexer::token::{Span, Token, TokenKind};
use ast::*;

/// パースエラーを表します。
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

/// CQL の構文解析器です。
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// クエリ文字列をパースして AST とエラーのリストを返します。
    pub fn parse(mut self) -> (Option<Query>, Vec<ParseError>) {
        let query = self.parse_query();
        (query, self.errors)
    }

    // ---- ユーティリティ ----

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or_else(|| {
                self.tokens
                    .last()
                    .map(|t| Span::new(t.span.end, t.span.end))
                    .unwrap_or(Span::new(0, 0))
            })
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            span: self.current_span(),
        });
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParseError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("expected {:?}, found {:?}", expected, self.peek()),
                span,
            ))
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    /// 現在位置からドット区切り識別子が続く場合、そのトークン数を返します。
    /// 例: `aip.city` → Identifier + Dot + Identifier で追加 2 トークン分を返します。
    fn dotted_identifier_extra(&self, base: usize) -> usize {
        let mut extra = 0;
        loop {
            let dot_pos = base + extra;
            let ident_pos = base + extra + 1;
            let is_dot = matches!(
                self.tokens.get(dot_pos).map(|t| &t.kind),
                Some(TokenKind::Dot)
            );
            let is_ident = matches!(
                self.tokens.get(ident_pos).map(|t| &t.kind),
                Some(TokenKind::Identifier(_))
            );
            if is_dot && is_ident {
                extra += 2;
            } else {
                break;
            }
        }
        extra
    }

    /// 識別子名を受け取り、後続の `.identifier` トークンを消費して結合した名前を返します。
    fn consume_dotted_suffix(&mut self, mut name: String) -> String {
        while matches!(self.peek(), TokenKind::Dot) {
            if let Some(TokenKind::Identifier(_)) = self.tokens.get(self.pos + 1).map(|t| &t.kind) {
                self.advance(); // `.` を消費します
                let tok = self.advance();
                if let TokenKind::Identifier(part) = tok.kind {
                    name.push('.');
                    name.push_str(&part);
                }
            } else {
                break;
            }
        }
        name
    }

    /// 現在のトークンがパイプラインステージの終端か判定します。
    fn at_stage_end(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Pipe | TokenKind::Eof | TokenKind::Semicolon | TokenKind::FatArrow
        )
    }

    /// ステージパース失敗時のエラーを報告します。
    /// Error トークンの場合はその内容を、それ以外の場合は指定メッセージを使います。
    fn report_stage_error(&mut self, fallback_msg: &str) {
        let span = self.current_span();
        let message = if let TokenKind::Error(msg) = self.peek() {
            msg.clone()
        } else {
            fallback_msg.to_string()
        };
        self.errors.push(ParseError::new(message, span));
    }

    /// 次のパイプまたは EOF までトークンを読み飛ばします。
    /// エラーリカバリに使用します。
    fn skip_to_next_pipe(&mut self) {
        while !matches!(self.peek(), TokenKind::Pipe | TokenKind::Eof) {
            self.advance();
        }
    }

    // ---- パース関数 ----

    /// query = stage ("|" stage)*
    fn parse_query(&mut self) -> Option<Query> {
        let start = self.current_span().start;
        let mut stages = Vec::new();

        if self.at_end() {
            return Some(Query {
                stages,
                span: Span::new(start, start),
            });
        }

        match self.parse_stage() {
            Some(stage) => stages.push(stage),
            None => {
                if !self.at_end() {
                    self.report_stage_error("expected a pipeline stage");
                    // 次のパイプまでスキップしてリカバリします
                    self.skip_to_next_pipe();
                }
            }
        }

        loop {
            if matches!(self.peek(), TokenKind::Pipe) {
                self.advance(); // `|` を消費します
                match self.parse_stage() {
                    Some(stage) => stages.push(stage),
                    None => {
                        self.report_stage_error("expected a pipeline stage after '|'");
                        // 次のパイプまでスキップしてリカバリします
                        self.skip_to_next_pipe();
                    }
                }
            } else if !self.at_end() {
                self.report_stage_error("unexpected token");
                // 次のパイプまでスキップしてリカバリします
                self.skip_to_next_pipe();
            } else {
                break;
            }
        }

        let end = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(start);

        Some(Query {
            stages,
            span: Span::new(start, end),
        })
    }

    /// stage = assignment | function_call | filter_expr
    fn parse_stage(&mut self) -> Option<PipelineStage> {
        if self.at_stage_end() {
            return None;
        }

        // Error トークンの場合は None を返します
        // エラー報告は呼び出し元で行います
        if matches!(self.peek(), TokenKind::Error(_)) {
            return None;
        }

        let start = self.current_span().start;

        // 識別子の後に `:=` が続く場合は代入です
        if self.is_assignment() {
            return self.parse_assignment(start);
        }

        // case { ... } の場合は case 文です
        if matches!(self.peek(), TokenKind::Case)
            && matches!(
                self.tokens.get(self.pos + 1).map(|t| &t.kind),
                Some(TokenKind::LBrace)
            )
        {
            return self.parse_case_statement(start);
        }

        // field match { ... } の場合は match 文です
        if self.is_match_statement() {
            return self.parse_match_statement(start);
        }

        // $savedQuery の場合は保存済みクエリ呼び出しです
        if matches!(self.peek(), TokenKind::SavedQuery(_)) {
            return self.parse_saved_query_stage(start);
        }

        // [field, ...] の場合は stats shorthand です
        if matches!(self.peek(), TokenKind::LBracket) {
            return self.parse_stats_shorthand(start);
        }

        // 識別子の後に `(` が続く場合は関数呼び出しです
        // `!` の後の識別子 + `(` も関数呼び出し (否定) です
        if self.is_function_call() {
            return self.parse_function_call_stage(start);
        }

        // それ以外はフィルタ式です
        self.parse_filter_stage(start)
    }

    fn is_assignment(&self) -> bool {
        if !matches!(self.peek(), TokenKind::Identifier(_)) {
            return false;
        }
        let extra = self.dotted_identifier_extra(self.pos + 1);
        self.tokens
            .get(self.pos + 1 + extra)
            .map(|t| t.kind == TokenKind::Assign)
            .unwrap_or(false)
    }

    fn is_function_call(&self) -> bool {
        let offset = if matches!(self.peek(), TokenKind::Bang) {
            1
        } else {
            0
        };

        let is_func_name = matches!(
            self.tokens.get(self.pos + offset).map(|t| &t.kind),
            Some(TokenKind::Identifier(_) | TokenKind::In | TokenKind::Match)
        );
        let has_lparen = matches!(
            self.tokens.get(self.pos + offset + 1).map(|t| &t.kind),
            Some(TokenKind::LParen)
        );
        is_func_name && has_lparen
    }

    // ---- 代入 ----

    fn parse_assignment(&mut self, start: usize) -> Option<PipelineStage> {
        let field_tok = self.advance();
        let field = match field_tok.kind {
            TokenKind::Identifier(name) => self.consume_dotted_suffix(name),
            _ => unreachable!(),
        };
        self.advance(); // `:=` を消費します

        match self.parse_expr() {
            Some(value) => {
                let end = self.prev_span().end;
                Some(PipelineStage {
                    kind: StageKind::Assignment(Assignment {
                        field,
                        value,
                        span: Span::new(start, end),
                    }),
                    span: Span::new(start, end),
                })
            }
            None => {
                let span = self.current_span();
                self.errors
                    .push(ParseError::new("expected expression after ':='", span));
                None
            }
        }
    }

    // ---- 関数呼び出し ----

    fn parse_function_call_stage(&mut self, start: usize) -> Option<PipelineStage> {
        let negated = if matches!(self.peek(), TokenKind::Bang) {
            self.advance();
            true
        } else {
            false
        };

        let fc = self.parse_function_call_expr()?;

        let end = fc.span.end;
        let stage_kind = if negated {
            StageKind::Filter(FilterExpr::Not(Box::new(FilterExpr::FunctionCall(fc))))
        } else {
            StageKind::FunctionCall(fc)
        };

        Some(PipelineStage {
            kind: stage_kind,
            span: Span::new(start, end),
        })
    }

    fn parse_function_call_expr(&mut self) -> Option<FunctionCall> {
        let start = self.current_span().start;
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::Identifier(name) => name,
            TokenKind::In => "in".to_string(),
            TokenKind::Match => "match".to_string(),
            _ => {
                self.errors
                    .push(ParseError::new("expected function name", name_tok.span));
                return None;
            }
        };

        if let Err(e) = self.expect(&TokenKind::LParen) {
            self.errors.push(e);
            return None;
        }

        let mut arguments = Vec::new();

        while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
            match self.parse_argument() {
                Some(arg) => arguments.push(arg),
                None => {
                    // エラーリカバリ: `)` か `,` まで読み飛ばします
                    while !matches!(
                        self.peek(),
                        TokenKind::RParen | TokenKind::Comma | TokenKind::Eof
                    ) {
                        self.advance();
                    }
                }
            }
            if matches!(self.peek(), TokenKind::Comma) {
                self.advance();
            }
        }

        if let Err(e) = self.expect(&TokenKind::RParen) {
            self.errors.push(e);
        }

        let end = self.prev_span().end;
        Some(FunctionCall {
            name,
            arguments,
            span: Span::new(start, end),
        })
    }

    fn parse_argument(&mut self) -> Option<Argument> {
        // 名前付き引数の判定: (identifier | "as") = value
        if self.is_named_arg() {
            let name_tok = self.advance();
            let name = match name_tok.kind {
                TokenKind::Identifier(n) => n,
                TokenKind::As => "as".to_string(),
                _ => unreachable!(),
            };
            self.advance(); // `=` を消費します
            let value = self.parse_expr()?;
            return Some(Argument::Named { name, value });
        }

        // 位置引数 (比較式も許可します)
        let expr = self.parse_argument_expr()?;
        Some(Argument::Positional(expr))
    }

    /// 名前付き引数かどうかを先読みで判定します。
    fn is_named_arg(&self) -> bool {
        let is_name = matches!(self.peek(), TokenKind::Identifier(_) | TokenKind::As);
        if !is_name {
            return false;
        }
        self.tokens
            .get(self.pos + 1)
            .map(|t| t.kind == TokenKind::Eq)
            .unwrap_or(false)
    }

    /// 引数内で使える式をパースします。通常の式に加えて比較式も許可します。
    fn parse_argument_expr(&mut self) -> Option<Expr> {
        let expr = self.parse_expr()?;

        // 比較演算子が続く場合は比較式として扱います
        let op = match self.peek() {
            TokenKind::Eq => Some(CompareOp::Eq),
            TokenKind::NotEq => Some(CompareOp::NotEq),
            TokenKind::EqEq => Some(CompareOp::EqEq),
            TokenKind::Lt => Some(CompareOp::Lt),
            TokenKind::LtEq => Some(CompareOp::LtEq),
            TokenKind::Gt => Some(CompareOp::Gt),
            TokenKind::GtEq => Some(CompareOp::GtEq),
            TokenKind::Like => Some(CompareOp::Like),
            TokenKind::Link => Some(CompareOp::Link),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let right = self.parse_expr()?;
            return Some(Expr::Comparison {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            });
        }

        Some(expr)
    }

    // ---- フィルタ式 ----

    fn parse_filter_stage(&mut self, start: usize) -> Option<PipelineStage> {
        let filter = self.parse_filter_or()?;
        let end = self.prev_span().end;
        Some(PipelineStage {
            kind: StageKind::Filter(filter),
            span: Span::new(start, end),
        })
    }

    /// CQL では OR は AND より結合が強いです。
    /// filter_or = filter_and ("and" filter_and)*
    fn parse_filter_or(&mut self) -> Option<FilterExpr> {
        let mut left = self.parse_filter_and()?;

        while matches!(self.peek(), TokenKind::And) {
            self.advance();
            let right = self.parse_filter_and()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// filter_and = filter_not ("or" filter_not)*
    fn parse_filter_and(&mut self) -> Option<FilterExpr> {
        let mut left = self.parse_filter_not()?;

        while matches!(self.peek(), TokenKind::Or) {
            self.advance();
            let right = self.parse_filter_not()?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// filter_not = "not" filter_not | "!" filter_primary | filter_primary
    fn parse_filter_not(&mut self) -> Option<FilterExpr> {
        if matches!(self.peek(), TokenKind::Not) && !self.is_field_filter() {
            self.advance();
            let inner = self.parse_filter_not()?;
            return Some(FilterExpr::Not(Box::new(inner)));
        }

        // `!` の後に関数呼び出し以外が続く場合
        if matches!(self.peek(), TokenKind::Bang) {
            // `!identifier(` のパターンは is_function_call で処理済みなので、
            // ここに来る場合は単純な NOT です
            self.advance();
            let inner = self.parse_filter_primary()?;
            return Some(FilterExpr::Not(Box::new(inner)));
        }

        self.parse_filter_primary()
    }

    /// filter_primary = "(" filter_or ")" | field_filter | free_text
    fn parse_filter_primary(&mut self) -> Option<FilterExpr> {
        // 括弧
        if matches!(self.peek(), TokenKind::LParen) {
            self.advance();
            let inner = self.parse_filter_or()?;
            if let Err(e) = self.expect(&TokenKind::RParen) {
                self.errors.push(e);
            }
            return Some(FilterExpr::Grouped(Box::new(inner)));
        }

        // フィールドフィルタ: identifier op value
        if self.is_field_filter() {
            return self.parse_field_filter();
        }

        // フリーテキストフィルタ
        self.parse_free_text_filter()
    }

    fn is_field_filter(&self) -> bool {
        let is_field = matches!(
            self.peek(),
            TokenKind::Identifier(_)
                | TokenKind::AtField(_)
                | TokenKind::HashField(_)
                | TokenKind::QuotedIdentifier(_)
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Not
                | TokenKind::Like
                | TokenKind::In
                | TokenKind::Match
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Case
                | TokenKind::As
        );
        if !is_field {
            return false;
        }
        let extra = if matches!(self.peek(), TokenKind::Identifier(_)) {
            self.dotted_identifier_extra(self.pos + 1)
        } else {
            0
        };
        matches!(
            self.tokens.get(self.pos + 1 + extra).map(|t| &t.kind),
            Some(
                TokenKind::Eq
                    | TokenKind::NotEq
                    | TokenKind::EqEq
                    | TokenKind::MatchOp
                    | TokenKind::Lt
                    | TokenKind::LtEq
                    | TokenKind::Gt
                    | TokenKind::GtEq
                    | TokenKind::Like
                    | TokenKind::Link
            )
        )
    }

    fn parse_field_filter(&mut self) -> Option<FilterExpr> {
        let field_tok = self.advance();
        let field = match field_tok.kind {
            TokenKind::Identifier(name) => self.consume_dotted_suffix(name),
            TokenKind::AtField(name) | TokenKind::HashField(name) => name,
            TokenKind::QuotedIdentifier(name) => name,
            TokenKind::And => "and".to_string(),
            TokenKind::Or => "or".to_string(),
            TokenKind::Not => "not".to_string(),
            TokenKind::Like => "like".to_string(),
            TokenKind::In => "in".to_string(),
            TokenKind::Match => "match".to_string(),
            TokenKind::True => "true".to_string(),
            TokenKind::False => "false".to_string(),
            TokenKind::Case => "case".to_string(),
            TokenKind::As => "as".to_string(),
            _ => unreachable!(),
        };

        // MatchOp (=~) の場合は右辺が関数呼び出しになります
        if matches!(self.peek(), TokenKind::MatchOp) {
            self.advance(); // =~ を消費します
            let fc = self.parse_function_call_expr()?;
            // 暗黙の AND チェック
            if self.is_implicit_and() {
                let left = FilterExpr::FieldShorthand {
                    field,
                    function: fc,
                };
                let right = self.parse_filter_primary()?;
                return Some(FilterExpr::And(Box::new(left), Box::new(right)));
            }
            return Some(FilterExpr::FieldShorthand {
                field,
                function: fc,
            });
        }

        let op_tok = self.advance();
        let op = match op_tok.kind {
            TokenKind::Eq => CompareOp::Eq,
            TokenKind::NotEq => CompareOp::NotEq,
            TokenKind::EqEq => CompareOp::EqEq,
            TokenKind::Lt => CompareOp::Lt,
            TokenKind::LtEq => CompareOp::LtEq,
            TokenKind::Gt => CompareOp::Gt,
            TokenKind::GtEq => CompareOp::GtEq,
            TokenKind::Like => CompareOp::Like,
            TokenKind::Link => CompareOp::Link,
            _ => unreachable!(),
        };

        let value = self.parse_filter_value()?;

        // 暗黙の AND: フィールドフィルタの後にフィルタが続く場合
        if self.is_implicit_and() {
            let left = FilterExpr::FieldFilter { field, op, value };
            let right = self.parse_filter_primary()?;
            return Some(FilterExpr::And(Box::new(left), Box::new(right)));
        }

        Some(FilterExpr::FieldFilter { field, op, value })
    }

    fn is_implicit_and(&self) -> bool {
        // パイプ、EOF、閉じ括弧/閉じ波括弧、明示的な論理演算子、
        // セミコロン、FatArrow、Error トークンの場合は暗黙の AND ではありません
        !matches!(
            self.peek(),
            TokenKind::Pipe
                | TokenKind::Eof
                | TokenKind::RParen
                | TokenKind::RBrace
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Comma
                | TokenKind::Semicolon
                | TokenKind::FatArrow
        ) && !matches!(self.peek(), TokenKind::Error(_))
            && !self.at_end()
    }

    fn parse_filter_value(&mut self) -> Option<FilterValue> {
        match self.peek().clone() {
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Some(FilterValue::String(s))
            }
            TokenKind::Integer(n) => {
                self.advance();
                Some(FilterValue::Number(n as f64))
            }
            TokenKind::Float(n) => {
                self.advance();
                Some(FilterValue::Number(n))
            }
            TokenKind::Regex { pattern, flags } => {
                let pattern = pattern.clone();
                let flags = flags.clone();
                self.advance();
                Some(FilterValue::Regex { pattern, flags })
            }
            TokenKind::Wildcard(w) => {
                let w = w.clone();
                self.advance();
                Some(FilterValue::Wildcard(w))
            }
            TokenKind::Star => {
                self.advance();
                Some(FilterValue::Wildcard("*".to_string()))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                let name = self.consume_dotted_suffix(name);
                Some(FilterValue::Field(name))
            }
            TokenKind::Parameter(name) => {
                let name = name.clone();
                self.advance();
                Some(FilterValue::Parameter(name))
            }
            TokenKind::True => {
                self.advance();
                Some(FilterValue::Field("true".to_string()))
            }
            TokenKind::False => {
                self.advance();
                Some(FilterValue::Field("false".to_string()))
            }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError::new(
                    format!("expected filter value, found {:?}", self.peek()),
                    span,
                ));
                None
            }
        }
    }

    fn parse_free_text_filter(&mut self) -> Option<FilterExpr> {
        match self.peek().clone() {
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                let filter = FilterExpr::FreeText(s);
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            TokenKind::Regex { pattern, flags } => {
                let pattern = pattern.clone();
                let flags = flags.clone();
                self.advance();
                let filter = FilterExpr::FreeText(format!("/{}/{}", pattern, flags));
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            TokenKind::Identifier(s) => {
                let s = s.clone();
                self.advance();
                let s = self.consume_dotted_suffix(s);
                let filter = FilterExpr::FreeText(s);
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            TokenKind::Wildcard(w) => {
                let w = w.clone();
                self.advance();
                let filter = FilterExpr::FreeText(w);
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            TokenKind::Star => {
                self.advance();
                let filter = FilterExpr::FreeText("*".to_string());
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            TokenKind::Parameter(p) => {
                let p = p.clone();
                self.advance();
                let filter = FilterExpr::FreeText(format!("?{}", p));
                if self.is_implicit_and() {
                    let right = self.parse_filter_primary()?;
                    return Some(FilterExpr::And(Box::new(filter), Box::new(right)));
                }
                Some(filter)
            }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError::new(
                    format!("expected filter expression, found {:?}", self.peek()),
                    span,
                ));
                None
            }
        }
    }

    // ---- 式 (Expression) ----

    /// expr = additive
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_additive()
    }

    /// additive = multiplicative (("+"|"-") multiplicative)*
    fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// multiplicative = unary (("*"|"/") unary)*
    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// unary = "-" unary | "!" unary | primary
    fn parse_unary(&mut self) -> Option<Expr> {
        if matches!(self.peek(), TokenKind::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            return Some(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        if matches!(self.peek(), TokenKind::Bang) {
            self.advance();
            let operand = self.parse_unary()?;
            return Some(Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    /// primary = number | string | bool | regex | wildcard | array | function_call | field | "(" expr ")"
    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek().clone() {
            TokenKind::Integer(n) => {
                self.advance();
                Some(Expr::Number(n as f64))
            }
            TokenKind::Float(n) => {
                self.advance();
                Some(Expr::Number(n))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Some(Expr::String(s))
            }
            TokenKind::True => {
                self.advance();
                Some(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Some(Expr::Bool(false))
            }
            TokenKind::Regex { pattern, flags } => {
                let pattern = pattern.clone();
                let flags = flags.clone();
                self.advance();
                Some(Expr::Regex { pattern, flags })
            }
            TokenKind::Wildcard(w) => {
                let w = w.clone();
                self.advance();
                Some(Expr::Wildcard(w))
            }
            TokenKind::Star => {
                self.advance();
                Some(Expr::Wildcard("*".to_string()))
            }
            TokenKind::LBracket => self.parse_array(),
            TokenKind::Identifier(_) => {
                // 関数呼び出しか、フィールド参照かを判定します
                if matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::LParen)
                ) {
                    let fc = self.parse_function_call_expr()?;
                    Some(Expr::FunctionCall(fc))
                } else {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Identifier(name) => {
                            let name = self.consume_dotted_suffix(name);
                            let mut expr = Expr::Field(name);
                            // 配列インデックスが続く場合
                            while matches!(self.peek(), TokenKind::LBracket) {
                                self.advance(); // `[` を消費します
                                let index = self.parse_expr()?;
                                if let Err(e) = self.expect(&TokenKind::RBracket) {
                                    self.errors.push(e);
                                }
                                expr = Expr::IndexAccess {
                                    object: Box::new(expr),
                                    index: Box::new(index),
                                };
                            }
                            Some(expr)
                        }
                        _ => unreachable!(),
                    }
                }
            }
            TokenKind::AtField(name) => {
                let name = name.clone();
                self.advance();
                Some(Expr::Field(name))
            }
            TokenKind::HashField(name) => {
                let name = name.clone();
                self.advance();
                Some(Expr::Field(name))
            }
            TokenKind::Parameter(name) => {
                let name = name.clone();
                self.advance();
                Some(Expr::Parameter(name))
            }
            TokenKind::SavedQuery(name) => {
                let name = name.clone();
                self.advance();
                if matches!(self.peek(), TokenKind::LParen) {
                    self.advance(); // `(` を消費します
                    let mut arguments = Vec::new();
                    while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                        match self.parse_argument() {
                            Some(arg) => arguments.push(arg),
                            None => {
                                while !matches!(
                                    self.peek(),
                                    TokenKind::RParen | TokenKind::Comma | TokenKind::Eof
                                ) {
                                    self.advance();
                                }
                            }
                        }
                        if matches!(self.peek(), TokenKind::Comma) {
                            self.advance();
                        }
                    }
                    if let Err(e) = self.expect(&TokenKind::RParen) {
                        self.errors.push(e);
                    }
                    Some(Expr::SavedQueryCall { name, arguments })
                } else {
                    Some(Expr::Field(format!("${}", name)))
                }
            }
            TokenKind::QuotedIdentifier(name) => {
                let name = name.clone();
                self.advance();
                Some(Expr::Field(name))
            }
            TokenKind::LBrace => self.parse_sub_query(),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                if let Err(e) = self.expect(&TokenKind::RParen) {
                    self.errors.push(e);
                }
                Some(expr)
            }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError::new(
                    format!("expected expression, found {:?}", self.peek()),
                    span,
                ));
                None
            }
        }
    }

    /// array = "[" (expr ("," expr)*)? "]"
    fn parse_array(&mut self) -> Option<Expr> {
        self.advance(); // `[` を消費します
        let mut elements = Vec::new();

        while !matches!(self.peek(), TokenKind::RBracket | TokenKind::Eof) {
            match self.parse_expr() {
                Some(expr) => elements.push(expr),
                None => break,
            }
            if matches!(self.peek(), TokenKind::Comma) {
                self.advance();
            }
        }

        if let Err(e) = self.expect(&TokenKind::RBracket) {
            self.errors.push(e);
        }

        Some(Expr::Array(elements))
    }

    /// sub_query = "{" pipeline "}"
    fn parse_sub_query(&mut self) -> Option<Expr> {
        self.advance(); // `{` を消費します
        let start = self.current_span().start;

        let mut stages = Vec::new();

        // サブクエリ内のパイプラインをパースします
        if !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            if let Some(stage) = self.parse_stage() {
                stages.push(stage);
            }
        }

        while matches!(self.peek(), TokenKind::Pipe) {
            self.advance();
            if let Some(stage) = self.parse_stage() {
                stages.push(stage);
            }
        }

        if let Err(e) = self.expect(&TokenKind::RBrace) {
            self.errors.push(e);
        }

        let end = self.prev_span().end;
        Some(Expr::SubQuery(Box::new(Query {
            stages,
            span: Span::new(start, end),
        })))
    }

    // ---- case 文 ----

    fn parse_case_statement(&mut self, start: usize) -> Option<PipelineStage> {
        self.advance(); // `case` を消費します
        self.advance(); // `{` を消費します
        let mut branches = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let branch_start = self.current_span().start;
            let pipeline = self.parse_branch_pipeline();
            let branch_end = self.prev_span().end;
            if !pipeline.is_empty() {
                branches.push(CaseBranch {
                    pipeline,
                    span: Span::new(branch_start, branch_end),
                });
            }
            if matches!(self.peek(), TokenKind::Semicolon) {
                self.advance();
            }
        }
        if let Err(e) = self.expect(&TokenKind::RBrace) {
            self.errors.push(e);
        }
        let end = self.prev_span().end;
        Some(PipelineStage {
            kind: StageKind::CaseStatement(CaseStatement {
                branches,
                span: Span::new(start, end),
            }),
            span: Span::new(start, end),
        })
    }

    // ---- match 文 ----

    fn is_match_statement(&self) -> bool {
        matches!(self.peek(), TokenKind::Identifier(_))
            && matches!(
                self.tokens.get(self.pos + 1).map(|t| &t.kind),
                Some(TokenKind::Match)
            )
            && matches!(
                self.tokens.get(self.pos + 2).map(|t| &t.kind),
                Some(TokenKind::LBrace)
            )
    }

    fn parse_match_statement(&mut self, start: usize) -> Option<PipelineStage> {
        let field_tok = self.advance();
        let field = match field_tok.kind {
            TokenKind::Identifier(name) => self.consume_dotted_suffix(name),
            _ => unreachable!(),
        };
        self.advance(); // `match` キーワードを消費します
        self.advance(); // `{` を消費します
        let mut branches = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let branch_start = self.current_span().start;
            let pattern = match self.parse_expr() {
                Some(p) => p,
                None => {
                    self.skip_to_semicolon_or_rbrace();
                    if matches!(self.peek(), TokenKind::Semicolon) {
                        self.advance();
                    }
                    continue;
                }
            };
            if let Err(e) = self.expect(&TokenKind::FatArrow) {
                self.errors.push(e);
                self.skip_to_semicolon_or_rbrace();
                if matches!(self.peek(), TokenKind::Semicolon) {
                    self.advance();
                }
                continue;
            }
            let pipeline = self.parse_branch_pipeline();
            let branch_end = self.prev_span().end;
            branches.push(MatchBranch {
                pattern,
                pipeline,
                span: Span::new(branch_start, branch_end),
            });
            if matches!(self.peek(), TokenKind::Semicolon) {
                self.advance();
            }
        }
        if let Err(e) = self.expect(&TokenKind::RBrace) {
            self.errors.push(e);
        }
        let end = self.prev_span().end;
        Some(PipelineStage {
            kind: StageKind::MatchStatement(MatchStatement {
                field,
                branches,
                span: Span::new(start, end),
            }),
            span: Span::new(start, end),
        })
    }

    // ---- 保存済みクエリ ----

    fn parse_saved_query_stage(&mut self, start: usize) -> Option<PipelineStage> {
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::SavedQuery(name) => name,
            _ => unreachable!(),
        };
        if matches!(self.peek(), TokenKind::LParen) {
            self.advance();
            let mut arguments = Vec::new();
            while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                match self.parse_argument() {
                    Some(arg) => arguments.push(arg),
                    None => {
                        while !matches!(
                            self.peek(),
                            TokenKind::RParen | TokenKind::Comma | TokenKind::Eof
                        ) {
                            self.advance();
                        }
                    }
                }
                if matches!(self.peek(), TokenKind::Comma) {
                    self.advance();
                }
            }
            if let Err(e) = self.expect(&TokenKind::RParen) {
                self.errors.push(e);
            }
            let end = self.prev_span().end;
            Some(PipelineStage {
                kind: StageKind::FunctionCall(FunctionCall {
                    name: format!("${}", name),
                    arguments,
                    span: Span::new(start, end),
                }),
                span: Span::new(start, end),
            })
        } else {
            let end = self.prev_span().end;
            Some(PipelineStage {
                kind: StageKind::Filter(FilterExpr::FreeText(format!("${}", name))),
                span: Span::new(start, end),
            })
        }
    }

    // ---- Stats Shorthand ----

    fn parse_stats_shorthand(&mut self, start: usize) -> Option<PipelineStage> {
        let array_expr = self.parse_array()?;
        let end = self.prev_span().end;
        Some(PipelineStage {
            kind: StageKind::FunctionCall(FunctionCall {
                name: "stats".to_string(),
                arguments: vec![Argument::Positional(array_expr)],
                span: Span::new(start, end),
            }),
            span: Span::new(start, end),
        })
    }

    // ---- 分岐パイプライン ----

    fn parse_branch_pipeline(&mut self) -> Vec<PipelineStage> {
        let mut stages = Vec::new();
        if !matches!(
            self.peek(),
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) {
            if let Some(stage) = self.parse_stage() {
                stages.push(stage);
            }
        }
        while matches!(self.peek(), TokenKind::Pipe) {
            self.advance();
            if let Some(stage) = self.parse_stage() {
                stages.push(stage);
            }
        }
        stages
    }

    fn skip_to_semicolon_or_rbrace(&mut self) {
        while !matches!(
            self.peek(),
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) {
            self.advance();
        }
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }
}

// AST に FunctionCall バリアントを追加する必要があります
impl FilterExpr {
    /// ステージレベルで否定された関数呼び出しを表現するために使います。
    fn _is_function_call(&self) -> bool {
        matches!(self, FilterExpr::FunctionCall(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> (Option<Query>, Vec<ParseError>) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let parser = Parser::new(tokens);
        parser.parse()
    }

    fn parse_ok(input: &str) -> Query {
        let (query, errors) = parse(input);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        query.expect("expected query")
    }

    fn parse_err(input: &str) -> Vec<ParseError> {
        let (_, errors) = parse(input);
        assert!(!errors.is_empty(), "expected errors but got none");
        errors
    }

    // ---- 空のクエリ ----

    #[test]
    fn test_empty_query() {
        let query = parse_ok("");
        assert!(query.stages.is_empty());
    }

    // ---- フリーテキストフィルタ ----

    #[test]
    fn test_free_text() {
        let query = parse_ok("error");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FreeText(s)) => assert_eq!(s, "error"),
            other => panic!("expected FreeText, got {:?}", other),
        }
    }

    #[test]
    fn test_free_text_string() {
        let query = parse_ok(r#""error message""#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FreeText(s)) => assert_eq!(s, "error message"),
            other => panic!("expected FreeText, got {:?}", other),
        }
    }

    #[test]
    fn test_free_text_regex() {
        let query = parse_ok("/error/i");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FreeText(s)) => assert_eq!(s, "/error/i"),
            other => panic!("expected FreeText, got {:?}", other),
        }
    }

    // ---- フィールドフィルタ ----

    #[test]
    fn test_field_eq_string() {
        let query = parse_ok(r#"user = "Alan Turing""#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "user");
                assert_eq!(*op, CompareOp::Eq);
                assert_eq!(*value, FilterValue::String("Alan Turing".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_field_neq() {
        let query = parse_ok(r#"user != "root""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { op, .. }) => {
                assert_eq!(*op, CompareOp::NotEq);
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_field_numeric_comparison() {
        let query = parse_ok("statuscode >= 400");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "statuscode");
                assert_eq!(*op, CompareOp::GtEq);
                assert_eq!(*value, FilterValue::Number(400.0));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_field_regex_filter() {
        let query = parse_ok("url = /login/i");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { value, .. }) => {
                assert_eq!(
                    *value,
                    FilterValue::Regex {
                        pattern: "login".to_string(),
                        flags: "i".to_string()
                    }
                );
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_field_wildcard() {
        let query = parse_ok("url = *login*");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { value, .. }) => {
                assert_eq!(*value, FilterValue::Wildcard("*login*".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_field_existence() {
        let query = parse_ok("user = *");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { value, .. }) => {
                assert_eq!(*value, FilterValue::Wildcard("*".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_like_operator() {
        let query = parse_ok(r#"class like "Bucket""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { op, .. }) => {
                assert_eq!(*op, CompareOp::Like);
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_link_operator_in_field_filter() {
        let query = parse_ok(r#"field1 <=> field2"#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "field1");
                assert_eq!(*op, CompareOp::Link);
                assert_eq!(*value, FilterValue::Field("field2".to_string()));
            }
            other => panic!("expected FieldFilter with Link, got {:?}", other),
        }
    }

    #[test]
    fn test_link_operator_in_function_argument() {
        let query = parse_ok(r#"correlate(field1 <=> field2)"#);
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "correlate");
                assert_eq!(fc.arguments.len(), 1);
                match &fc.arguments[0] {
                    Argument::Positional(Expr::Comparison { left, op, right }) => {
                        assert!(matches!(left.as_ref(), Expr::Field(f) if f == "field1"));
                        assert_eq!(*op, CompareOp::Link);
                        assert!(matches!(right.as_ref(), Expr::Field(f) if f == "field2"));
                    }
                    other => panic!("expected Comparison with Link, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- 予約語をフィールド名として使用 ----

    #[test]
    fn test_keyword_and_as_field_name() {
        let query = parse_ok(r#"and = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, .. }) => {
                assert_eq!(field, "and");
                assert_eq!(*op, CompareOp::Eq);
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_or_as_field_name() {
        let query = parse_ok(r#"or = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "or");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_not_as_field_name() {
        let query = parse_ok(r#"not = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "not");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_case_as_field_name() {
        let query = parse_ok(r#"case = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "case");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_match_as_field_name() {
        let query = parse_ok(r#"match = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "match");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_in_as_field_name() {
        let query = parse_ok(r#"in = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "in");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_true_as_field_name() {
        let query = parse_ok(r#"true = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "true");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_false_as_field_name() {
        let query = parse_ok(r#"false = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "false");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_as_as_field_name() {
        let query = parse_ok(r#"as = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "as");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_keyword_like_as_field_name() {
        let query = parse_ok(r#"like = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "like");
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_true_as_filter_value() {
        let query = parse_ok("Success=true");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "Success");
                assert_eq!(*op, CompareOp::Eq);
                assert_eq!(*value, FilterValue::Field("true".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_false_as_filter_value() {
        let query = parse_ok("Enabled=false");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "Enabled");
                assert_eq!(*op, CompareOp::Eq);
                assert_eq!(*value, FilterValue::Field("false".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    // ---- 論理演算子 ----

    #[test]
    fn test_explicit_and() {
        let query = parse_ok(r#"status = 200 and method = "GET""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::And(left, right)) => {
                assert!(matches!(left.as_ref(), FilterExpr::FieldFilter { .. }));
                assert!(matches!(right.as_ref(), FilterExpr::FieldFilter { .. }));
            }
            other => panic!("expected And, got {:?}", other),
        }
    }

    #[test]
    fn test_implicit_and() {
        let query = parse_ok(r#"src="client" ip="127.0.0.1""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::And(left, right)) => {
                assert!(matches!(left.as_ref(), FilterExpr::FieldFilter { .. }));
                assert!(matches!(right.as_ref(), FilterExpr::FieldFilter { .. }));
            }
            other => panic!("expected And (implicit), got {:?}", other),
        }
    }

    #[test]
    fn test_or() {
        let query = parse_ok("method = GET or method = POST");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::Or(..)) => {}
            other => panic!("expected Or, got {:?}", other),
        }
    }

    #[test]
    fn test_not() {
        let query = parse_ok(r#"not status = 200"#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::Not(inner)) => {
                assert!(matches!(inner.as_ref(), FilterExpr::FieldFilter { .. }));
            }
            other => panic!("expected Not, got {:?}", other),
        }
    }

    #[test]
    fn test_grouped_expression() {
        let query = parse_ok("statuscode=404 and (method=GET or method=POST)");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::And(left, right)) => {
                assert!(matches!(left.as_ref(), FilterExpr::FieldFilter { .. }));
                assert!(matches!(right.as_ref(), FilterExpr::Grouped(_)));
            }
            other => panic!("expected And with Grouped, got {:?}", other),
        }
    }

    /// CQL では OR は AND より結合が強いことを確認します。
    /// `a and b or c` は `a and (b or c)` と解釈されます。
    #[test]
    fn test_or_binds_tighter_than_and() {
        let query = parse_ok("x=1 and y=2 or z=3");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::And(left, right)) => {
                assert!(matches!(left.as_ref(), FilterExpr::FieldFilter { .. }));
                assert!(matches!(right.as_ref(), FilterExpr::Or(..)));
            }
            other => panic!("expected And(field, Or(..)), got {:?}", other),
        }
    }

    // ---- パイプライン ----

    #[test]
    fn test_pipeline() {
        let query = parse_ok("status = 404 | count()");
        assert_eq!(query.stages.len(), 2);
        assert!(matches!(
            &query.stages[0].kind,
            StageKind::Filter(FilterExpr::FieldFilter { .. })
        ));
        assert!(matches!(&query.stages[1].kind, StageKind::FunctionCall(_)));
    }

    #[test]
    fn test_multi_stage_pipeline() {
        let query = parse_ok(r#"status >= 400 | groupBy(field=url) | sort(count, order=desc)"#);
        assert_eq!(query.stages.len(), 3);
    }

    // ---- 関数呼び出し ----

    #[test]
    fn test_function_no_args() {
        let query = parse_ok("count()");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "count");
                assert!(fc.arguments.is_empty());
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_function_positional_arg() {
        let query = parse_ok("count(status)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "count");
                assert_eq!(fc.arguments.len(), 1);
                match &fc.arguments[0] {
                    Argument::Positional(Expr::Field(name)) => assert_eq!(name, "status"),
                    other => panic!("expected Positional field, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_function_named_arg() {
        let query = parse_ok("groupBy(field=src)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "groupBy");
                assert_eq!(fc.arguments.len(), 1);
                match &fc.arguments[0] {
                    Argument::Named { name, value } => {
                        assert_eq!(name, "field");
                        assert_eq!(*value, Expr::Field("src".to_string()));
                    }
                    other => panic!("expected Named arg, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_function_mixed_args() {
        let query = parse_ok(r#"top(field, limit=10)"#);
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.arguments.len(), 2);
                assert!(matches!(&fc.arguments[0], Argument::Positional(_)));
                assert!(matches!(&fc.arguments[1], Argument::Named { .. }));
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_function_array_arg() {
        let query = parse_ok("in(field, values=[a, b, c])");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "in");
                assert_eq!(fc.arguments.len(), 2);
                match &fc.arguments[1] {
                    Argument::Named { name, value } => {
                        assert_eq!(name, "values");
                        match value {
                            Expr::Array(elems) => assert_eq!(elems.len(), 3),
                            other => panic!("expected Array, got {:?}", other),
                        }
                    }
                    other => panic!("expected Named arg, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_negated_function() {
        let query = parse_ok("!cidr(ip)");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::Not(inner)) => {
                assert!(matches!(inner.as_ref(), FilterExpr::FunctionCall(_)));
            }
            other => panic!("expected Not(FunctionCall), got {:?}", other),
        }
    }

    // ---- 代入 ----

    #[test]
    fn test_assignment() {
        let query = parse_ok("newField := value + 1");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert_eq!(a.field, "newField");
                match &a.value {
                    Expr::BinaryOp { op, .. } => assert_eq!(*op, BinaryOp::Add),
                    other => panic!("expected BinaryOp, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_assignment_string() {
        let query = parse_ok(r#"label := "critical""#);
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert_eq!(a.field, "label");
                assert_eq!(a.value, Expr::String("critical".to_string()));
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ---- 式 ----

    #[test]
    fn test_arithmetic_expr() {
        let query = parse_ok("result := a * 2 + b");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => match &a.value {
                Expr::BinaryOp {
                    left,
                    op: BinaryOp::Add,
                    right,
                } => {
                    assert!(matches!(
                        left.as_ref(),
                        Expr::BinaryOp {
                            op: BinaryOp::Mul,
                            ..
                        }
                    ));
                    assert!(matches!(right.as_ref(), Expr::Field(_)));
                }
                other => panic!("expected Add(Mul(..), field), got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_unary_neg() {
        let query = parse_ok("x := -1");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert!(matches!(
                    &a.value,
                    Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        ..
                    }
                ));
            }
            other => panic!("expected Assignment with Neg, got {:?}", other),
        }
    }

    // ---- 複合クエリ ----

    #[test]
    fn test_complex_pipeline() {
        let query = parse_ok(
            r#"src="client" ip="127.0.0.1" | groupBy(field=src) | sort(count, order=desc)"#,
        );
        assert_eq!(query.stages.len(), 3);
        assert!(matches!(
            &query.stages[0].kind,
            StageKind::Filter(FilterExpr::And(..))
        ));
        assert!(matches!(&query.stages[1].kind, StageKind::FunctionCall(_)));
        assert!(matches!(&query.stages[2].kind, StageKind::FunctionCall(_)));
    }

    #[test]
    fn test_filter_then_assignment() {
        let query = parse_ok("status = 200 | label := ok");
        assert_eq!(query.stages.len(), 2);
        assert!(matches!(
            &query.stages[0].kind,
            StageKind::Filter(FilterExpr::FieldFilter { .. })
        ));
        assert!(matches!(&query.stages[1].kind, StageKind::Assignment(_)));
    }

    // ---- エラーケース ----

    #[test]
    fn test_missing_rparen() {
        let errors = parse_err("count(");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_empty_pipe_stage() {
        let errors = parse_err("foo |");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_assignment_missing_value() {
        let errors = parse_err("field := |");
        assert!(!errors.is_empty());
    }

    // ---- @field ----

    #[test]
    fn test_at_field_filter() {
        let query = parse_ok(r#"@timestamp > "2024-01-01""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, .. }) => {
                assert_eq!(field, "@timestamp");
                assert_eq!(*op, CompareOp::Gt);
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    // ---- 関数呼び出しを式の中で使う ----

    #[test]
    fn test_function_call_in_expr() {
        let query = parse_ok("result := length(name)");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert!(matches!(&a.value, Expr::FunctionCall(_)));
            }
            other => panic!("expected Assignment with FunctionCall, got {:?}", other),
        }
    }

    // ---- タグフィールド (#field) ----

    #[test]
    fn test_hash_field_in_function() {
        let query = parse_ok("in(field=#event_simpleName, values=[A, B])");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "in");
                match &fc.arguments[0] {
                    Argument::Named { name, value } => {
                        assert_eq!(name, "field");
                        assert_eq!(*value, Expr::Field("#event_simpleName".to_string()));
                    }
                    other => panic!("expected Named arg with HashField, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- as= 名前付き引数 ----

    #[test]
    fn test_as_named_arg() {
        let query = parse_ok("rename(field=ComputerName, as=RemoteComputerName)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "rename");
                assert_eq!(fc.arguments.len(), 2);
                match &fc.arguments[1] {
                    Argument::Named { name, value } => {
                        assert_eq!(name, "as");
                        assert_eq!(*value, Expr::Field("RemoteComputerName".to_string()));
                    }
                    other => panic!("expected Named arg with as=, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- サブクエリ ({...}) ----

    #[test]
    fn test_sub_query() {
        let query = parse_ok("join({ count() | sort(field) }, field=key)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "join");
                assert!(fc.arguments.len() >= 2);
                match &fc.arguments[0] {
                    Argument::Positional(Expr::SubQuery(sub)) => {
                        assert_eq!(sub.stages.len(), 2);
                    }
                    other => panic!("expected SubQuery, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- 引数内の比較式 ----

    #[test]
    fn test_compare_expr_in_arg() {
        let query = parse_ok("test(RemoteAddressIP4 != LocalAddressIP4)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "test");
                assert_eq!(fc.arguments.len(), 1);
                match &fc.arguments[0] {
                    Argument::Positional(Expr::Comparison { op, .. }) => {
                        assert_eq!(*op, CompareOp::NotEq);
                    }
                    other => panic!("expected Comparison, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- ドット区切り識別子 ----

    #[test]
    fn test_dotted_field_in_expr() {
        let query = parse_ok("table(fields=[aip.city, aip.country])");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "table");
                match &fc.arguments[0] {
                    Argument::Named { name, value } => {
                        assert_eq!(name, "fields");
                        match value {
                            Expr::Array(elems) => {
                                assert_eq!(elems.len(), 2);
                                assert_eq!(elems[0], Expr::Field("aip.city".to_string()));
                                assert_eq!(elems[1], Expr::Field("aip.country".to_string()));
                            }
                            other => panic!("expected Array, got {:?}", other),
                        }
                    }
                    other => panic!("expected Named arg, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_dotted_field_filter() {
        let query = parse_ok(r#"aip.city = "Tokyo""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, value }) => {
                assert_eq!(field, "aip.city");
                assert_eq!(*op, CompareOp::Eq);
                assert_eq!(*value, FilterValue::String("Tokyo".to_string()));
            }
            other => panic!("expected FieldFilter, got {:?}", other),
        }
    }

    #[test]
    fn test_dotted_field_assignment() {
        let query = parse_ok(r#"geo.city := "Tokyo""#);
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert_eq!(a.field, "geo.city");
                assert_eq!(a.value, Expr::String("Tokyo".to_string()));
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_multi_dotted_field() {
        let query = parse_ok("table(fields=[a.b.c])");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => match &fc.arguments[0] {
                Argument::Named { value, .. } => match value {
                    Expr::Array(elems) => {
                        assert_eq!(elems[0], Expr::Field("a.b.c".to_string()));
                    }
                    other => panic!("expected Array, got {:?}", other),
                },
                other => panic!("expected Named arg, got {:?}", other),
            },
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }

    // ---- 実践クエリ (endpoint monitor) ----

    #[test]
    fn test_endpoint_monitor_query() {
        let input = r#"in(field=#event_simpleName, values=[NetworkReceiveAcceptIP4, NetworkReceiveAcceptIP6])
| event_platform != "Lin"
| ConnectionDirection=1 Protocol = 6
| cidr(RemoteAddressIP4, subnet=["172.16.0.0/12", "192.168.0.0/16"])
| test(RemoteAddressIP4 != LocalAddressIP4)
| !in(field=ContextBaseFileName, values=["ControlCenter", "rapportd"])
| format(format="%s_%s", field=[RemoteAddressIP4, LPort], as=joinKey)
| join({
    in(field=#event_simpleName, values=[NetworkConnectIP4])
    | cidr(LocalAddressIP4, subnet=["172.16.0.0/12"])
    | rename(field=ComputerName, as=RemoteComputerName)
    | groupBy(field=[joinKey, RemoteComputerName], limit=50000)
  }, field=joinKey, key=joinKey, include=[RemoteComputerName], mode=left)
| table(field=[@timestamp, ComputerName, RemoteComputerName])"#;
        let query = parse_ok(input);
        // in | filter | filter | cidr | test | !in | format | join | table = 9 stages
        assert_eq!(query.stages.len(), 9);
    }

    #[test]
    fn test_error_recovery_multibyte_continues_pipeline() {
        let (query, errors) = parse("これはテスト | count()");
        assert!(!errors.is_empty());
        let query = query.expect("should produce a partial AST");
        assert!(
            query
                .stages
                .iter()
                .any(|s| { matches!(&s.kind, StageKind::FunctionCall(fc) if fc.name == "count") })
        );
    }

    #[test]
    fn test_error_recovery_multiple_errors() {
        let (query, errors) = parse("あ | count() | い");
        assert!(errors.len() >= 2);
        let query = query.expect("should produce a partial AST");
        assert!(
            query
                .stages
                .iter()
                .any(|s| { matches!(&s.kind, StageKind::FunctionCall(fc) if fc.name == "count") })
        );
    }

    #[test]
    fn test_error_recovery_midstream_error() {
        // フィルタの後に非 ASCII テキストが続き、さらにパイプで別ステージが続く場合
        let (query, errors) = parse("status=200\nこれはエラー\n| count()");
        assert!(!errors.is_empty());
        let query = query.expect("should produce a partial AST");
        assert!(
            query
                .stages
                .iter()
                .any(|s| { matches!(&s.kind, StageKind::FunctionCall(fc) if fc.name == "count") })
        );
    }

    // ---- =~ 演算子 ----

    #[test]
    fn test_field_shorthand() {
        let query = parse_ok(r#"ip =~ cidr(subnet="10.0.0.0/8")"#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldShorthand { field, function }) => {
                assert_eq!(field, "ip");
                assert_eq!(function.name, "cidr");
            }
            other => panic!("expected FieldShorthand, got {:?}", other),
        }
    }

    // ---- == 演算子 ----

    #[test]
    fn test_eq_eq_filter() {
        let query = parse_ok("status == 200");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, op, .. }) => {
                assert_eq!(field, "status");
                assert_eq!(*op, CompareOp::EqEq);
            }
            other => panic!("expected FieldFilter with EqEq, got {:?}", other),
        }
    }

    // ---- % モジュロ演算子 ----

    #[test]
    fn test_modulo_operator() {
        let query = parse_ok("result := x % 10");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert_eq!(a.field, "result");
                match &a.value {
                    Expr::BinaryOp { op, .. } => assert_eq!(*op, BinaryOp::Mod),
                    other => panic!("expected BinaryOp with Mod, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ---- case 文 ----

    #[test]
    fn test_case_statement() {
        let query =
            parse_ok(r#"case { status < 300 | label := "ok" ; status >= 500 | label := "error" }"#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::CaseStatement(cs) => {
                assert_eq!(cs.branches.len(), 2);
                assert_eq!(cs.branches[0].pipeline.len(), 2);
                assert_eq!(cs.branches[1].pipeline.len(), 2);
            }
            other => panic!("expected CaseStatement, got {:?}", other),
        }
    }

    #[test]
    fn test_case_single_branch() {
        let query = parse_ok(r#"case { status = 200 | count() }"#);
        match &query.stages[0].kind {
            StageKind::CaseStatement(cs) => {
                assert_eq!(cs.branches.len(), 1);
            }
            other => panic!("expected CaseStatement, got {:?}", other),
        }
    }

    // ---- match 文 ----

    #[test]
    fn test_match_statement() {
        let query = parse_ok(r#"status match { 200 => label := "ok" ; * => label := "other" }"#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::MatchStatement(ms) => {
                assert_eq!(ms.field, "status");
                assert_eq!(ms.branches.len(), 2);
            }
            other => panic!("expected MatchStatement, got {:?}", other),
        }
    }

    // ---- ユーザーパラメータ ----

    #[test]
    fn test_parameter_in_filter_value() {
        let query = parse_ok("status = ?myStatus");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { value, .. }) => {
                assert!(matches!(value, FilterValue::Parameter(_)));
            }
            other => panic!("expected FieldFilter with Parameter value, got {:?}", other),
        }
    }

    #[test]
    fn test_parameter_in_expr() {
        let query = parse_ok("result := ?defaultValue");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert!(matches!(&a.value, Expr::Parameter(_)));
            }
            other => panic!("expected Assignment with Parameter, got {:?}", other),
        }
    }

    #[test]
    fn test_parameter_as_free_text() {
        let query = parse_ok("?searchTerm");
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FreeText(s)) => {
                assert_eq!(s, "?searchTerm");
            }
            other => panic!("expected FreeText, got {:?}", other),
        }
    }

    // ---- 保存済みクエリ ----

    #[test]
    fn test_saved_query_call() {
        let query = parse_ok(r#"$mySearch(param="value")"#);
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "$mySearch");
                assert_eq!(fc.arguments.len(), 1);
            }
            other => panic!("expected FunctionCall for saved query, got {:?}", other),
        }
    }

    // ---- 配列インデックス ----

    #[test]
    fn test_array_index_access() {
        let query = parse_ok("result := items[0]");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => {
                assert!(matches!(&a.value, Expr::IndexAccess { .. }));
            }
            other => panic!("expected Assignment with IndexAccess, got {:?}", other),
        }
    }

    #[test]
    fn test_nested_array_index() {
        let query = parse_ok("result := data[0][1]");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => match &a.value {
                Expr::IndexAccess { object, .. } => {
                    assert!(matches!(object.as_ref(), Expr::IndexAccess { .. }));
                }
                other => panic!("expected nested IndexAccess, got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ---- Stats Shorthand ----

    #[test]
    fn test_stats_shorthand() {
        let query = parse_ok("[count(), avg(duration)]");
        assert_eq!(query.stages.len(), 1);
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "stats");
            }
            other => panic!("expected FunctionCall stats, got {:?}", other),
        }
    }

    // ---- ! 論理否定 (式内) ----

    #[test]
    fn test_unary_not_in_expr() {
        let query = parse_ok("result := !flag");
        match &query.stages[0].kind {
            StageKind::Assignment(a) => match &a.value {
                Expr::UnaryOp { op, .. } => assert_eq!(*op, UnaryOp::Not),
                other => panic!("expected UnaryOp Not, got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ---- バッククォート付きフィールド ----

    #[test]
    fn test_backtick_field_filter() {
        let query = parse_ok(r#"`field name` = "value""#);
        match &query.stages[0].kind {
            StageKind::Filter(FilterExpr::FieldFilter { field, .. }) => {
                assert_eq!(field, "field name");
            }
            other => panic!("expected FieldFilter with backtick field, got {:?}", other),
        }
    }

    // ---- == in argument ----

    #[test]
    fn test_eq_eq_in_argument() {
        let query = parse_ok("test(status == 200)");
        match &query.stages[0].kind {
            StageKind::FunctionCall(fc) => {
                assert_eq!(fc.name, "test");
                match &fc.arguments[0] {
                    Argument::Positional(Expr::Comparison { op, .. }) => {
                        assert_eq!(*op, CompareOp::EqEq);
                    }
                    other => panic!("expected Comparison with EqEq, got {:?}", other),
                }
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        }
    }
}
