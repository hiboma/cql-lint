pub mod token;

use token::{Span, Token, TokenKind};

/// CQL の字句解析器です。
pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    /// 全てのトークンを返します。末尾に Eof トークンを含みます。
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// 次のトークンを 1 つ返します。
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        if self.pos >= self.bytes.len() {
            return Token::new(TokenKind::Eof, Span::new(self.pos, self.pos));
        }

        let start = self.pos;
        let ch = self.bytes[self.pos];

        match ch {
            b'"' => self.lex_string_literal(start),
            b'/' => self.lex_slash(start),
            b'0'..=b'9' => self.lex_number(start),
            b'@' => self.lex_at_field(start),
            b'#' => self.lex_hash_field(start),
            b'|' => self.single_char_token(TokenKind::Pipe, start),
            b'(' => self.single_char_token(TokenKind::LParen, start),
            b')' => self.single_char_token(TokenKind::RParen, start),
            b'[' => self.single_char_token(TokenKind::LBracket, start),
            b']' => self.single_char_token(TokenKind::RBracket, start),
            b'{' => self.single_char_token(TokenKind::LBrace, start),
            b'}' => self.single_char_token(TokenKind::RBrace, start),
            b',' => self.single_char_token(TokenKind::Comma, start),
            b'.' => self.single_char_token(TokenKind::Dot, start),
            b'+' => self.single_char_token(TokenKind::Plus, start),
            b'-' => self.single_char_token(TokenKind::Minus, start),
            b'%' => self.single_char_token(TokenKind::Percent, start),
            b';' => self.single_char_token(TokenKind::Semicolon, start),
            b'?' => self.lex_parameter(start),
            b'$' => self.lex_saved_query(start),
            b'`' => self.lex_backtick_identifier(start),
            b'*' => self.lex_star_or_wildcard(start),
            b'=' => self.lex_eq(start),
            b'!' => self.lex_bang(start),
            b':' => self.lex_colon(start),
            b'<' => self.lex_lt(start),
            b'>' => self.lex_gt(start),
            _ if is_ident_start(ch) => self.lex_identifier_or_wildcard(start),
            _ if !ch.is_ascii() => {
                // 連続する非ASCIIバイトをまとめて1つのErrorトークンにします
                while self.pos < self.bytes.len() && !self.bytes[self.pos].is_ascii() {
                    self.pos += 1;
                }
                let text = String::from_utf8_lossy(&self.bytes[start..self.pos]);
                Token::new(
                    TokenKind::Error(format!("unexpected text: '{}'", text)),
                    Span::new(start, self.pos),
                )
            }
            _ => {
                self.pos += 1;
                Token::new(
                    TokenKind::Error(format!("unexpected character: '{}'", ch as char)),
                    Span::new(start, self.pos),
                )
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // 空白をスキップします
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }

            if self.pos + 1 < self.bytes.len() {
                if self.bytes[self.pos] == b'/' && self.bytes[self.pos + 1] == b'/' {
                    // 単一行コメントをスキップします
                    self.pos += 2;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                    continue;
                }

                if self.bytes[self.pos] == b'/' && self.bytes[self.pos + 1] == b'*' {
                    // 複数行コメントをスキップします
                    self.pos += 2;
                    while self.pos + 1 < self.bytes.len() {
                        if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }

            break;
        }
    }

    fn single_char_token(&mut self, kind: TokenKind, start: usize) -> Token {
        self.pos += 1;
        Token::new(kind, Span::new(start, self.pos))
    }

    fn lex_string_literal(&mut self, start: usize) -> Token {
        self.pos += 1; // 開始の " をスキップします
        let mut value = String::new();

        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'"' => {
                    self.pos += 1;
                    return Token::new(TokenKind::StringLiteral(value), Span::new(start, self.pos));
                }
                b'\\' => {
                    self.pos += 1;
                    if self.pos < self.bytes.len() {
                        match self.bytes[self.pos] {
                            b'"' => value.push('"'),
                            b'\\' => value.push('\\'),
                            b'n' => value.push('\n'),
                            b't' => value.push('\t'),
                            b'r' => value.push('\r'),
                            other => {
                                value.push('\\');
                                value.push(other as char);
                            }
                        }
                        self.pos += 1;
                    }
                }
                _ => {
                    value.push(self.bytes[self.pos] as char);
                    self.pos += 1;
                }
            }
        }

        Token::new(
            TokenKind::Error("unterminated string literal".to_string()),
            Span::new(start, self.pos),
        )
    }

    fn lex_slash(&mut self, start: usize) -> Token {
        // コメントは skip_whitespace_and_comments で処理済みなので、
        // ここに到達する `/` は正規表現リテラルか除算演算子です。
        // 正規表現リテラルの判定: `/` の後に内容があり、対応する閉じ `/` がある場合
        if self.try_lex_regex(start) {
            return self.lex_regex(start);
        }

        self.pos += 1;
        Token::new(TokenKind::Slash, Span::new(start, self.pos))
    }

    fn try_lex_regex(&self, start: usize) -> bool {
        // `/` の後を先読みして、正規表現リテラルとして妥当か確認します
        let mut i = start + 1;
        if i >= self.bytes.len() {
            return false;
        }
        // 空の正規表現 `//` はコメントとして扱われるため、正規表現にはなりません
        if self.bytes[i] == b'/' {
            return false;
        }
        // 閉じ `/` を探します
        while i < self.bytes.len() {
            match self.bytes[i] {
                b'/' => return true,
                b'\\' => {
                    i += 2; // エスケープされた文字をスキップします
                }
                b'\n' => return false, // 改行があれば正規表現ではありません
                _ => i += 1,
            }
        }
        false
    }

    fn lex_regex(&mut self, start: usize) -> Token {
        self.pos += 1; // 開始の `/` をスキップします
        let mut pattern = String::new();

        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'/' => {
                    self.pos += 1;
                    // フラグを読み取ります (i, m, s など)
                    let mut flags = String::new();
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphabetic()
                    {
                        flags.push(self.bytes[self.pos] as char);
                        self.pos += 1;
                    }
                    return Token::new(
                        TokenKind::Regex { pattern, flags },
                        Span::new(start, self.pos),
                    );
                }
                b'\\' => {
                    pattern.push('\\');
                    self.pos += 1;
                    if self.pos < self.bytes.len() {
                        pattern.push(self.bytes[self.pos] as char);
                        self.pos += 1;
                    }
                }
                _ => {
                    pattern.push(self.bytes[self.pos] as char);
                    self.pos += 1;
                }
            }
        }

        Token::new(
            TokenKind::Error("unterminated regex literal".to_string()),
            Span::new(start, self.pos),
        )
    }

    fn lex_number(&mut self, start: usize) -> Token {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        // 小数点があるか確認します
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            // `.` の後に数字が続く場合のみ浮動小数点として扱います
            if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1].is_ascii_digit() {
                self.pos += 1; // `.` をスキップします
                while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                let text = &self.source[start..self.pos];
                let value = text.parse::<f64>().unwrap_or(0.0);
                return Token::new(TokenKind::Float(value), Span::new(start, self.pos));
            }
        }

        let text = &self.source[start..self.pos];
        let value = text.parse::<i64>().unwrap_or(0);
        Token::new(TokenKind::Integer(value), Span::new(start, self.pos))
    }

    fn lex_at_field(&mut self, start: usize) -> Token {
        self.pos += 1; // `@` をスキップします
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let name = self.source[start..self.pos].to_string();
        Token::new(TokenKind::AtField(name), Span::new(start, self.pos))
    }

    fn lex_hash_field(&mut self, start: usize) -> Token {
        self.pos += 1; // `#` をスキップします
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let name = self.source[start..self.pos].to_string();
        Token::new(TokenKind::HashField(name), Span::new(start, self.pos))
    }

    fn lex_star_or_wildcard(&mut self, start: usize) -> Token {
        self.pos += 1; // `*` をスキップします

        // `*` の後に識別子文字が続く場合はワイルドカードとして扱います
        if self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            while self.pos < self.bytes.len()
                && (is_ident_continue(self.bytes[self.pos]) || self.bytes[self.pos] == b'*')
            {
                self.pos += 1;
            }
            let text = self.source[start..self.pos].to_string();
            return Token::new(TokenKind::Wildcard(text), Span::new(start, self.pos));
        }

        Token::new(TokenKind::Star, Span::new(start, self.pos))
    }

    fn lex_bang(&mut self, start: usize) -> Token {
        self.pos += 1;
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
            self.pos += 1;
            return Token::new(TokenKind::NotEq, Span::new(start, self.pos));
        }
        Token::new(TokenKind::Bang, Span::new(start, self.pos))
    }

    fn lex_colon(&mut self, start: usize) -> Token {
        self.pos += 1;
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
            self.pos += 1;
            return Token::new(TokenKind::Assign, Span::new(start, self.pos));
        }
        Token::new(TokenKind::Colon, Span::new(start, self.pos))
    }

    fn lex_lt(&mut self, start: usize) -> Token {
        self.pos += 1;
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
            self.pos += 1;
            // `<=>` の判定
            if self.pos < self.bytes.len() && self.bytes[self.pos] == b'>' {
                self.pos += 1;
                return Token::new(TokenKind::Link, Span::new(start, self.pos));
            }
            return Token::new(TokenKind::LtEq, Span::new(start, self.pos));
        }
        Token::new(TokenKind::Lt, Span::new(start, self.pos))
    }

    fn lex_gt(&mut self, start: usize) -> Token {
        self.pos += 1;
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
            self.pos += 1;
            return Token::new(TokenKind::GtEq, Span::new(start, self.pos));
        }
        Token::new(TokenKind::Gt, Span::new(start, self.pos))
    }

    fn lex_eq(&mut self, start: usize) -> Token {
        self.pos += 1;
        if self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'~' => {
                    self.pos += 1;
                    return Token::new(TokenKind::MatchOp, Span::new(start, self.pos));
                }
                b'=' => {
                    self.pos += 1;
                    return Token::new(TokenKind::EqEq, Span::new(start, self.pos));
                }
                b'>' => {
                    self.pos += 1;
                    return Token::new(TokenKind::FatArrow, Span::new(start, self.pos));
                }
                _ => {}
            }
        }
        Token::new(TokenKind::Eq, Span::new(start, self.pos))
    }

    fn lex_parameter(&mut self, start: usize) -> Token {
        self.pos += 1; // `?` をスキップします
        // `?"Descriptive Label"` 形式
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'"' {
            self.pos += 1; // `"` をスキップします
            let content_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
                self.pos += 1;
            }
            let content = self.source[content_start..self.pos].to_string();
            if self.pos < self.bytes.len() {
                self.pos += 1; // 閉じ `"` をスキップします
            }
            return Token::new(TokenKind::Parameter(content), Span::new(start, self.pos));
        }
        // `?{param=default}` 形式
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'{' {
            self.pos += 1; // `{` をスキップします
            let content_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos] != b'}' {
                self.pos += 1;
            }
            let content = self.source[content_start..self.pos].to_string();
            if self.pos < self.bytes.len() {
                self.pos += 1; // `}` をスキップします
            }
            return Token::new(TokenKind::Parameter(content), Span::new(start, self.pos));
        }
        // `?paramName` 形式
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let name = self.source[start + 1..self.pos].to_string();
        Token::new(TokenKind::Parameter(name), Span::new(start, self.pos))
    }

    fn lex_saved_query(&mut self, start: usize) -> Token {
        self.pos += 1; // `$` をスキップします
        // `$"quoted name"` 形式
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'"' {
            self.pos += 1;
            let content_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
                self.pos += 1;
            }
            let name = self.source[content_start..self.pos].to_string();
            if self.pos < self.bytes.len() {
                self.pos += 1;
            }
            return Token::new(TokenKind::SavedQuery(name), Span::new(start, self.pos));
        }
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }
        let name = self.source[start + 1..self.pos].to_string();
        Token::new(TokenKind::SavedQuery(name), Span::new(start, self.pos))
    }

    fn lex_backtick_identifier(&mut self, start: usize) -> Token {
        self.pos += 1; // 開始の ` をスキップします
        let content_start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'`' {
            self.pos += 1;
        }
        let name = self.source[content_start..self.pos].to_string();
        if self.pos < self.bytes.len() {
            self.pos += 1; // 閉じの ` をスキップします
        } else {
            return Token::new(
                TokenKind::Error("unterminated backtick identifier".to_string()),
                Span::new(start, self.pos),
            );
        }
        Token::new(
            TokenKind::QuotedIdentifier(name),
            Span::new(start, self.pos),
        )
    }

    fn lex_identifier_or_wildcard(&mut self, start: usize) -> Token {
        while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
            self.pos += 1;
        }

        // 識別子の後に `*` が続く場合はワイルドカードとして扱います
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'*' {
            self.pos += 1;
            // ワイルドカードの残りを読み取ります (例: foo*bar*)
            while self.pos < self.bytes.len()
                && (is_ident_continue(self.bytes[self.pos]) || self.bytes[self.pos] == b'*')
            {
                self.pos += 1;
            }
            let text = self.source[start..self.pos].to_string();
            return Token::new(TokenKind::Wildcard(text), Span::new(start, self.pos));
        }

        // ドット区切りの識別子に対応します (例: array:contains, time:hour)
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b':' {
            // `:=` でない場合のみ名前空間として扱います
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos + 1] != b'='
                && is_ident_start(self.bytes[self.pos + 1])
            {
                self.pos += 1; // `:` をスキップします
                while self.pos < self.bytes.len() && is_ident_continue(self.bytes[self.pos]) {
                    self.pos += 1;
                }
            }
        }

        let text = &self.source[start..self.pos];
        let kind = match text {
            "and" | "AND" => TokenKind::And,
            "or" | "OR" => TokenKind::Or,
            "not" | "NOT" => TokenKind::Not,
            "like" | "LIKE" => TokenKind::Like,
            "as" => TokenKind::As,
            "case" => TokenKind::Case,
            "match" => TokenKind::Match,
            "in" | "IN" => TokenKind::In,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text.to_string()),
        };
        Token::new(kind, Span::new(start, self.pos))
    }
}

