use crate::token::{LocatedToken, Position, StringPart, Token};

/// Lexer errors
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { ch: char, position: Position },
    UnterminatedString { position: Position },
    InvalidNumber { text: String, position: Position },
    InvalidToken { text: String, position: Position },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LexError::UnexpectedCharacter { ch, position } => {
                write!(
                    f,
                    "Unexpected character '{}' at line {}, column {}",
                    ch, position.line, position.column
                )
            }
            LexError::UnterminatedString { position } => {
                write!(
                    f,
                    "Unterminated string at line {}, column {}",
                    position.line, position.column
                )
            }
            LexError::InvalidNumber { text, position } => {
                write!(
                    f,
                    "Invalid number '{}' at line {}, column {}",
                    text, position.line, position.column
                )
            }
            LexError::InvalidToken { text, position } => {
                write!(
                    f,
                    "Invalid token '{}' at line {}, column {}",
                    text, position.line, position.column
                )
            }
        }
    }
}

impl std::error::Error for LexError {}

/// PowerShell lexer/tokenizer
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from input string
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get current position
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    /// Peek at current character without consuming
    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    /// Peek ahead n characters
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.position + n;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    /// Advance to next character
    fn advance(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace (except newlines, which are significant in PowerShell)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip comments (# to end of line)
    fn skip_comment(&mut self) {
        if self.peek() == Some('#') {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    /// Process an escape sequence and return the resulting character(s)
    fn process_escape(&mut self) -> Option<String> {
        self.advance(); // consume backslash
        self.advance().map(|escaped| match escaped {
            'n' => "\n".to_string(),
            'r' => "\r".to_string(),
            't' => "\t".to_string(),
            '\\' => "\\".to_string(),
            '"' => "\"".to_string(),
            '\'' => "'".to_string(),
            '$' => "$".to_string(),
            // Unknown escape sequences are kept as-is (PowerShell behavior)
            _ => format!("\\{}", escaped),
        })
    }

    /// Check if character sequence looks like a variable reference
    fn is_variable_start(&self) -> bool {
        self.peek() == Some('$')
            && self
                .peek_ahead(1)
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false)
    }

    /// Read a string literal
    fn read_string(&mut self, quote: char) -> Result<String, LexError> {
        let start_pos = self.current_position();
        self.advance(); // consume opening quote

        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(result);
            } else if ch == '\\' && self.peek_ahead(1).is_some() {
                // Handle escape sequences
                if let Some(escaped_str) = self.process_escape() {
                    result.push_str(&escaped_str);
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(LexError::UnterminatedString {
            position: start_pos,
        })
    }

    /// Read an interpolated string (double-quoted with variables)
    fn read_interpolated_string(&mut self) -> Result<Vec<StringPart>, LexError> {
        let start_pos = self.current_position();
        self.advance(); // consume opening quote

        let mut parts = Vec::new();
        let mut current_literal = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                // End of string
                if !current_literal.is_empty() {
                    parts.push(StringPart::Literal(current_literal));
                }
                self.advance(); // consume closing quote
                return Ok(parts);
            } else if self.is_variable_start() {
                // Variable interpolation
                if !current_literal.is_empty() {
                    parts.push(StringPart::Literal(current_literal.clone()));
                    current_literal.clear();
                }

                self.advance(); // consume $
                let var_name = self.read_identifier();
                parts.push(StringPart::Variable(var_name));
            } else if ch == '\\' && self.peek_ahead(1).is_some() {
                // Handle escape sequences
                if let Some(escaped_str) = self.process_escape() {
                    current_literal.push_str(&escaped_str);
                }
            } else {
                current_literal.push(ch);
                self.advance();
            }
        }

        Err(LexError::UnterminatedString {
            position: start_pos,
        })
    }

    /// Read a number literal
    fn read_number(&mut self) -> Result<f64, LexError> {
        let start_pos = self.current_position();
        let mut num_str = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        num_str.parse::<f64>().map_err(|_| LexError::InvalidNumber {
            text: num_str,
            position: start_pos,
        })
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    /// Read a variable name (after $)
    fn read_variable(&mut self) -> Result<String, LexError> {
        let start_pos = self.current_position();
        self.advance(); // consume $

        let mut var_name = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                var_name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if var_name.is_empty() {
            Err(LexError::InvalidToken {
                text: "$".to_string(),
                position: start_pos,
            })
        } else {
            Ok(var_name)
        }
    }

    /// Check if identifier is a keyword
    fn keyword_or_identifier(&self, ident: &str) -> Token {
        match ident.to_lowercase().as_str() {
            "if" => Token::If,
            "else" => Token::Else,
            "elseif" => Token::ElseIf,
            "function" => Token::Function,
            "return" => Token::Return,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => Token::Identifier(ident.to_string()),
        }
    }

    /// Read operator starting with '-'
    fn read_operator(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        self.advance(); // consume '-'

        let op_name = self.read_identifier();

        match op_name.to_lowercase().as_str() {
            "eq" => Ok(Token::Equal),
            "ne" => Ok(Token::NotEqual),
            "gt" => Ok(Token::Greater),
            "lt" => Ok(Token::Less),
            "ge" => Ok(Token::GreaterOrEqual),
            "le" => Ok(Token::LessOrEqual),
            _ => Err(LexError::InvalidToken {
                text: format!("-{}", op_name),
                position: start_pos,
            }),
        }
    }

    /// Get next token
    pub fn next_token(&mut self) -> Result<LocatedToken, LexError> {
        self.skip_whitespace();

        // Skip comments
        if self.peek() == Some('#') {
            self.skip_comment();
            self.skip_whitespace();
        }

        let position = self.current_position();

        match self.peek() {
            None => Ok(LocatedToken::new(Token::Eof, position)),
            Some('\n') => {
                self.advance();
                Ok(LocatedToken::new(Token::Newline, position))
            }
            Some(';') => {
                self.advance();
                Ok(LocatedToken::new(Token::Semicolon, position))
            }
            Some('(') => {
                self.advance();
                Ok(LocatedToken::new(Token::LeftParen, position))
            }
            Some(')') => {
                self.advance();
                Ok(LocatedToken::new(Token::RightParen, position))
            }
            Some('{') => {
                self.advance();
                Ok(LocatedToken::new(Token::LeftBrace, position))
            }
            Some('}') => {
                self.advance();
                Ok(LocatedToken::new(Token::RightBrace, position))
            }
            Some('[') => {
                self.advance();
                Ok(LocatedToken::new(Token::LeftBracket, position))
            }
            Some(']') => {
                self.advance();
                Ok(LocatedToken::new(Token::RightBracket, position))
            }
            Some(',') => {
                self.advance();
                Ok(LocatedToken::new(Token::Comma, position))
            }
            Some('.') => {
                self.advance();
                Ok(LocatedToken::new(Token::Dot, position))
            }
            Some('|') => {
                self.advance();
                Ok(LocatedToken::new(Token::Pipeline, position))
            }
            Some('=') => {
                self.advance();
                Ok(LocatedToken::new(Token::Assignment, position))
            }
            Some('+') => {
                self.advance();
                Ok(LocatedToken::new(Token::Plus, position))
            }
            Some('*') => {
                self.advance();
                Ok(LocatedToken::new(Token::Multiply, position))
            }
            Some('/') => {
                self.advance();
                Ok(LocatedToken::new(Token::Divide, position))
            }
            Some('%') => {
                self.advance();
                Ok(LocatedToken::new(Token::Modulo, position))
            }
            Some('-') => {
                // Could be minus or an operator like -eq
                if let Some(next) = self.peek_ahead(1) {
                    if next.is_alphabetic() {
                        // It's an operator like -eq
                        let token = self.read_operator()?;
                        Ok(LocatedToken::new(token, position))
                    } else {
                        // It's a minus
                        self.advance();
                        Ok(LocatedToken::new(Token::Minus, position))
                    }
                } else {
                    self.advance();
                    Ok(LocatedToken::new(Token::Minus, position))
                }
            }
            Some('"') => {
                // Check if string contains variable interpolation
                let parts = self.read_interpolated_string()?;
                if parts.is_empty() {
                    // Empty string
                    Ok(LocatedToken::new(Token::String(String::new()), position))
                } else if parts.len() == 1 {
                    // Single part - could be literal or variable
                    match &parts[0] {
                        StringPart::Literal(s) => {
                            Ok(LocatedToken::new(Token::String(s.clone()), position))
                        }
                        StringPart::Variable(_) => {
                            // Single variable in quotes - still interpolated
                            Ok(LocatedToken::new(
                                Token::InterpolatedString(parts),
                                position,
                            ))
                        }
                    }
                } else {
                    // Multiple parts - definitely interpolated
                    Ok(LocatedToken::new(
                        Token::InterpolatedString(parts),
                        position,
                    ))
                }
            }
            Some('\'') => {
                let s = self.read_string('\'')?;
                Ok(LocatedToken::new(Token::String(s), position))
            }
            Some('$') => {
                let var = self.read_variable()?;
                Ok(LocatedToken::new(Token::Variable(var), position))
            }
            Some(ch) if ch.is_ascii_digit() => {
                let num = self.read_number()?;
                Ok(LocatedToken::new(Token::Number(num), position))
            }
            Some(ch) if ch.is_alphabetic() => {
                let ident = self.read_identifier();
                let token = self.keyword_or_identifier(&ident);
                Ok(LocatedToken::new(token, position))
            }
            Some(ch) => {
                self.advance();
                Err(LexError::UnexpectedCharacter { ch, position })
            }
        }
    }

    /// Tokenize entire input
    pub fn tokenize(&mut self) -> Result<Vec<LocatedToken>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.token == Token::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}
