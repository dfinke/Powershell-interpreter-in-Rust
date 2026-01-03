/// Manual verification test for the Select-Object fix
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::{Evaluator, Value};

#[test]
fn verify_select_object_issue_fix() {
    // This is the exact code from the issue
    let code = r#"
        $processes = @(
            @{Name="chrome"; CPU=45.2; Id=5678; WorkingSet=512000}
            @{Name="code"; CPU=23.1; Id=9012; WorkingSet=256000}
            @{Name="pwsh"; CPU=5.0; Id=3456; WorkingSet=51200}
            @{Name="explorer"; CPU=15.5; Id=1234; WorkingSet=102400}
            @{Name="System"; CPU=0.0; Id=4; WorkingSet=1024}
        )
        
        $processes | Select-Object Name, CPU
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parser should succeed");
    
    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);
    
    let result = evaluator.eval(program).expect("Should execute without 'Name' cmdlet error");
    
    // Verify the result is correct
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 5, "Should have 5 process objects");
        
        // Check each object only has Name and CPU properties
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 2, "Each object should have exactly 2 properties");
                assert!(props.contains_key("Name"), "Should have Name property");
                assert!(props.contains_key("CPU"), "Should have CPU property");
                assert!(!props.contains_key("Id"), "Should NOT have Id property");
                assert!(!props.contains_key("WorkingSet"), "Should NOT have WorkingSet property");
            } else {
                panic!("Expected Object value in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}
