/// Manual verification test for the Select-Object fix
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::{Evaluator, Value};

#[test]
fn test_case_insensitive_property_selection() {
    // Test for case-insensitive property lookup
    let code = r#"
        $process = @{CPU=45.2; Name="pwsh"; Id=3456}
        $process | Select-Object CPu
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);

    let result = evaluator.eval(program).unwrap();

    // Verify the property was found despite different case
    if let Value::Object(props) = result {
        assert_eq!(props.len(), 1, "Should have exactly 1 property");
        // The property should be found (case-insensitive)
        assert!(props.contains_key("CPu"), "Should have CPu property");
        assert_eq!(props.get("CPu"), Some(&Value::Number(45.2)));
    } else {
        panic!("Expected object result, got: {:?}", result);
    }
}

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

    let result = evaluator
        .eval(program)
        .expect("Should execute without 'Name' cmdlet error");

    // Verify the result is correct
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 5, "Should have 5 process objects");

        // Check each object only has Name and CPU properties
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(
                    props.len(),
                    2,
                    "Each object should have exactly 2 properties"
                );
                assert!(props.contains_key("Name"), "Should have Name property");
                assert!(props.contains_key("CPU"), "Should have CPU property");
                assert!(!props.contains_key("Id"), "Should NOT have Id property");
                assert!(
                    !props.contains_key("WorkingSet"),
                    "Should NOT have WorkingSet property"
                );
            } else {
                panic!("Expected Object value in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_select_object_single_property_bare_word() {
    // Test with a single property as bare word
    let code = r#"
        $items = @(
            @{Name="A"; Value=1}
            @{Name="B"; Value=2}
        )
        $items | Select-Object Name
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);

    let result = evaluator.eval(program).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 1);
                assert!(props.contains_key("Name"));
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_select_object_three_properties() {
    // Test with three properties
    let code = r#"
        $items = @(
            @{A=1; B=2; C=3; D=4}
        )
        $items | Select-Object A, B, C
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);

    let result = evaluator.eval(program).unwrap();

    // When pipeline returns a single item, it's not wrapped in an array
    if let Value::Object(props) = result {
        assert_eq!(props.len(), 3);
        assert!(props.contains_key("A"));
        assert!(props.contains_key("B"));
        assert!(props.contains_key("C"));
        assert!(!props.contains_key("D"));
    } else {
        panic!("Expected object result, got: {:?}", result);
    }
}

#[test]
fn test_bare_words_with_first_parameter() {
    // Test combining bare words with -First parameter
    let code = r#"
        $items = @(
            @{Name="A"; Value=1}
            @{Name="B"; Value=2}
            @{Name="C"; Value=3}
        )
        $items | Select-Object Name -First 2
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let mut registry = pwsh_runtime::CmdletRegistry::new();
    pwsh_cmdlets::register_all(&mut registry);
    let mut evaluator = Evaluator::with_registry(registry);

    let result = evaluator.eval(program).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2, "Should only return first 2 items");
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 1);
                assert!(props.contains_key("Name"));
                assert!(!props.contains_key("Value"));
            }
        }
    } else {
        panic!("Expected array result");
    }
}
