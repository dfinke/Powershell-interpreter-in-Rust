/// Parser implementation using recursive descent with Pratt parsing for expressions
use crate::ast::*;
use crate::error::ParseError;
use pwsh_lexer::{LocatedToken, Token};

/// Parser for PowerShell code
pub struct Parser {
    tokens: Vec<LocatedToken>,
    current: usize,
}

impl Parser {
    /// Create a new parser from a token stream
    pub fn new(tokens: Vec<LocatedToken>) -> Self {
        Parser { tokens, current: 0 }
    }

    /// Parse the token stream into a program
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines and semicolons at statement level
            if self.check(&Token::Newline) || self.check(&Token::Semicolon) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    /// Parse a single statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Skip leading newlines/semicolons
        while self.check(&Token::Newline) || self.check(&Token::Semicolon) {
            self.advance();
        }

        // Check for keywords first
        if self.check(&Token::If) {
            return self.parse_if_statement();
        }

        if self.check(&Token::Function) {
            return self.parse_function_def();
        }

        if self.check(&Token::Return) {
            return self.parse_return_statement();
        }

        // Check for variable assignment
        if self.check_ahead_for_assignment() {
            return self.parse_assignment();
        }

        // Check for pipeline (contains |)
        if self.contains_pipeline() {
            let pipeline = self.parse_pipeline()?;
            self.consume_statement_terminator();
            return Ok(Statement::Pipeline(pipeline));
        }

