/// Integration tests for Week 6: Object Pipeline with 5 Cmdlets
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::{Evaluator, Value};

/// Helper function to evaluate PowerShell code with cmdlets
fn eval_with_cmdlets(input: &str) -> Result<Value, pwsh_runtime::RuntimeError> {
    // Create evaluator with cmdlets registered
    let mut evaluator = Evaluator::new();
    pwsh_cmdlets::register_all(evaluator.registry_mut());

    // Parse and execute
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    evaluator.eval(program)
}

#[test]
fn test_write_output_simple() {
    let result = eval_with_cmdlets("Write-Output 42").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_write_output_string() {
    let result = eval_with_cmdlets("Write-Output \"Hello World\"").unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_get_process_basic() {
    let result = eval_with_cmdlets("Get-Process").unwrap();
    // Get-Process returns an array, but statement returns the last value
    // which should be a process object (we'd need to inspect this differently)
    // For now, just check it doesn't error
    assert_ne!(result, Value::Null);
}

#[test]
fn test_pipeline_write_output() {
    // Test that pipeline works correctly
    let result = eval_with_cmdlets("5 | Write-Output").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_simple_value_through_pipeline() {
    // Simple value -> Write-Output
    let result = eval_with_cmdlets("42 | Write-Output").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_variable_through_pipeline() {
    let result = eval_with_cmdlets("$x = 10\n$x | Write-Output").unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_cmdlet_not_found() {
    let result = eval_with_cmdlets("NonExistent-Cmdlet");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("not recognized"));
    }
}

#[test]
fn test_week6_success_criteria_basic() {
    // From ROADMAP: Write-Output "Hello World"
    let result = eval_with_cmdlets("Write-Output \"Hello World\"").unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_week6_success_criteria_variable() {
    // From ROADMAP: $x = 5; Write-Output $x
    let result = eval_with_cmdlets("$x = 5\nWrite-Output $x").unwrap();
    assert_eq!(result, Value::Number(5.0));
}
