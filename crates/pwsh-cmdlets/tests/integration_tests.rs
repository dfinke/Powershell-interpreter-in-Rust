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

// Week 7: Function Definitions - Integration Tests

#[test]
fn test_function_with_write_output() {
    let result = eval_with_cmdlets("function Test() { Write-Output 42 }\nTest").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_function_calling_cmdlet_with_parameter() {
    let result = eval_with_cmdlets(
        "function Greet($name) { Write-Output \"Hello $name\" }\nGreet \"Alice\"",
    )
    .unwrap();
    assert_eq!(result, Value::String("Hello Alice".to_string()));
}

#[test]
fn test_week7_success_criteria_simple() {
    // From ROADMAP Week 7: function Add($a, $b) { return $a + $b }; $result = Add 5 10
    let result =
        eval_with_cmdlets("function Add($a, $b) { return $a + $b }\n$result = Add 5 10\n$result")
            .unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_week7_success_criteria_with_cmdlet() {
    // Function that calls cmdlets
    let result = eval_with_cmdlets(
        "function Double($x) { $result = $x * 2\nWrite-Output $result }\nDouble 21",
    )
    .unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_function_with_default_and_cmdlet() {
    let result = eval_with_cmdlets(
        "function Greet($name = \"World\") { Write-Output \"Hello $name\" }\nGreet",
    )
    .unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_case_insensitive_function_calls() {
    // Test that function names are case-insensitive
    let result = eval_with_cmdlets("function Add($a, $b) { return $a + $b }\nadd 5 10").unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_case_insensitive_cmdlet_calls() {
    // Test that cmdlet names are case-insensitive
    let result = eval_with_cmdlets("write-output 42").unwrap();
    assert_eq!(result, Value::Number(42.0));

    let result2 = eval_with_cmdlets("WRITE-OUTPUT 42").unwrap();
    assert_eq!(result2, Value::Number(42.0));
}

#[test]
fn test_case_insensitive_string_comparison() {
    // Test that string comparisons are case-insensitive by default
    let result = eval_with_cmdlets("\"hello\" -eq \"HELLO\"").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result2 = eval_with_cmdlets("\"PowerShell\" -eq \"powershell\"").unwrap();
    assert_eq!(result2, Value::Boolean(true));
}

#[test]
fn test_case_insensitive_comprehensive() {
    // Comprehensive test combining all case-insensitive features
    let code = r#"
        function Multiply($x, $y) { return $x * $y }
        $MyVar = 10
        $result = MULTIPLY $myvar 2
        if ("TEST" -eq "test") {
            WRITE-OUTPUT $RESULT
        }
    "#;
    let result = eval_with_cmdlets(code).unwrap();
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn test_week12_select_object_properties() {
    // Test selecting specific properties from objects
    let code = r#"
        $procs = @(
            @{Name="chrome"; CPU=45.2; Id=5678}
            @{Name="code"; CPU=23.1; Id=9012}
        )
        $procs | Select-Object -Property @("Name", "CPU")
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    // Result should be an array
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);

        // Each item should be an object with only Name and CPU
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 2);
                assert!(props.contains_key("Name"));
                assert!(props.contains_key("CPU"));
                assert!(!props.contains_key("Id"));
            } else {
                panic!("Expected object in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week12_select_object_first() {
    // Test selecting first N items
    let code = r#"
        $procs = @(
            @{Name="chrome"; CPU=45.2}
            @{Name="code"; CPU=23.1}
            @{Name="pwsh"; CPU=5.0}
        )
        $procs | Select-Object -First 2
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week12_select_object_last() {
    // Test selecting last N items
    let code = r#"
        $nums = @(1, 2, 3, 4, 5)
        $nums | Select-Object -Last 2
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(4.0));
        assert_eq!(items[1], Value::Number(5.0));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week12_combined_property_and_first() {
    // Test combining property selection with -First
    let code = r#"
        $procs = @(
            @{Name="chrome"; CPU=45.2; Id=5678}
            @{Name="code"; CPU=23.1; Id=9012}
            @{Name="pwsh"; CPU=5.0; Id=3456}
        )
        $procs | Select-Object -Property @("Name", "CPU") -First 2
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);

        // Each item should have only Name and CPU
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 2);
                assert!(props.contains_key("Name"));
                assert!(props.contains_key("CPU"));
                assert!(!props.contains_key("Id"));
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_select_object_positional_arguments() {
    // Test using bare identifiers as positional arguments (issue fix)
    // This is the syntax from the problem statement: Select-Object Name, CPU
    let code = r#"
        $processes = @(
            @{Name="chrome"; CPU=45.2; Id=5678; WorkingSet=512000}
            @{Name="code"; CPU=23.1; Id=9012; WorkingSet=256000}
            @{Name="pwsh"; CPU=5.0; Id=3456; WorkingSet=51200}
        )
        $processes | Select-Object Name, CPU
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    // Result should be an array
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 3);

        // Each item should be an object with only Name and CPU
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 2);
                assert!(props.contains_key("Name"));
                assert!(props.contains_key("CPU"));
                assert!(!props.contains_key("Id"));
                assert!(!props.contains_key("WorkingSet"));
            } else {
                panic!("Expected object in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week15_get_childitem_basic() {
    // Test that Get-ChildItem returns an array of file objects
    let result = eval_with_cmdlets("Get-ChildItem").unwrap();

    // When a cmdlet returns an array and there's no pipeline,
    // the evaluator returns the last element of the array
    // So we need to check if it's an Object with Name property
    if let Value::Object(props) = result {
        assert!(
            props.contains_key("Name"),
            "File object should have a Name property"
        );
    } else {
        panic!("Expected object result from Get-ChildItem, got {:?}", result);
    }
}

#[test]
fn test_week15_get_childitem_with_select() {
    // Test Get-ChildItem piped to Select-Object
    let result = eval_with_cmdlets("Get-ChildItem | Select-Object Name").unwrap();

    // Result should be an array
    if let Value::Array(items) = result {
        assert!(items.len() > 0, "Should return at least some items");

        // Each item should only have Name property
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 1, "Should only have Name property");
                assert!(props.contains_key("Name"));
            } else {
                panic!("Expected object in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}
