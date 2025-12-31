/// Get-Process cmdlet - retrieves system process information
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;

/// Get-Process cmdlet retrieves process information
pub struct GetProcessCmdlet;

impl Cmdlet for GetProcessCmdlet {
    fn name(&self) -> &str {
        "Get-Process"
    }

    fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError> {
        // For Week 6 MVP, we'll create mock process data
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
        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_get_process_by_name() {
        let cmdlet = GetProcessCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Name".to_string(), Value::String("chrome".to_string()));
        let result = cmdlet.execute(context).unwrap();
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
        let result = cmdlet.execute(context).unwrap();
        
        if let Value::Object(props) = &result[0] {
            assert!(props.contains_key("Name"));
            assert!(props.contains_key("Id"));
            assert!(props.contains_key("CPU"));
            assert!(props.contains_key("WorkingSet"));
        } else {
            panic!("Expected object result");
        }
    }
}
