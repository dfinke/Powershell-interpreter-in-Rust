use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;

#[test]
fn test_select_object_bare_identifiers() {
    // This reproduces the issue: Name and CPU should be treated as strings, not cmdlets
    let input = r#"
        $processes = @(
            @{Name="chrome"; CPU=45.2; Id=5678; WorkingSet=512000}
            @{Name="code"; CPU=23.1; Id=9012; WorkingSet=256000}
        )
        $processes | Select-Object Name, CPU
    "#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    
    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);
    
    let result = evaluator.eval(program);
    println!("Result: {:?}", result);
    
    assert!(result.is_ok(), "Should not fail with 'Name' cmdlet error");
}
