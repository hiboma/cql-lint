use crate::lexer::token::Span;

/// クエリ全体を表します。パイプラインステージのリストです。
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub stages: Vec<PipelineStage>,
    pub span: Span,
}

/// パイプラインの 1 ステージを表します。
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStage {
    pub kind: StageKind,
    pub span: Span,
}

/// ステージの種類を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum StageKind {
    /// フィルタ式
    Filter(FilterExpr),
    /// 関数呼び出し
    FunctionCall(FunctionCall),
    /// フィールド代入 (field := expr)
    Assignment(Assignment),
    /// case 文 (case { guard | pipeline; ... })
    CaseStatement(CaseStatement),
    /// match 文 (field match { pattern => pipeline; ... })
    MatchStatement(MatchStatement),
}

/// フィルタ式を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpr {
    /// フリーテキストフィルタ
    FreeText(String),
    /// フィールドフィルタ (field op value)
    FieldFilter {
        field: String,
        op: CompareOp,
        value: FilterValue,
    },
    /// 論理 AND
    And(Box<FilterExpr>, Box<FilterExpr>),
    /// 論理 OR
    Or(Box<FilterExpr>, Box<FilterExpr>),
    /// 論理 NOT
    Not(Box<FilterExpr>),
    /// 括弧で囲まれた式
    Grouped(Box<FilterExpr>),
    /// 関数呼び出し (フィルタコンテキスト内)
    FunctionCall(FunctionCall),
    /// フィールド =~ 関数呼び出し (FieldShorthand)
    FieldShorthand {
        field: String,
        function: FunctionCall,
    },
}

/// 比較演算子を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    NotEq,
    EqEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Like,
    Link,
}

/// フィルタの右辺値を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// 文字列リテラル
    String(String),
    /// 数値
    Number(f64),
    /// 正規表現
    Regex { pattern: String, flags: String },
    /// ワイルドカード
    Wildcard(String),
    /// フィールド参照
    Field(String),
    /// ユーザーパラメータ
    Parameter(String),
}

/// 関数呼び出しを表します。
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub span: Span,
}

/// 関数の引数を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// 位置引数
    Positional(Expr),
    /// 名前付き引数 (name=value)
    Named { name: String, value: Expr },
}

/// 式を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 数値リテラル
    Number(f64),
    /// 文字列リテラル
    String(String),
    /// 真偽値リテラル
    Bool(bool),
    /// フィールド参照
    Field(String),
    /// 正規表現リテラル
    Regex { pattern: String, flags: String },
    /// ワイルドカード
    Wildcard(String),
    /// 配列 [a, b, c]
    Array(Vec<Expr>),
    /// 関数呼び出し
    FunctionCall(FunctionCall),
    /// 二項演算 (算術)
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// 単項演算
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    /// サブクエリ ({...} ブロック内のパイプライン)
    SubQuery(Box<Query>),
    /// 比較式 (引数内で使用: field != value)
    Comparison {
        left: Box<Expr>,
        op: CompareOp,
        right: Box<Expr>,
    },
    /// ユーザーパラメータ (?param, ?{param=default})
    Parameter(String),
    /// 保存済みクエリ呼び出し ($savedQuery(...))
    SavedQueryCall {
        name: String,
        arguments: Vec<Argument>,
    },
    /// 配列インデックスアクセス (field[0])
    IndexAccess { object: Box<Expr>, index: Box<Expr> },
}

/// 二項演算子を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// 単項演算子を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// フィールド代入を表します。
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub field: String,
    pub value: Expr,
    pub span: Span,
}

/// case 文を表します。
/// case { guard | pipeline; guard | pipeline; ... }
#[derive(Debug, Clone, PartialEq)]
pub struct CaseStatement {
    pub branches: Vec<CaseBranch>,
    pub span: Span,
}

/// case 文の 1 つの分岐を表します。
#[derive(Debug, Clone, PartialEq)]
pub struct CaseBranch {
    pub pipeline: Vec<PipelineStage>,
    pub span: Span,
}

/// match 文を表します。
/// field match { pattern => pipeline; pattern => pipeline; ... }
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement {
    pub field: String,
    pub branches: Vec<MatchBranch>,
    pub span: Span,
}

/// match 文の 1 つの分岐を表します。
#[derive(Debug, Clone, PartialEq)]
pub struct MatchBranch {
    pub pattern: Expr,
    pub pipeline: Vec<PipelineStage>,
    pub span: Span,
}
