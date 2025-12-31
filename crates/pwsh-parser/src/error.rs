/// Parser errors
use pwsh_lexer::{Position, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token encountered
    UnexpectedToken {
        expected: String,
        found: Token,
        position: Position,
    },
    /// Unexpected end of input
    UnexpectedEof { expected: String },
    /// Invalid expression
    InvalidExpression { message: String, position: Position },
    /// Invalid statement
    InvalidStatement { message: String, position: Position },
    /// Invalid operator
    InvalidOperator { operator: Token, position: Position },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token at line {}, column {}: expected {}, found {}",
                    position.line, position.column, expected, found
                )
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of input: expected {}", expected)
            }
            ParseError::InvalidExpression { message, position } => {
                write!(
                    f,
                    "Invalid expression at line {}, column {}: {}",
                    position.line, position.column, message
                )
            }
            ParseError::InvalidStatement { message, position } => {
                write!(
                    f,
                    "Invalid statement at line {}, column {}: {}",
                    position.line, position.column, message
                )
            }
            ParseError::InvalidOperator { operator, position } => {
                write!(
                    f,
                    "Invalid operator {} at line {}, column {}",
                    operator, position.line, position.column
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
