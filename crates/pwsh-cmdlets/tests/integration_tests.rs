/// Integration tests for Week 6: Object Pipeline with 5 Cmdlets
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::{Evaluator, Value};
use std::fs;
use tempfile::TempDir;

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
fn test_select_object_case_insensitive_property_lookup() {
    // Test that Select-Object finds properties regardless of case
    let code = r#"
        $obj = @{Name="Test"; CPU=10}
        $obj | Select-Object name, cpu
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Object(props) = result {
        assert_eq!(props.len(), 2);
        // The keys in the result object should match the requested case
        assert!(props.contains_key("name"));
        assert!(props.contains_key("cpu"));
        assert_eq!(props.get("name"), Some(&Value::String("Test".to_string())));
        assert_eq!(props.get("cpu"), Some(&Value::Number(10.0)));
    } else {
        panic!("Expected object result, got {:?}", result);
    }
}

#[test]
fn test_select_object_mixed_case_input_and_request() {
    // Test with mixed case in both input and request
    let code = r#"
        $obj = @{nAmE="Mixed"; cPu=20}
        $obj | Select-Object Name, CPU
    "#;
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Object(props) = result {
        assert_eq!(props.len(), 2);
        assert!(props.contains_key("Name"));
        assert!(props.contains_key("CPU"));
        assert_eq!(props.get("Name"), Some(&Value::String("Mixed".to_string())));
        assert_eq!(props.get("CPU"), Some(&Value::Number(20.0)));
    } else {
        panic!("Expected object result, got {:?}", result);
    }
}

#[test]
fn test_select_object_get_process_case_insensitive() {
    let code = "Get-Process | Select-Object name";
    let result = eval_with_cmdlets(code).unwrap();

    if let Value::Array(items) = result {
        assert!(!items.is_empty());
        for item in items {
            if let Value::Object(props) = item {
                assert!(
                    props.contains_key("Name") || props.contains_key("name"),
                    "Should contain name property. Props: {:?}",
                    props
                );
            } else {
                panic!("Expected object");
            }
        }
    } else if let Value::Object(props) = result {
        assert!(
            props.contains_key("Name") || props.contains_key("name"),
            "Should contain name property. Props: {:?}",
            props
        );
    } else {
        panic!("Expected array or object, got {:?}", result);
    }
}

#[test]
fn test_week15_get_childitem_basic() {
    // Test that Get-ChildItem returns an array of file objects
    let result = eval_with_cmdlets("Get-ChildItem").unwrap();

    if let Value::Array(items) = result {
        assert!(!items.is_empty(), "Expected at least one item");
        for item in items {
            if let Value::Object(props) = item {
                assert!(
                    props.contains_key("Name") || props.contains_key("name"),
                    "File object should have a Name property. Props: {:?}",
                    props
                );
            } else {
                panic!("Expected object item, got {:?}", item);
            }
        }
    } else {
        panic!("Expected array result from Get-ChildItem, got {:?}", result);
    }
}

#[test]
fn test_week15_get_childitem_with_select() {
    // Test Get-ChildItem piped to Select-Object
    let result = eval_with_cmdlets("Get-ChildItem | Select-Object Name").unwrap();

    // Result should be an array
    if let Value::Array(items) = result {
        assert!(!items.is_empty(), "Should return at least some items");

        // Each item should only have Name property
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 1, "Should only have Name property");
                assert!(
                    props.contains_key("Name") || props.contains_key("name"),
                    "Should contain Name property. Props: {:?}",
                    props
                );
            } else {
                panic!("Expected object in array");
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week15_get_childitem_path_parameter() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("a.txt"), "a").unwrap();
    fs::write(temp_path.join("b.rs"), "b").unwrap();

    // NOTE: The lexer treats backslashes as escape sequences, so normalize to forward slashes.
    let path_str = temp_path.to_string_lossy().replace('\\', "/");
    let code = format!("Get-ChildItem -Path '{}' | Select-Object Name", path_str);
    let result = eval_with_cmdlets(&code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);
        for item in items {
            if let Value::Object(props) = item {
                assert_eq!(props.len(), 1);
                assert!(props.contains_key("Name") || props.contains_key("name"));
            } else {
                panic!("Expected object");
            }
        }
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_week15_get_childitem_filter_include_exclude_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("file1.txt"), "one").unwrap();
    fs::write(temp_path.join("file2.rs"), "two").unwrap();
    fs::write(temp_path.join("README.md"), "readme").unwrap();

    // Normalize to forward slashes due to lexer escape handling.
    let path_str = temp_path.to_string_lossy().replace('\\', "/");

    // -Filter
    let code = format!(
        "Get-ChildItem -Path '{}' -Filter '*.rs' | Select-Object Name",
        path_str
    );
    let result = eval_with_cmdlets(&code).unwrap();
    let items: Vec<Value> = match result {
        Value::Array(items) => items,
        other => vec![other],
    };
    assert_eq!(items.len(), 1);
    if let Value::Object(props) = &items[0] {
        assert_eq!(
            props.get("Name"),
            Some(&Value::String("file2.rs".to_string()))
        );
    } else {
        panic!("Expected object");
    }

    // -Include/-Exclude using @() array literal
    let code = format!(
        "Get-ChildItem -Path '{}' -Include @('*.md','*.txt') -Exclude 'README*' | Select-Object Name",
        path_str
    );
    let result = eval_with_cmdlets(&code).unwrap();
    let items: Vec<Value> = match result {
        Value::Array(items) => items,
        other => vec![other],
    };
    assert_eq!(items.len(), 1);
    if let Value::Object(props) = &items[0] {
        assert_eq!(
            props.get("Name"),
            Some(&Value::String("file1.txt".to_string()))
        );
    } else {
        panic!("Expected object");
    }
}

