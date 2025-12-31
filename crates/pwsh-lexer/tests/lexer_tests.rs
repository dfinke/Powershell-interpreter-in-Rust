use pwsh_lexer::{LexError, Lexer, StringPart, Token};

#[test]
fn test_tokenize_variable() {
    let mut lexer = Lexer::new("$myVar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // Variable + Eof
    assert_eq!(tokens[0].token, Token::Variable("myVar".to_string()));
    assert_eq!(tokens[1].token, Token::Eof);
}

#[test]
fn test_tokenize_number() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Number(42.0));
}

#[test]
fn test_tokenize_float() {
    let mut lexer = Lexer::new("3.5");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Number(3.5));
}

#[test]
fn test_tokenize_string_double_quotes() {
    let mut lexer = Lexer::new("\"hello world\"");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::String("hello world".to_string()));
}

#[test]
fn test_tokenize_string_single_quotes() {
    let mut lexer = Lexer::new("'hello world'");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::String("hello world".to_string()));
}

#[test]
fn test_tokenize_identifier() {
    let mut lexer = Lexer::new("myFunction");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Identifier("myFunction".to_string()));
}

#[test]
fn test_tokenize_cmdlet_name() {
    let mut lexer = Lexer::new("Get-Process");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].token,
        Token::Identifier("Get-Process".to_string())
    );
}

#[test]
fn test_tokenize_pipeline() {
    let mut lexer = Lexer::new("Get-Process | Where-Object");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // Get-Process, |, Where-Object, Eof
    assert_eq!(
        tokens[0].token,
        Token::Identifier("Get-Process".to_string())
    );
    assert_eq!(tokens[1].token, Token::Pipeline);
    assert_eq!(
        tokens[2].token,
        Token::Identifier("Where-Object".to_string())
    );
}

#[test]
fn test_tokenize_assignment() {
    let mut lexer = Lexer::new("$x = 5");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // $x, =, 5, Eof
    assert_eq!(tokens[0].token, Token::Variable("x".to_string()));
    assert_eq!(tokens[1].token, Token::Assignment);
    assert_eq!(tokens[2].token, Token::Number(5.0));
}

#[test]
fn test_tokenize_arithmetic_operators() {
    let mut lexer = Lexer::new("10 + 20 - 5 * 2 / 3");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Number(10.0));
    assert_eq!(tokens[1].token, Token::Plus);
    assert_eq!(tokens[2].token, Token::Number(20.0));
    assert_eq!(tokens[3].token, Token::Minus);
    assert_eq!(tokens[4].token, Token::Number(5.0));
    assert_eq!(tokens[5].token, Token::Multiply);
    assert_eq!(tokens[6].token, Token::Number(2.0));
    assert_eq!(tokens[7].token, Token::Divide);
    assert_eq!(tokens[8].token, Token::Number(3.0));
}

#[test]
fn test_tokenize_comparison_operators() {
    let mut lexer = Lexer::new("$x -eq 5");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Variable("x".to_string()));
    assert_eq!(tokens[1].token, Token::Equal);
    assert_eq!(tokens[2].token, Token::Number(5.0));
}

#[test]
fn test_tokenize_all_comparison_operators() {
    let mut lexer = Lexer::new("-eq -ne -gt -lt -ge -le");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Equal);
    assert_eq!(tokens[1].token, Token::NotEqual);
    assert_eq!(tokens[2].token, Token::Greater);
    assert_eq!(tokens[3].token, Token::Less);
    assert_eq!(tokens[4].token, Token::GreaterOrEqual);
    assert_eq!(tokens[5].token, Token::LessOrEqual);
}

#[test]
fn test_tokenize_keywords() {
    let mut lexer = Lexer::new("if else elseif function return");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::If);
    assert_eq!(tokens[1].token, Token::Else);
    assert_eq!(tokens[2].token, Token::ElseIf);
    assert_eq!(tokens[3].token, Token::Function);
    assert_eq!(tokens[4].token, Token::Return);
}

#[test]
fn test_tokenize_boolean_literals() {
    let mut lexer = Lexer::new("true false");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Boolean(true));
    assert_eq!(tokens[1].token, Token::Boolean(false));
}

#[test]
fn test_tokenize_braces_and_parens() {
    let mut lexer = Lexer::new("{ } ( ) [ ]");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::LeftBrace);
    assert_eq!(tokens[1].token, Token::RightBrace);
    assert_eq!(tokens[2].token, Token::LeftParen);
    assert_eq!(tokens[3].token, Token::RightParen);
    assert_eq!(tokens[4].token, Token::LeftBracket);
    assert_eq!(tokens[5].token, Token::RightBracket);
}

#[test]
fn test_tokenize_comment() {
    let mut lexer = Lexer::new("$x = 5 # this is a comment");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // $x, =, 5, Eof (comment is skipped)
    assert_eq!(tokens[0].token, Token::Variable("x".to_string()));
    assert_eq!(tokens[1].token, Token::Assignment);
    assert_eq!(tokens[2].token, Token::Number(5.0));
}

#[test]
fn test_tokenize_newline() {
    let mut lexer = Lexer::new("$x = 5\n$y = 10");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[3].token, Token::Newline);
}

