/// Get-Process cmdlet - retrieves system process information
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;

/// Get-Process cmdlet retrieves process information
pub struct GetProcessCmdlet;

impl Cmdlet for GetProcessCmdlet {
    fn name(&self) -> &str {
        "Get-Process"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // For now, we use mock process data for demonstration
        // In a production implementation, this would read from the OS

        let mock_processes = create_mock_processes();

        // Check if we have a -Name parameter to filter by name
        if let Some(name_value) = context.get_parameter("Name") {
            let filter_name = name_value.to_string().to_lowercase();
            let filtered: Vec<Value> = mock_processes
                .into_iter()
                .filter(|proc| {
                    if let Value::Object(props) = proc {
                        if let Some(Value::String(name)) = props.get("Name") {
                            return name.to_lowercase().contains(&filter_name);
                        }
                    }
                    false
                })
                .collect();
            return Ok(filtered);
        }

        Ok(mock_processes)
    }
}

/// Create mock process data for demonstration
fn create_mock_processes() -> Vec<Value> {
    vec![
        create_process("System", 4, 0.0, 1024),
        create_process("explorer", 1234, 15.5, 102400),
        create_process("chrome", 5678, 45.2, 512000),
        create_process("code", 9012, 23.1, 256000),
        create_process("pwsh", 3456, 5.0, 51200),
    ]
}

/// Helper to create a process object
fn create_process(name: &str, id: i32, cpu: f64, memory: i64) -> Value {
    let mut props = HashMap::new();
    props.insert("Name".to_string(), Value::String(name.to_string()));
    props.insert("Id".to_string(), Value::Number(id as f64));
    props.insert("CPU".to_string(), Value::Number(cpu));
    props.insert("WorkingSet".to_string(), Value::Number(memory as f64));
    Value::Object(props)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_process_all() {
        let cmdlet = GetProcessCmdlet;
        let context = CmdletContext::new();
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_get_process_by_name() {
        let cmdlet = GetProcessCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Name".to_string(), Value::String("chrome".to_string()));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::Object(props) = &result[0] {
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("chrome".to_string()))
            );
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_get_process_properties() {
        let cmdlet = GetProcessCmdlet;
        let context = CmdletContext::new();
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        if let Value::Object(props) = &result[0] {
            assert!(props.contains_key("Name"));
            assert!(props.contains_key("Id"));
            assert!(props.contains_key("CPU"));
            assert!(props.contains_key("WorkingSet"));
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_week14_success_criteria() {
        // Week 14 Success Criteria from ROADMAP.md:
        // Get-Process | 
        //     Where-Object { $_.CPU -gt 10 } | 
        //     Select-Object Name, CPU | 
        //     ForEach-Object { Write-Output "$($_.Name): $($_.CPU)" }
        
        use pwsh_lexer::Lexer;
        use pwsh_parser::Parser;
        use pwsh_runtime::Evaluator;

        let code = r#"Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let mut evaluator = Evaluator::new();
        
        // Register cmdlets
        crate::register_all(evaluator.registry_mut());
        
        let result = evaluator.eval(program).unwrap();
        
        // Should return array of objects with Name and CPU properties
        // Processes with CPU > 10: explorer (15.5), chrome (45.2), code (23.1)
        match result {
            Value::Array(values) => {
                assert_eq!(values.len(), 3, "Expected 3 processes with CPU > 10");
                
                // Verify each result has Name and CPU properties
                for val in &values {
                    if let Value::Object(props) = val {
                        assert!(props.contains_key("Name"), "Result should have Name property");
                        assert!(props.contains_key("CPU"), "Result should have CPU property");
                        assert_eq!(props.len(), 2, "Result should only have Name and CPU properties");
                    } else {
                        panic!("Expected Object in results, got {:?}", val);
                    }
                }
            }
            _ => panic!("Expected Array result, got {:?}", result),
        }
    }
}