// Week 16: Get-Content - Integration Tests

#[test]
fn test_week16_get_content_reads_text_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("sample.txt");
    fs::write(&file_path, "one\ntwo\nthree\n").unwrap();

    // NOTE: The lexer treats backslashes as escape sequences, so normalize to forward slashes.
    let path_str = file_path.to_string_lossy().replace('\\', "/");
    let code = format!("Get-Content -Path '{}'", path_str);
    let result = eval_with_cmdlets(&code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(
            items,
            vec![
                Value::String("one".to_string()),
                Value::String("two".to_string()),
                Value::String("three".to_string())
            ]
        );
    } else {
        panic!("Expected array result from Get-Content, got {:?}", result);
    }
}

#[test]
fn test_week16_get_content_reads_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.txt");
    fs::write(&file_path, "").unwrap();

    let path_str = file_path.to_string_lossy().replace('\\', "/");
    let code = format!("Get-Content '{}'", path_str);
    let result = eval_with_cmdlets(&code).unwrap();

    match result {
        Value::Array(items) => assert!(items.is_empty()),
        Value::Null => {
            // Current evaluator behavior: when a cmdlet produces no output values,
            // the statement evaluates to $null.
        }
        other => panic!(
            "Expected array or Null result from Get-Content, got {:?}",
            other
        ),
    }
}

#[test]
fn test_week16_get_content_nonexistent_file_errors() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("does_not_exist.txt");

    let path_str = file_path.to_string_lossy().replace('\\', "/");
    let code = format!("Get-Content '{}'", path_str);
    let result = eval_with_cmdlets(&code);

    assert!(result.is_err());
    let msg = result.err().unwrap().to_string();
    assert!(msg.contains("Failed to open file") || msg.contains("Failed to read file"));
}

#[test]
fn test_week16_get_content_with_encoding_unicode_utf16le() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("utf16.txt");

    // Write UTF-16LE with BOM.
    let s = "one\ntwo\n";
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
    for u in s.encode_utf16() {
        bytes.push((u & 0x00FF) as u8);
        bytes.push((u >> 8) as u8);
    }
    fs::write(&file_path, bytes).unwrap();

    // NOTE: The lexer treats backslashes as escape sequences, so normalize to forward slashes.
    let path_str = file_path.to_string_lossy().replace('\\', "/");
    let code = format!("Get-Content -Path '{}' -Encoding 'Unicode'", path_str);
    let result = eval_with_cmdlets(&code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(
            items,
            vec![
                Value::String("one".to_string()),
                Value::String("two".to_string())
            ]
        );
    } else {
        panic!("Expected array result from Get-Content, got {:?}", result);
    }
}

#[test]
fn test_week16_get_content_with_select_object_skip_first() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    fs::write(&file_path, "one\ntwo\nthree\nfour\nfive\n").unwrap();

    let code = format!(
        "Get-Content '{}' | Select-Object -Skip 2 -First 2",
        file_path.to_string_lossy().replace('\\', "\\\\")
    );

    let result = eval_with_cmdlets(&code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::String("three".to_string()));
        assert_eq!(items[1], Value::String("four".to_string()));
    } else {
        panic!("Expected array result");
    }
}

// Week 16: Set-Content - Integration Tests

#[test]
fn test_week16_set_content_writes_string_value() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("out.txt");
    let path_str = file_path.to_string_lossy().replace('\\', "/");

    let code = format!(
        "Set-Content -Path '{}' -Value 'Hello'\nGet-Content '{}'",
        path_str, path_str
    );
    let result = eval_with_cmdlets(&code).unwrap();

    match result {
        Value::Array(items) => {
            assert_eq!(items, vec![Value::String("Hello".to_string())]);
        }
        Value::String(s) => {
            assert_eq!(s, "Hello".to_string());
        }
        other => panic!(
            "Expected string or array result from Get-Content, got {:?}",
            other
        ),
    }
}