fn is_ident_start(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_'
}

fn is_ident_continue(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize()
    }

    fn kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(kinds(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(kinds("   \n\t  "), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_single_line_comment() {
        assert_eq!(kinds("// this is a comment\n"), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_multi_line_comment() {
        assert_eq!(kinds("/* comment */"), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_comment_before_token() {
        assert_eq!(
            kinds("// comment\nfoo"),
            vec![TokenKind::Identifier("foo".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_integer_literal() {
        assert_eq!(kinds("42"), vec![TokenKind::Integer(42), TokenKind::Eof]);
    }

    #[test]
    fn test_float_literal() {
        assert_eq!(kinds("3.14"), vec![TokenKind::Float(3.14), TokenKind::Eof]);
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            kinds(r#""hello world""#),
            vec![
                TokenKind::StringLiteral("hello world".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_escape() {
        assert_eq!(
            kinds(r#""say \"hi\"""#),
            vec![
                TokenKind::StringLiteral("say \"hi\"".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_unterminated_string() {
        let tokens = kinds(r#""unterminated"#);
        assert!(matches!(&tokens[0], TokenKind::Error(_)));
    }

    #[test]
    fn test_regex_literal() {
        assert_eq!(
            kinds("/error/i"),
            vec![
                TokenKind::Regex {
                    pattern: "error".to_string(),
                    flags: "i".to_string()
                },
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_regex_no_flags() {
        assert_eq!(
            kinds("/login/"),
            vec![
                TokenKind::Regex {
                    pattern: "login".to_string(),
                    flags: String::new()
                },
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_regex_with_escape() {
        assert_eq!(
            kinds(r"/foo\.bar/"),
            vec![
                TokenKind::Regex {
                    pattern: r"foo\.bar".to_string(),
                    flags: String::new()
                },
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            kinds("and or not like as case match in true false"),
            vec![
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Not,
                TokenKind::Like,
                TokenKind::As,
                TokenKind::Case,
                TokenKind::Match,
                TokenKind::In,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords_uppercase() {
        assert_eq!(
            kinds("AND OR NOT LIKE IN"),
            vec![
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Not,
                TokenKind::Like,
                TokenKind::In,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            kinds("statuscode"),
            vec![
                TokenKind::Identifier("statuscode".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_at_field() {
        assert_eq!(
            kinds("@timestamp"),
            vec![TokenKind::AtField("@timestamp".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_namespaced_function() {
        assert_eq!(
            kinds("array:contains"),
            vec![
                TokenKind::Identifier("array:contains".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            kinds("= != := < <= > >= <=> | ! + - * /"),
            vec![
                TokenKind::Eq,
                TokenKind::NotEq,
                TokenKind::Assign,
                TokenKind::Lt,
                TokenKind::LtEq,
                TokenKind::Gt,
                TokenKind::GtEq,
                TokenKind::Link,
                TokenKind::Pipe,
                TokenKind::Bang,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                // `/` はここでは正規表現ではなく Slash になる（後に閉じ `/` がないため）
                // ただしテスト上、`/` 単体は次が Eof なので Slash になります
                TokenKind::Slash,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_delimiters() {
        assert_eq!(
            kinds("( ) [ ] { } , ."),
            vec![
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_wildcard_prefix() {
        assert_eq!(
            kinds("*login*"),
            vec![TokenKind::Wildcard("*login*".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_wildcard_suffix() {
        assert_eq!(
            kinds("error*"),
            vec![TokenKind::Wildcard("error*".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_star_alone() {
        assert_eq!(kinds("*"), vec![TokenKind::Star, TokenKind::Eof]);
    }

    #[test]
    fn test_pipeline_query() {
        let tokens = kinds(r#"status = 404 | count()"#);
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("status".to_string()),
                TokenKind::Eq,
                TokenKind::Integer(404),
                TokenKind::Pipe,
                TokenKind::Identifier("count".to_string()),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_complex_query() {
        let tokens = kinds(r#"src="client" ip="127.0.0.1" | groupBy(field=src)"#);
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("src".to_string()),
                TokenKind::Eq,
                TokenKind::StringLiteral("client".to_string()),
                TokenKind::Identifier("ip".to_string()),
                TokenKind::Eq,
                TokenKind::StringLiteral("127.0.0.1".to_string()),
                TokenKind::Pipe,
                TokenKind::Identifier("groupBy".to_string()),
                TokenKind::LParen,
                TokenKind::Identifier("field".to_string()),
                TokenKind::Eq,
                TokenKind::Identifier("src".to_string()),
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_field_assignment() {
        let tokens = kinds("newField := value + 1");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("newField".to_string()),
                TokenKind::Assign,
                TokenKind::Identifier("value".to_string()),
                TokenKind::Plus,
                TokenKind::Integer(1),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_field_filter_with_regex() {
        let tokens = kinds("url = /login/i");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("url".to_string()),
                TokenKind::Eq,
                TokenKind::Regex {
                    pattern: "login".to_string(),
                    flags: "i".to_string()
                },
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_logical_expression() {
        let tokens = kinds("statuscode=404 and (method=GET or method=POST)");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("statuscode".to_string()),
                TokenKind::Eq,
                TokenKind::Integer(404),
                TokenKind::And,
                TokenKind::LParen,
                TokenKind::Identifier("method".to_string()),
                TokenKind::Eq,
                TokenKind::Identifier("GET".to_string()),
                TokenKind::Or,
                TokenKind::Identifier("method".to_string()),
                TokenKind::Eq,
                TokenKind::Identifier("POST".to_string()),
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_negated_function() {
        let tokens = kinds("!cidr(ip, subnet=\"127.0.0/16\")");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Bang,
                TokenKind::Identifier("cidr".to_string()),
                TokenKind::LParen,
                TokenKind::Identifier("ip".to_string()),
                TokenKind::Comma,
                TokenKind::Identifier("subnet".to_string()),
                TokenKind::Eq,
                TokenKind::StringLiteral("127.0.0/16".to_string()),
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_span_positions() {
        let tokens = tokenize("foo = 42");
        assert_eq!(tokens[0].span, Span::new(0, 3)); // foo
        assert_eq!(tokens[1].span, Span::new(4, 5)); // =
        assert_eq!(tokens[2].span, Span::new(6, 8)); // 42
    }

    #[test]
    fn test_colon_not_namespace() {
        // `:=` の場合は名前空間ではなく Assign になります
        let tokens = kinds("field:=value");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("field".to_string()),
                TokenKind::Assign,
                TokenKind::Identifier("value".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_like_operator() {
        let tokens = kinds(r#"class like "Bucket""#);
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("class".to_string()),
                TokenKind::Like,
                TokenKind::StringLiteral("Bucket".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_numeric_comparison() {
        let tokens = kinds("statuscode >= 400");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("statuscode".to_string()),
                TokenKind::GtEq,
                TokenKind::Integer(400),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multibyte_error_is_single_token() {
        let tokens = tokenize("これはテスト");
        assert_eq!(tokens.len(), 2); // Error + Eof
        assert!(matches!(&tokens[0].kind, TokenKind::Error(msg) if msg.contains("これはテスト")));
        assert_eq!(tokens[0].span, Span::new(0, 18));
    }

    #[test]
    fn test_multibyte_before_pipe() {
        let tokens = tokenize("日本語 | count()");
        assert!(matches!(&tokens[0].kind, TokenKind::Error(msg) if msg.contains("日本語")));
        assert_eq!(tokens[1].kind, TokenKind::Pipe);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("count".to_string()));
    }

    #[test]
    fn test_multibyte_mixed_with_ascii() {
        let tokens = tokenize("あ=い");
        assert!(matches!(&tokens[0].kind, TokenKind::Error(_)));
        assert_eq!(tokens[1].kind, TokenKind::Eq);
        assert!(matches!(&tokens[2].kind, TokenKind::Error(_)));
    }

    #[test]
    fn test_match_op() {
        assert_eq!(
            kinds("ip =~ cidr()"),
            vec![
                TokenKind::Identifier("ip".to_string()),
                TokenKind::MatchOp,
                TokenKind::Identifier("cidr".to_string()),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_eq_eq() {
        assert_eq!(
            kinds("a == b"),
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::EqEq,
                TokenKind::Identifier("b".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_fat_arrow() {
        assert_eq!(
            kinds("x => y"),
            vec![
                TokenKind::Identifier("x".to_string()),
                TokenKind::FatArrow,
                TokenKind::Identifier("y".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_percent() {
        assert_eq!(
            kinds("a % b"),
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::Percent,
                TokenKind::Identifier("b".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_semicolon() {
        assert_eq!(kinds(";"), vec![TokenKind::Semicolon, TokenKind::Eof]);
    }

    #[test]
    fn test_parameter_simple() {
        assert_eq!(
            kinds("?myParam"),
            vec![TokenKind::Parameter("myParam".to_string()), TokenKind::Eof,]
        );
    }

    #[test]
    fn test_parameter_with_default() {
        assert_eq!(
            kinds("?{param=default}"),
            vec![
                TokenKind::Parameter("param=default".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_parameter_quoted() {
        assert_eq!(
            kinds(r#"?"Search Term""#),
            vec![
                TokenKind::Parameter("Search Term".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_saved_query() {
        assert_eq!(
            kinds("$mySearch()"),
            vec![
                TokenKind::SavedQuery("mySearch".to_string()),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_backtick_identifier() {
        assert_eq!(
            kinds("`field name`"),
            vec![
                TokenKind::QuotedIdentifier("field name".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_unterminated_backtick() {
        let tokens = kinds("`unterminated");
        assert!(matches!(&tokens[0], TokenKind::Error(_)));
    }

    #[test]
    fn test_eq_still_works() {
        // `=` 単体が引き続き Eq として認識されることを確認します
        assert_eq!(
            kinds("a = b"),
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::Eq,
                TokenKind::Identifier("b".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_eq_variants_in_sequence() {
        assert_eq!(
            kinds("= == =~ =>"),
            vec![
                TokenKind::Eq,
                TokenKind::EqEq,
                TokenKind::MatchOp,
                TokenKind::FatArrow,
                TokenKind::Eof,
            ]
        );
    }
}
