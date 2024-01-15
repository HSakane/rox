#[derive(Debug)]
pub struct Position {
    pub line: i32,
    pub column: i32,
    pub length: i32,
}

impl Position {
    pub fn new(line: i32, column: i32, length: i32) -> Self {
        Position {
            line,
            column,
            length,
        }
    }
}

#[derive(Debug)]
pub enum Token {
    WhiteSpace(Position),
    LineFeed(Position),
    LeftBrace(Position),
    RightBrace(Position),
    LeftBracket(Position),
    RightBracket(Position),
    LeftParen(Position),
    RightParen(Position),
    Comma(Position),
    Dot(Position),
    Minus(Position),
    Plus(Position),
    Semicolon(Position),
    Slash(Position),
    Star(Position),
    Pow(Position),
    Percent(Position),
    Bang(Position),
    BangEqual(Position),
    Equal(Position),
    EqualEqual(Position),
    Greater(Position),
    GreaterEqual(Position),
    Less(Position),
    LessEqual(Position),
    Identifer { position: Position, value: String },
    String { position: Position, value: String },
    Float { position: Position, value: f64 },
    Integer { position: Position, value: i64 },
    // キーワード
    And(Position),
    Class(Position),
    Else(Position),
    False(Position),
    For(Position),
    Fun(Position),
    If(Position),
    Null(Position),
    Or(Position),
    Return(Position),
    True(Position),
    Var(Position),
    While(Position),
    In(Position),
    Print(Position),
    This(Position),
    Super(Position),
    To(Position),
}
