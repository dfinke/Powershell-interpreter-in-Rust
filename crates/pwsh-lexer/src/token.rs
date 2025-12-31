/// Token types for PowerShell lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    String(String),
    InterpolatedString(Vec<StringPart>), // "Hello $name" - will be expanded later
    Number(f64),
    Boolean(bool),

    // Identifiers and Variables
    Identifier(String),
    Variable(String), // $varName

    // Operators - Arithmetic
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %

    // Operators - Comparison
    Equal,          // -eq
    NotEqual,       // -ne
    Greater,        // -gt
    Less,           // -lt
    GreaterOrEqual, // -ge
    LessOrEqual,    // -le

    // Keywords
    If,
    Else,
    ElseIf,
    Function,
    Return,

    // Syntax
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Dot,          // .
    Pipeline,     // |
    Assignment,   // =
    Semicolon,    // ;
    Newline,

    // Special
    Eof,
}

/// Parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::InterpolatedString(parts) => {
                write!(f, "InterpolatedString(")?;
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 { write!(f, " + ")?; }
                    match part {
                        StringPart::Literal(s) => write!(f, "\"{}\"", s)?,
                        StringPart::Variable(v) => write!(f, "${}", v)?,
                    }
                }
                write!(f, ")")
            }
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Boolean(b) => write!(f, "Boolean({})", b),
            Token::Identifier(id) => write!(f, "Identifier({})", id),
            Token::Variable(var) => write!(f, "Variable(${})", var),
            Token::Plus => write!(f, "Plus"),
            Token::Minus => write!(f, "Minus"),
            Token::Multiply => write!(f, "Multiply"),
            Token::Divide => write!(f, "Divide"),
            Token::Modulo => write!(f, "Modulo"),
            Token::Equal => write!(f, "Equal(-eq)"),
            Token::NotEqual => write!(f, "NotEqual(-ne)"),
            Token::Greater => write!(f, "Greater(-gt)"),
            Token::Less => write!(f, "Less(-lt)"),
            Token::GreaterOrEqual => write!(f, "GreaterOrEqual(-ge)"),
            Token::LessOrEqual => write!(f, "LessOrEqual(-le)"),
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::ElseIf => write!(f, "ElseIf"),
            Token::Function => write!(f, "Function"),
            Token::Return => write!(f, "Return"),
            Token::LeftParen => write!(f, "LeftParen"),
            Token::RightParen => write!(f, "RightParen"),
            Token::LeftBrace => write!(f, "LeftBrace"),
            Token::RightBrace => write!(f, "RightBrace"),
            Token::LeftBracket => write!(f, "LeftBracket"),
            Token::RightBracket => write!(f, "RightBracket"),
            Token::Comma => write!(f, "Comma"),
            Token::Dot => write!(f, "Dot"),
            Token::Pipeline => write!(f, "Pipeline"),
            Token::Assignment => write!(f, "Assignment"),
            Token::Semicolon => write!(f, "Semicolon"),
            Token::Newline => write!(f, "Newline"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}

/// Position in source code for error reporting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

/// Token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub position: Position,
}

impl LocatedToken {
    pub fn new(token: Token, position: Position) -> Self {
        LocatedToken { token, position }
    }
}
