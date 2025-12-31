use pwsh_lexer::Lexer;
use pwsh_parser::*;

// Helper function to parse a string
fn parse_str(input: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_parse_number_literal() {
    let program = parse_str("42").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Number(n))) => {
            assert_eq!(*n, 42.0);
        }
        _ => panic!("Expected number literal"),
    }
}

#[test]
fn test_parse_string_literal() {
    let program = parse_str("\"hello\"").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "hello");
        }
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_parse_boolean_literal() {
    let program = parse_str("true").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::Literal(Literal::Boolean(b))) => {
            assert_eq!(*b, true);
        }
        _ => panic!("Expected boolean literal"),
    }
}

#[test]
fn test_parse_variable() {
    let program = parse_str("$x").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::Variable(name)) => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected variable"),
    }
}

#[test]
fn test_parse_assignment() {
    let program = parse_str("$x = 5").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Assignment { variable, value } => {
            assert_eq!(variable, "x");
            match value {
                Expression::Literal(Literal::Number(n)) => assert_eq!(*n, 5.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_parse_binary_addition() {
    let program = parse_str("5 + 3").unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            left,
            operator,
            right,
        }) => {
            assert_eq!(*operator, BinaryOperator::Add);
            match **left {
                Expression::Literal(Literal::Number(n)) => assert_eq!(n, 5.0),
                _ => panic!("Expected number literal"),
            }
            match **right {
                Expression::Literal(Literal::Number(n)) => assert_eq!(n, 3.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_binary_multiplication() {
    let program = parse_str("10 * 2").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            operator,
            ..
        }) => {
            assert_eq!(*operator, BinaryOperator::Multiply);
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_comparison_equal() {
    let program = parse_str("$x -eq 5").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            operator,
            ..
        }) => {
            assert_eq!(*operator, BinaryOperator::Equal);
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_comparison_greater() {
    let program = parse_str("$x -gt 5").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            operator,
            ..
        }) => {
            assert_eq!(*operator, BinaryOperator::Greater);
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_operator_precedence() {
    // 10 + 20 * 2 should parse as 10 + (20 * 2)
    let program = parse_str("10 + 20 * 2").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            left,
            operator,
            right,
        }) => {
            assert_eq!(*operator, BinaryOperator::Add);

            // Left should be 10
            match **left {
                Expression::Literal(Literal::Number(n)) => assert_eq!(n, 10.0),
                _ => panic!("Expected number literal"),
            }

            // Right should be 20 * 2
            match &**right {
                Expression::BinaryOp {
                    operator: op,
                    ..
                } => {
                    assert_eq!(*op, BinaryOperator::Multiply);
                }
                _ => panic!("Expected multiplication"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_parenthesized_expression() {
    let program = parse_str("(10 + 20) * 2").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::BinaryOp {
            left,
            operator,
            right,
        }) => {
            assert_eq!(*operator, BinaryOperator::Multiply);

            // Left should be (10 + 20)
            match &**left {
                Expression::BinaryOp {
                    operator: op,
                    ..
                } => {
                    assert_eq!(*op, BinaryOperator::Add);
                }
                _ => panic!("Expected addition"),
            }

            // Right should be 2
            match **right {
                Expression::Literal(Literal::Number(n)) => assert_eq!(n, 2.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected binary operation"),
    }
}

#[test]
fn test_parse_unary_minus() {
    let program = parse_str("-5").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::UnaryOp { operator, operand }) => {
            assert_eq!(*operator, UnaryOperator::Negate);
            match **operand {
                Expression::Literal(Literal::Number(n)) => assert_eq!(n, 5.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected unary operation"),
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let program = parse_str("Get-Process").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::Call { name, arguments }) => {
            assert_eq!(name, "Get-Process");
            assert_eq!(arguments.len(), 0);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let program = parse_str("Write-Output \"hello\"").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::Call { name, arguments }) => {
            assert_eq!(name, "Write-Output");
            assert_eq!(arguments.len(), 1);

            match &arguments[0] {
                Argument::Positional(Expression::Literal(Literal::String(s))) => {
                    assert_eq!(s, "hello");
                }
                _ => panic!("Expected positional string argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_multiple_args() {
    let program = parse_str("Add 5 10").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::Call { name, arguments }) => {
            assert_eq!(name, "Add");
            assert_eq!(arguments.len(), 2);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_member_access() {
    let program = parse_str("$obj.Property").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::MemberAccess { object, member }) => {
            match **object {
                Expression::Variable(ref name) => assert_eq!(name, "obj"),
                _ => panic!("Expected variable"),
            }
            assert_eq!(member, "Property");
        }
        _ => panic!("Expected member access"),
    }
}

#[test]
fn test_parse_script_block() {
    let program = parse_str("{ $x = 5 }").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::ScriptBlock(block)) => {
            assert_eq!(block.statements.len(), 1);
        }
        _ => panic!("Expected script block"),
    }
}

#[test]
fn test_parse_if_statement() {
    let program = parse_str("if ($x -eq 5) { Write-Output \"Five\" }").unwrap();

    match &program.statements[0] {
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // Check condition is a comparison
            match condition {
                Expression::BinaryOp { operator, .. } => {
                    assert_eq!(*operator, BinaryOperator::Equal);
                }
                _ => panic!("Expected binary operation"),
            }

            // Check then branch has statements
            assert!(!then_branch.statements.is_empty());

            // Check no else branch
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_else_statement() {
    let program = parse_str("if ($x -eq 5) { $y = 1 } else { $y = 2 }").unwrap();

    match &program.statements[0] {
        Statement::If {
            else_branch,
            ..
        } => {
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_function_def() {
    let program = parse_str("function Test { Write-Output \"Hello\" }").unwrap();

    match &program.statements[0] {
        Statement::FunctionDef {
            name,
            parameters,
            body,
        } => {
            assert_eq!(name, "Test");
            assert_eq!(parameters.len(), 0);
            assert!(!body.statements.is_empty());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_def_with_params() {
    let program = parse_str("function Add($a, $b) { $a + $b }").unwrap();

    match &program.statements[0] {
        Statement::FunctionDef {
            name,
            parameters,
            body,
        } => {
            assert_eq!(name, "Add");
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "a");
            assert_eq!(parameters[1].name, "b");
            assert!(!body.statements.is_empty());
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_return_statement() {
    let program = parse_str("return 42").unwrap();

    match &program.statements[0] {
        Statement::Return(Some(Expression::Literal(Literal::Number(n)))) => {
            assert_eq!(*n, 42.0);
        }
        _ => panic!("Expected return statement"),
    }
}

#[test]
fn test_parse_return_statement_no_value() {
    let program = parse_str("return").unwrap();

    match &program.statements[0] {
        Statement::Return(None) => {}
        _ => panic!("Expected return statement with no value"),
    }
}

#[test]
fn test_parse_pipeline_two_stages() {
    let program = parse_str("Get-Process | Where-Object").unwrap();

    match &program.statements[0] {
        Statement::Pipeline(pipeline) => {
            assert_eq!(pipeline.stages.len(), 2);

            match &pipeline.stages[0] {
                Expression::Call { name, .. } => {
                    assert_eq!(name, "Get-Process");
                }
                _ => panic!("Expected function call"),
            }

            match &pipeline.stages[1] {
                Expression::Call { name, .. } => {
                    assert_eq!(name, "Where-Object");
                }
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected pipeline"),
    }
}

#[test]
fn test_parse_pipeline_with_scriptblock() {
    let program = parse_str("Get-Process | Where-Object { $_.CPU -gt 10 }").unwrap();

    match &program.statements[0] {
        Statement::Pipeline(pipeline) => {
            assert_eq!(pipeline.stages.len(), 2);

            match &pipeline.stages[1] {
                Expression::Call { name, arguments } => {
                    assert_eq!(name, "Where-Object");
                    assert_eq!(arguments.len(), 1);

                    match &arguments[0] {
                        Argument::Positional(Expression::ScriptBlock(_)) => {}
                        _ => panic!("Expected script block argument"),
                    }
                }
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected pipeline"),
    }
}

#[test]
fn test_parse_multiple_statements() {
    let program = parse_str("$x = 5\n$y = 10\n$z = $x + $y").unwrap();
    assert_eq!(program.statements.len(), 3);

    // Verify all are assignments
    for stmt in &program.statements {
        match stmt {
            Statement::Assignment { .. } => {}
            _ => panic!("Expected assignment"),
        }
    }
}

#[test]
fn test_parse_multiple_statements_with_semicolon() {
    let program = parse_str("$x = 5; $y = 10; $z = $x + $y").unwrap();
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn test_parse_complex_expression() {
    let program = parse_str("$x = ($a + $b) * 2 - $c / 3").unwrap();

    match &program.statements[0] {
        Statement::Assignment { variable, value } => {
            assert_eq!(variable, "x");
            // Just verify it parses without error
            match value {
                Expression::BinaryOp { .. } => {}
                _ => panic!("Expected binary operation"),
            }
        }
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_parse_interpolated_string() {
    let program = parse_str("\"Hello $name\"").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::Literal(Literal::InterpolatedString(parts))) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(parts[1], StringPart::Variable("name".to_string()));
        }
        _ => panic!("Expected interpolated string"),
    }
}

#[test]
fn test_parse_empty_program() {
    let program = parse_str("").unwrap();
    assert_eq!(program.statements.len(), 0);
}

#[test]
fn test_parse_only_newlines() {
    let program = parse_str("\n\n\n").unwrap();
    assert_eq!(program.statements.len(), 0);
}

#[test]
fn test_parse_comments_ignored() {
    let program = parse_str("$x = 5 # this is a comment").unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_parse_cmdlet_with_named_param() {
    let program = parse_str("Select-Object -First 5").unwrap();

    match &program.statements[0] {
        Statement::Expression(Expression::Call { name, arguments }) => {
            assert_eq!(name, "Select-Object");
            assert_eq!(arguments.len(), 1);

            match &arguments[0] {
                Argument::Named { name, value } => {
                    assert_eq!(name, "First");
                    match value {
                        Expression::Literal(Literal::Number(n)) => assert_eq!(*n, 5.0),
                        _ => panic!("Expected number"),
                    }
                }
                _ => panic!("Expected named argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_complex_pipeline() {
    let program = parse_str(
        "Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU"
    ).unwrap();

    match &program.statements[0] {
        Statement::Pipeline(pipeline) => {
            assert_eq!(pipeline.stages.len(), 3);
        }
        _ => panic!("Expected pipeline"),
    }
}

#[test]
fn test_parse_nested_blocks() {
    let program = parse_str("if ($x -eq 5) { if ($y -eq 10) { $z = 1 } }").unwrap();

    match &program.statements[0] {
        Statement::If { then_branch, .. } => {
            assert_eq!(then_branch.statements.len(), 1);

            match &then_branch.statements[0] {
                Statement::If { .. } => {}
                _ => panic!("Expected nested if statement"),
            }
        }
        _ => panic!("Expected if statement"),
    }
}