#[test]
fn test_tokenize_semicolon() {
    let mut lexer = Lexer::new("$x = 5; $y = 10");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[3].token, Token::Semicolon);
}

#[test]
fn test_tokenize_complex_expression() {
    let mut lexer = Lexer::new("if ($x -eq 5) { Write-Output \"Five\" }");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::If);
    assert_eq!(tokens[1].token, Token::LeftParen);
    assert_eq!(tokens[2].token, Token::Variable("x".to_string()));
    assert_eq!(tokens[3].token, Token::Equal);
    assert_eq!(tokens[4].token, Token::Number(5.0));
    assert_eq!(tokens[5].token, Token::RightParen);
    assert_eq!(tokens[6].token, Token::LeftBrace);
    assert_eq!(
        tokens[7].token,
        Token::Identifier("Write-Output".to_string())
    );
    assert_eq!(tokens[8].token, Token::String("Five".to_string()));
    assert_eq!(tokens[9].token, Token::RightBrace);
}

#[test]
fn test_tokenize_pipeline_with_scriptblock() {
    let mut lexer = Lexer::new("Get-Process | Where-Object { $_.CPU -gt 10 }");
    let tokens = lexer.tokenize().unwrap();
    // Verify key tokens
    assert_eq!(
        tokens[0].token,
        Token::Identifier("Get-Process".to_string())
    );
    assert_eq!(tokens[1].token, Token::Pipeline);
    assert_eq!(
        tokens[2].token,
        Token::Identifier("Where-Object".to_string())
    );
    assert_eq!(tokens[3].token, Token::LeftBrace);
}

#[test]
fn test_unterminated_string() {
    let mut lexer = Lexer::new("\"hello");
    let result = lexer.tokenize();
    assert!(result.is_err());
    match result {
        Err(LexError::UnterminatedString { .. }) => (),
        _ => panic!("Expected UnterminatedString error"),
    }
}

#[test]
fn test_invalid_variable() {
    let mut lexer = Lexer::new("$ ");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_position_tracking() {
    let mut lexer = Lexer::new("$x = 5\n$y = 10");
    let tokens = lexer.tokenize().unwrap();

    // First line tokens
    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[0].position.column, 1);

    // After newline
    assert_eq!(tokens[4].position.line, 2);
}

#[test]
fn test_escape_sequences() {
    let mut lexer = Lexer::new("\"hello\\nworld\"");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::String("hello\nworld".to_string()));
}

#[test]
fn test_modulo_operator() {
    let mut lexer = Lexer::new("10 % 3");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1].token, Token::Modulo);
}

#[test]
fn test_dot_operator() {
    let mut lexer = Lexer::new("$obj.Property");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Variable("obj".to_string()));
    assert_eq!(tokens[1].token, Token::Dot);
    assert_eq!(tokens[2].token, Token::Identifier("Property".to_string()));
}

#[test]
fn test_comma_separator() {
    let mut lexer = Lexer::new("$a, $b, $c");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Variable("a".to_string()));
    assert_eq!(tokens[1].token, Token::Comma);
    assert_eq!(tokens[2].token, Token::Variable("b".to_string()));
    assert_eq!(tokens[3].token, Token::Comma);
    assert_eq!(tokens[4].token, Token::Variable("c".to_string()));
}

#[test]
fn test_string_interpolation_simple() {
    let mut lexer = Lexer::new("\"Hello $name\"");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // InterpolatedString + Eof
    
    match &tokens[0].token {
        Token::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(parts[1], StringPart::Variable("name".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_interpolation_multiple() {
    let mut lexer = Lexer::new("\"Hello $first $last!\"");
    let tokens = lexer.tokenize().unwrap();
    
    match &tokens[0].token {
        Token::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 5);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(parts[1], StringPart::Variable("first".to_string()));
            assert_eq!(parts[2], StringPart::Literal(" ".to_string()));
            assert_eq!(parts[3], StringPart::Variable("last".to_string()));
            assert_eq!(parts[4], StringPart::Literal("!".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_interpolation_at_start() {
    let mut lexer = Lexer::new("\"$name says hello\"");
    let tokens = lexer.tokenize().unwrap();
    
    match &tokens[0].token {
        Token::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Variable("name".to_string()));
            assert_eq!(parts[1], StringPart::Literal(" says hello".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_interpolation_at_end() {
    let mut lexer = Lexer::new("\"Hello $name\"");
    let tokens = lexer.tokenize().unwrap();
    
    match &tokens[0].token {
        Token::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(parts[1], StringPart::Variable("name".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_string_no_interpolation_single_quotes() {
    let mut lexer = Lexer::new("'Hello $name'");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::String("Hello $name".to_string()));
}

#[test]
fn test_escaped_dollar_sign() {
    let mut lexer = Lexer::new("\"Price: \\$100\"");
    let tokens = lexer.tokenize().unwrap();
    // Should be a simple string since $ is escaped
    assert_eq!(tokens[0].token, Token::String("Price: $100".to_string()));
}

#[test]
fn test_empty_interpolated_string() {
    let mut lexer = Lexer::new("\"\"");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::String(String::new()));
}