        // Otherwise, parse as expression statement
        let expr = self.parse_expression()?;
        self.consume_statement_terminator();
        Ok(Statement::Expression(expr))
    }

    /// Parse an assignment statement: $var = expr
    fn parse_assignment(&mut self) -> Result<Statement, ParseError> {
        let var_token = self.advance();
        let variable = match &var_token.token {
            Token::Variable(name) => name.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "variable".to_string(),
                    found: var_token.token.clone(),
                    position: var_token.position,
                })
            }
        };

        self.consume(&Token::Assignment, "=")?;
        let value = self.parse_expression()?;
        self.consume_statement_terminator();

        Ok(Statement::Assignment { variable, value })
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::If, "if")?;
        self.consume(&Token::LeftParen, "(")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RightParen, ")")?;

        let then_branch = self.parse_block()?;

        let else_branch = if self.check(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// Parse a function definition
    fn parse_function_def(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Function, "function")?;

        let name_token = self.advance();
        let name = match &name_token.token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "function name".to_string(),
                    found: name_token.token.clone(),
                    position: name_token.position,
                })
            }
        };

        // Parse parameters if present
        let parameters = if self.check(&Token::LeftParen) {
            self.parse_parameters()?
        } else {
            Vec::new()
        };

        let body = self.parse_block()?;

        Ok(Statement::FunctionDef {
            name,
            parameters,
            body,
        })
    }

    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        self.consume(&Token::LeftParen, "(")?;
        let mut parameters = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param_token = self.advance();
                let name = match &param_token.token {
                    Token::Variable(n) => n.clone(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "parameter name".to_string(),
                            found: param_token.token.clone(),
                            position: param_token.position,
                        })
                    }
                };

                // Check for default value
                let default_value = if self.check(&Token::Assignment) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                parameters.push(Parameter {
                    name,
                    default_value,
                });

                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(&Token::RightParen, ")")?;
        Ok(parameters)
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(&Token::Return, "return")?;

        // Return can have an optional expression
        let value = if self.check(&Token::Newline)
            || self.check(&Token::Semicolon)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume_statement_terminator();
        Ok(Statement::Return(value))
    }

    /// Parse a block: { statements }
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.consume(&Token::LeftBrace, "{")?;

        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Skip newlines and semicolons
            if self.check(&Token::Newline) || self.check(&Token::Semicolon) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RightBrace, "}")?;

        Ok(Block { statements })
    }

    /// Parse a pipeline: expr | expr | expr
    fn parse_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let mut stages = Vec::new();

        stages.push(self.parse_expression()?);

        while self.check(&Token::Pipeline) {
            self.advance();
            stages.push(self.parse_expression()?);
        }

        Ok(Pipeline { stages })
    }

    /// Parse an expression using Pratt parsing
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression_with_precedence(0)
    }

    /// Parse expression with precedence climbing (Pratt parser)
    fn parse_expression_with_precedence(&mut self, min_precedence: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_primary()?;

        loop {
            // Check for member access first (highest precedence)
            if self.check(&Token::Dot) {
                self.advance();
                let member_token = self.advance();
                let member = match &member_token.token {
                    Token::Identifier(name) => name.clone(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "property name".to_string(),
                            found: member_token.token.clone(),
                            position: member_token.position,
                        })
                    }
                };

                left = Expression::MemberAccess {
                    object: Box::new(left),
                    member,
                };
                continue;
            }

            // Check for binary operators
            if let Some((precedence, operator)) = self.get_binary_operator() {
                if precedence < min_precedence {
                    break;
                }

                self.advance(); // consume operator
                let right = self.parse_expression_with_precedence(precedence + 1)?;

                left = Expression::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Get binary operator and its precedence
    fn get_binary_operator(&self) -> Option<(u8, BinaryOperator)> {
        let token = self.peek()?;

        match token {
            // Comparison operators (lowest precedence)
            Token::Equal => Some((1, BinaryOperator::Equal)),
            Token::NotEqual => Some((1, BinaryOperator::NotEqual)),
            Token::Greater => Some((1, BinaryOperator::Greater)),
            Token::Less => Some((1, BinaryOperator::Less)),
            Token::GreaterOrEqual => Some((1, BinaryOperator::GreaterOrEqual)),
            Token::LessOrEqual => Some((1, BinaryOperator::LessOrEqual)),

            // Additive operators
            Token::Plus => Some((2, BinaryOperator::Add)),
            Token::Minus => Some((2, BinaryOperator::Subtract)),

            // Multiplicative operators (highest precedence)
            Token::Multiply => Some((3, BinaryOperator::Multiply)),
            Token::Divide => Some((3, BinaryOperator::Divide)),
            Token::Modulo => Some((3, BinaryOperator::Modulo)),

            _ => None,
        }
    }

    /// Parse a primary expression (literals, variables, calls, etc.)
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.peek();

        if token.is_none() {
            return Err(ParseError::UnexpectedEof {
                expected: "expression".to_string(),
            });
        }

        match token.unwrap() {
            // Literals
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Number(num)))
            }
            Token::String(s) => {
                let str_val = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(str_val)))
            }
            Token::InterpolatedString(parts) => {
                let string_parts: Vec<StringPart> = parts
                    .iter()
                    .map(|p| match p {
                        pwsh_lexer::StringPart::Literal(s) => StringPart::Literal(s.clone()),
                        pwsh_lexer::StringPart::Variable(v) => StringPart::Variable(v.clone()),
                    })
                    .collect();
                self.advance();
                Ok(Expression::Literal(Literal::InterpolatedString(
                    string_parts,
                )))
            }
            Token::Boolean(b) => {
                let bool_val = *b;
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(bool_val)))
            }

            // Variable
            Token::Variable(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(Expression::Variable(var_name))
            }

            // Identifier (function/cmdlet call or bare identifier)
            Token::Identifier(name) => {
                let func_name = name.clone();
                self.advance();

                // Check if this looks like a function call (has arguments following)
                // Don't parse arguments if we see comma, pipeline, statement terminators, etc.
                // Special case: -Identifier could be a named parameter, not subtraction
                let is_named_param_following = self.check(&Token::Minus)
                    && self.current + 1 < self.tokens.len()
                    && matches!(&self.tokens[self.current + 1].token, Token::Identifier(_));

                let should_parse_args = is_named_param_following
                    || (!self.check(&Token::Comma)
                        && !self.check(&Token::Pipeline)
                        && !self.is_statement_terminator()
                        && !self.check(&Token::RightParen)
                        && !self.check(&Token::RightBrace)
                        && !self.is_binary_operator());

                let arguments = if should_parse_args {
                    self.parse_call_arguments()?
                } else {
                    Vec::new()
                };

                Ok(Expression::Call {
                    name: func_name,
                    arguments,
                })
            }

            // Parenthesized expression
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, ")")?;
                Ok(expr)
            }

            // Script block
            Token::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expression::ScriptBlock(block))
            }

            // Unary minus
            Token::Minus => {
                self.advance();
                let operand = self.parse_primary()?;
                Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(operand),
                })
            }

            _ => {
                let tok = self.advance();
                Err(ParseError::InvalidExpression {
                    message: format!("Unexpected token: {}", tok.token),
                    position: tok.position,
                })
            }
        }
    }

    /// Parse function/cmdlet call arguments
    fn parse_call_arguments(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut arguments = Vec::new();

        // Arguments can be:
        // 1. Nothing (no args)
        // 2. Positional arguments separated by whitespace or commas
        // 3. Named arguments: -Name value

        // Keep parsing arguments until we hit a statement terminator or certain operators
        while !self.is_at_end()
            && !self.is_statement_terminator()
            && !self.check(&Token::Pipeline)
            && !self.check(&Token::RightParen)
            && !self.check(&Token::RightBrace)
        {
            // Check for named parameter (-Identifier pattern)
            let is_named_param = self.check(&Token::Minus)
                && self.current + 1 < self.tokens.len()
                && matches!(&self.tokens[self.current + 1].token, Token::Identifier(_));

            if is_named_param {
                self.advance(); // consume minus
                let name_token = self.advance(); // consume identifier
                let name = match &name_token.token {
                    Token::Identifier(n) => n.clone(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "identifier after -".to_string(),
                            found: name_token.token.clone(),
                            position: name_token.position,
                        })
                    }
                };

                // Next token should be the value
                let value = self.parse_primary()?;
                arguments.push(Argument::Named { name, value });
                
                // Skip optional comma
                if self.check(&Token::Comma) {
                    self.advance();
                }
                continue;
            }

            // Check if we hit a binary operator (but not named param pattern)
            if self.is_binary_operator() {
                break;
            }

            // Otherwise, parse as positional argument
            let arg = self.parse_primary()?;
            arguments.push(Argument::Positional(arg));
            
            // Skip optional comma
            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        Ok(arguments)
    }

    // Helper methods

    /// Check if current token matches the given token type
    fn check(&self, token: &Token) -> bool {
        if let Some(current) = self.peek() {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    /// Check ahead for assignment pattern: $var =
    fn check_ahead_for_assignment(&self) -> bool {
        if self.current < self.tokens.len() {
            if let Token::Variable(_) = &self.tokens[self.current].token {
                if self.current + 1 < self.tokens.len() {
                    return matches!(&self.tokens[self.current + 1].token, Token::Assignment);
                }
            }
        }
        false
    }

    /// Check if the upcoming tokens contain a pipeline operator
    fn contains_pipeline(&self) -> bool {
        let mut i = self.current;
        let mut depth = 0;

        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::LeftParen | Token::LeftBrace | Token::LeftBracket => depth += 1,
                Token::RightParen | Token::RightBrace | Token::RightBracket => depth -= 1,
                Token::Pipeline if depth == 0 => return true,
                Token::Newline | Token::Semicolon | Token::Eof if depth == 0 => return false,
                _ => {}
            }
            i += 1;
        }

        false
    }

    /// Peek at current token without consuming
    fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current].token)
        } else {
            None
        }
    }

    /// Advance to next token and return current
    fn advance(&mut self) -> LocatedToken {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            token
        } else {
            // Return EOF with the position of the last token
            let position = if !self.tokens.is_empty() {
                self.tokens[self.tokens.len() - 1].position
            } else {
                pwsh_lexer::Position::new(1, 1)
            };
            LocatedToken::new(Token::Eof, position)
        }
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
            || matches!(
                self.tokens.get(self.current).map(|t| &t.token),
                Some(Token::Eof)
            )
    }

    /// Check if current token is a statement terminator
    fn is_statement_terminator(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Newline) | Some(Token::Semicolon) | Some(Token::Eof)
        )
    }

    /// Check if current token is a binary operator
    fn is_binary_operator(&self) -> bool {
        self.get_binary_operator().is_some()
    }

    /// Consume expected token or return error
    fn consume(&mut self, expected: &Token, description: &str) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            let token = self.advance();
            Err(ParseError::UnexpectedToken {
                expected: description.to_string(),
                found: token.token,
                position: token.position,
            })
        }
    }

    /// Consume optional statement terminator (newline, semicolon)
    fn consume_statement_terminator(&mut self) {
        while self.check(&Token::Newline) || self.check(&Token::Semicolon) {
            self.advance();
        }
    }
}
