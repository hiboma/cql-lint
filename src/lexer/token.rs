use serde::Serialize;

/// ソースコード上の位置を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// 開始位置 (バイトオフセット)
    pub start: usize,
    /// 終了位置 (バイトオフセット、排他的)
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// トークンの種類を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // リテラル
    /// 整数リテラル
    Integer(i64),
    /// 浮動小数点リテラル
    Float(f64),
    /// 文字列リテラル ("..." で囲まれた文字列)
    StringLiteral(String),
    /// 正規表現リテラル (/pattern/flags)
    Regex {
        pattern: String,
        flags: String,
    },
    /// ワイルドカード付き文字列 (*foo*, foo*)
    Wildcard(String),

    // 識別子・キーワード
    /// 識別子 (フィールド名、関数名など)
    Identifier(String),
    /// @ 付きフィールド (@timestamp, @id など)
    AtField(String),
    /// # 付きタグフィールド (#event_simpleName など)
    HashField(String),
    /// ユーザーパラメータ (?param, ?{param=default})
    Parameter(String),
    /// 保存済みクエリ参照 ($savedQuery)
    SavedQuery(String),
    /// バッククォート付き識別子
    QuotedIdentifier(String),

    // キーワード
    And,
    Or,
    Not,
    Like,
    As,
    Case,
    Match,
    In,
    True,
    False,

    // 演算子
    /// `=`
    Eq,
    /// `!=`
    NotEq,
    /// `=~`
    MatchOp,
    /// `==`
    EqEq,
    /// `=>`
    FatArrow,
    /// `:=`
    Assign,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `<=>`
    Link,
    /// `|`
    Pipe,
    /// `!`
    Bang,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,

    // 区切り記号
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `:`
    Colon,
    /// `;`
    Semicolon,

    // 特殊
    /// ファイル終端
    Eof,
    /// 不正なトークン
    Error(String),
}

/// ソース位置付きのトークンを表します。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
