/// Select-Object cmdlet - selects specific properties from objects
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;

/// Select-Object cmdlet selects properties from objects
pub struct SelectObjectCmdlet;

impl Cmdlet for SelectObjectCmdlet {
    fn name(&self) -> &str {
        "Select-Object"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Extract parameters before consuming pipeline_input
        let property_param = context.parameters.get("Property").cloned();
        let first_param = context.parameters.get("First").cloned();
        let last_param = context.parameters.get("Last").cloned();

        // Start with the pipeline input
        let mut input = context.pipeline_input;

        // Check for -Property parameter (select specific properties)
        // Also support positional arguments: Select-Object Name, CPU
        let property_value = if let Some(prop) = property_param {
            Some(prop)
        } else if !context.arguments.is_empty() {
            // Use positional arguments as properties
            Some(Value::Array(context.arguments.clone()))
        } else {
            None
        };

        if let Some(property_value) = property_value {
            // Property can be a string (single property) or array (multiple properties)
            let properties = match property_value {
                Value::String(s) => vec![s.clone()],
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| {
                        if let Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => vec![],
            };

            // Select only specified properties from each object
            let mut results = Vec::new();
            for item in input {
                match item {
                    Value::Object(_) => {
                        let mut new_obj = HashMap::new();
                        for prop_name in &properties {
                            // Use case-insensitive property lookup
                            if let Some(value) = item.get_property(prop_name) {
                                new_obj.insert(prop_name.clone(), value);
                            }
                        }
                        results.push(Value::Object(new_obj));
                    }
                    // For non-objects, just pass through
                    _ => results.push(item),
                }
            }
            input = results;
        }

        // Check for -First parameter (limit output from the beginning)
        if let Some(first_value) = first_param {
            if let Some(count) = first_value.to_number() {
                let limit = count as usize;
                return Ok(input.into_iter().take(limit).collect());
            }
        }

        // Check for -Last parameter (limit output from the end)
        if let Some(last_value) = last_param {
            if let Some(count) = last_value.to_number() {
                let limit = count as usize;
                let total = input.len();
                let skip = total.saturating_sub(limit);
                return Ok(input.into_iter().skip(skip).collect());
            }
        }

        // No limiting parameters, return all
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_object_first() {
        let cmdlet = SelectObjectCmdlet;
        let input = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        let context = CmdletContext::with_input(input)
            .with_parameter("First".to_string(), Value::Number(2.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(1.0), Value::Number(2.0)]);
    }

    #[test]
    fn test_select_object_property() {
        let cmdlet = SelectObjectCmdlet;

        let mut obj = HashMap::new();
        obj.insert("Name".to_string(), Value::String("Test".to_string()));
        obj.insert("Value".to_string(), Value::Number(42.0));
        obj.insert("Extra".to_string(), Value::String("Ignore".to_string()));

        let input = vec![Value::Object(obj)];
        let context = CmdletContext::with_input(input)
            .with_parameter("Property".to_string(), Value::String("Name".to_string()));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::Object(props) = &result[0] {
            assert_eq!(props.len(), 1);
            assert_eq!(props.get("Name"), Some(&Value::String("Test".to_string())));
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_select_object_no_params() {
        let cmdlet = SelectObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(2.0)];
        let context = CmdletContext::with_input(input.clone());
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_select_object_last() {
        let cmdlet = SelectObjectCmdlet;
        let input = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        let context =
            CmdletContext::with_input(input).with_parameter("Last".to_string(), Value::Number(2.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(4.0), Value::Number(5.0)]);
    }

    #[test]
    fn test_select_object_multiple_properties() {
        let cmdlet = SelectObjectCmdlet;

        let mut obj1 = HashMap::new();
        obj1.insert("Name".to_string(), Value::String("chrome".to_string()));
        obj1.insert("CPU".to_string(), Value::Number(45.2));
        obj1.insert("Id".to_string(), Value::Number(5678.0));
        obj1.insert("WorkingSet".to_string(), Value::Number(512000.0));

        let mut obj2 = HashMap::new();
        obj2.insert("Name".to_string(), Value::String("code".to_string()));
        obj2.insert("CPU".to_string(), Value::Number(23.1));
        obj2.insert("Id".to_string(), Value::Number(9012.0));
        obj2.insert("WorkingSet".to_string(), Value::Number(256000.0));

        let input = vec![Value::Object(obj1), Value::Object(obj2)];

        // Select Name and CPU properties
        let properties = vec![
            Value::String("Name".to_string()),
            Value::String("CPU".to_string()),
        ];
        let context = CmdletContext::with_input(input)
            .with_parameter("Property".to_string(), Value::Array(properties));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);

        // Verify first object only has Name and CPU
        if let Value::Object(props) = &result[0] {
            assert_eq!(props.len(), 2);
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("chrome".to_string()))
            );
            assert_eq!(props.get("CPU"), Some(&Value::Number(45.2)));
            assert_eq!(props.get("Id"), None);
        } else {
            panic!("Expected object result");
        }

        // Verify second object only has Name and CPU
        if let Value::Object(props) = &result[1] {
            assert_eq!(props.len(), 2);
            assert_eq!(props.get("Name"), Some(&Value::String("code".to_string())));
            assert_eq!(props.get("CPU"), Some(&Value::Number(23.1)));
            assert_eq!(props.get("WorkingSet"), None);
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_select_object_property_then_first() {
        let cmdlet = SelectObjectCmdlet;

        let mut obj1 = HashMap::new();
        obj1.insert("Name".to_string(), Value::String("Test1".to_string()));
        obj1.insert("Value".to_string(), Value::Number(10.0));

        let mut obj2 = HashMap::new();
        obj2.insert("Name".to_string(), Value::String("Test2".to_string()));
        obj2.insert("Value".to_string(), Value::Number(20.0));

        let mut obj3 = HashMap::new();
        obj3.insert("Name".to_string(), Value::String("Test3".to_string()));
        obj3.insert("Value".to_string(), Value::Number(30.0));

        let input = vec![
            Value::Object(obj1),
            Value::Object(obj2),
            Value::Object(obj3),
        ];

        let context = CmdletContext::with_input(input)
            .with_parameter("Property".to_string(), Value::String("Name".to_string()))
            .with_parameter("First".to_string(), Value::Number(2.0));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);

        // Verify objects only have Name property
        if let Value::Object(props) = &result[0] {
            assert_eq!(props.len(), 1);
            assert_eq!(props.get("Name"), Some(&Value::String("Test1".to_string())));
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_select_object_case_insensitive_properties() {
        // Test case for issue: Property lookup should be case-insensitive
        let cmdlet = SelectObjectCmdlet;

        // Create object with properties in specific case
        let mut obj = HashMap::new();
        obj.insert("CPU".to_string(), Value::Number(45.2));
        obj.insert("Name".to_string(), Value::String("pwsh".to_string()));
        obj.insert("Id".to_string(), Value::Number(3456.0));

        let input = vec![Value::Object(obj)];

        // Select using different case - "CPu" instead of "CPU"
        let properties = vec![Value::String("CPu".to_string())];
        let context = CmdletContext::with_input(input)
            .with_parameter("Property".to_string(), Value::Array(properties));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);

        // Verify the property was found despite case difference
        if let Value::Object(props) = &result[0] {
            assert_eq!(props.len(), 1);
            // The result should contain the property with the value
            // Note: The key uses the requested case "CPu"
            assert!(props.contains_key("CPu"));
            assert_eq!(props.get("CPu"), Some(&Value::Number(45.2)));
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_week12_success_criteria() {
        let cmdlet = SelectObjectCmdlet;

        // Create mock process objects
        let mut chrome = HashMap::new();
        chrome.insert("Name".to_string(), Value::String("chrome".to_string()));
        chrome.insert("CPU".to_string(), Value::Number(45.2));
        chrome.insert("Id".to_string(), Value::Number(5678.0));

        let mut code = HashMap::new();
        code.insert("Name".to_string(), Value::String("code".to_string()));
        code.insert("CPU".to_string(), Value::Number(23.1));
        code.insert("Id".to_string(), Value::Number(9012.0));

        let mut pwsh = HashMap::new();
        pwsh.insert("Name".to_string(), Value::String("pwsh".to_string()));
        pwsh.insert("CPU".to_string(), Value::Number(5.0));
        pwsh.insert("Id".to_string(), Value::Number(3456.0));

        let input = vec![
            Value::Object(chrome),
            Value::Object(code),
            Value::Object(pwsh),
        ];

        // Test: $objects | Select-Object Name, CPU
        let properties = vec![
            Value::String("Name".to_string()),
            Value::String("CPU".to_string()),
        ];
        let context = CmdletContext::with_input(input.clone())
            .with_parameter("Property".to_string(), Value::Array(properties));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 3);

        // Verify each object only has Name and CPU
        for obj in result {
            if let Value::Object(props) = obj {
                assert_eq!(props.len(), 2);
                assert!(props.contains_key("Name"));
                assert!(props.contains_key("CPU"));
                assert!(!props.contains_key("Id"));
            } else {
                panic!("Expected object result");
            }
        }

        // Test: $objects | Select-Object -First 5
        let context = CmdletContext::with_input(input)
            .with_parameter("First".to_string(), Value::Number(5.0));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 3); // Only 3 objects available, so returns all 3
    }
}
