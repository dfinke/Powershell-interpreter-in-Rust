/// Final verification test for the exact issue from the problem statement
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::{Evaluator, Value};

#[test]
fn test_exact_issue_scenario() {
    // This is the EXACT code from the GitHub issue
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
    let tokens = lexer.tokenize().expect("Should tokenize successfully");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Should parse successfully");
    
    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);
    
    // This should NOT fail with "The term 'Name' is not recognized as a cmdlet"
    let result = evaluator.eval(program)
        .expect("Should execute successfully without 'Name' cmdlet error");
    
    // Verify the result structure
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 5, "Should return all 5 process objects");
        
        // Verify first object (chrome)
        if let Value::Object(props) = &items[0] {
            assert_eq!(props.len(), 2, "Should have exactly 2 properties");
            assert_eq!(props.get("Name"), Some(&Value::String("chrome".to_string())));
            assert_eq!(props.get("CPU"), Some(&Value::Number(45.2)));
            assert!(props.get("Id").is_none(), "Should not have Id");
            assert!(props.get("WorkingSet").is_none(), "Should not have WorkingSet");
        } else {
            panic!("First item should be an object");
        }
        
        // Verify second object (code)
        if let Value::Object(props) = &items[1] {
            assert_eq!(props.get("Name"), Some(&Value::String("code".to_string())));
            assert_eq!(props.get("CPU"), Some(&Value::Number(23.1)));
        }
        
        // Verify all objects only have Name and CPU
        for (i, item) in items.iter().enumerate() {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 2, "Item {} should have 2 properties", i);
                assert!(props.contains_key("Name"), "Item {} should have Name", i);
                assert!(props.contains_key("CPU"), "Item {} should have CPU", i);
            }
        }
    } else {
        panic!("Expected array result, got: {:?}", result);
    }
}