#[test]
fn test_week16_set_content_writes_pipeline_values() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("out.txt");
    let path_str = file_path.to_string_lossy().replace('\\', "/");

    let code = format!(
        "@('a','b','c') | Set-Content -Path '{}'\nGet-Content '{}'",
        path_str, path_str
    );
    let result = eval_with_cmdlets(&code).unwrap();

    if let Value::Array(items) = result {
        assert_eq!(
            items,
            vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string())
            ]
        );
    } else {
        panic!("Expected array result from Get-Content, got {:?}", result);
    }
}

// Week 16: Additional File Cmdlets - Integration Tests (Chunk 5)

#[test]
fn test_week16_test_path_true_false() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("exists.txt");
    fs::write(&file_path, "hello").unwrap();

    let exists_str = file_path.to_string_lossy().replace('\\', "/");
    let missing_str = temp_dir
        .path()
        .join("missing.txt")
        .to_string_lossy()
        .replace('\\', "/");

    let code = format!("Test-Path '{}'\nTest-Path '{}'", exists_str, missing_str);
    let result = eval_with_cmdlets(&code).unwrap();

    // The statement result is the last cmdlet output.
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_week16_new_item_creates_file_and_directory() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("d");
    let file_path = temp_dir.path().join("f.txt");

    let dir_str = dir_path.to_string_lossy().replace('\\', "/");
    let file_str = file_path.to_string_lossy().replace('\\', "/");

    let code = format!(
        "New-Item -Path '{}' -Type 'Directory'\nNew-Item -Path '{}' -Type 'File'\nTest-Path '{}'",
        dir_str, file_str, file_str
    );
    let result = eval_with_cmdlets(&code).unwrap();
    assert_eq!(result, Value::Boolean(true));

    assert!(dir_path.exists() && dir_path.is_dir());
    assert!(file_path.exists() && file_path.is_file());
}

#[test]
fn test_week16_remove_item_deletes_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("todelete.txt");
    fs::write(&file_path, "hello").unwrap();

    let file_str = file_path.to_string_lossy().replace('\\', "/");

    let code = format!("Remove-Item -Path '{}'\nTest-Path '{}'", file_str, file_str);
    let result = eval_with_cmdlets(&code).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_week16_remove_item_recurse_deletes_directory_tree() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("tree");
    let file_path = dir_path.join("sub").join("x.txt");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "x").unwrap();

    let dir_str = dir_path.to_string_lossy().replace('\\', "/");

    let code = format!(
        "Remove-Item -Path '{}' -Recurse\nTest-Path '{}'",
        dir_str, dir_str
    );
    let result = eval_with_cmdlets(&code).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// Week 17: Sort-Object & Group-Object - Integration Tests

#[test]
fn test_week17_sort_object_numbers() {
    let result = eval_with_cmdlets("@(3,1,4,1,5,9) | Sort-Object").unwrap();

    if let Value::Array(items) = result {
        let nums: Vec<f64> = items
            .iter()
            .filter_map(|v| v.to_number())
            .collect();
        assert_eq!(nums, vec![1.0, 1.0, 3.0, 4.0, 5.0, 9.0]);
    } else {
        panic!("Expected array result from Sort-Object, got {:?}", result);
    }
}

#[test]
fn test_week17_sort_object_by_property_descending() {
    let code = r#"
        $procs = @(
            @{Name="a"; CPU=1}
            @{Name="b"; CPU=3}
            @{Name="c"; CPU=2}
        )
        $procs | Sort-Object -Property CPU -Descending true | Select-Object Name
    "#;

    let result = eval_with_cmdlets(code).unwrap();
    if let Value::Array(items) = result {
        let names: Vec<String> = items
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec!["b".to_string(), "c".to_string(), "a".to_string()]);
    } else {
        panic!("Expected array result, got {:?}", result);
    }
}

#[test]
fn test_week17_group_object_numbers() {
    let result = eval_with_cmdlets("@(1,2,2,3,3,3) | Group-Object").unwrap();

    if let Value::Array(items) = result {
        assert_eq!(items.len(), 3);
        let names: Vec<String> = items
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    } else {
        panic!("Expected array result from Group-Object, got {:?}", result);
    }
}

#[test]
fn test_week17_group_object_as_hash_table() {
    let result = eval_with_cmdlets("@('a','b','b') | Group-Object -AsHashTable true").unwrap();

    if let Value::Object(map) = result {
        assert!(map.contains_key("a"));
        assert!(map.contains_key("b"));

        let b_group = map.get("b").unwrap();
        assert_eq!(b_group.get_property("Count"), Some(Value::Number(2.0)));
    } else {
        panic!("Expected object hashtable result, got {:?}", result);
    }
}
