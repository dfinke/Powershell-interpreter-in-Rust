/// Select-Object cmdlet - selects specific properties from objects
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;

/// Select-Object cmdlet selects properties from objects
pub struct SelectObjectCmdlet;

impl Cmdlet for SelectObjectCmdlet {
    fn name(&self) -> &str {
        "Select-Object"
    }

    fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError> {
        let mut results = Vec::new();

        // Check for -First parameter (limit output)
        if let Some(first_value) = context.get_parameter("First") {
            if let Some(count) = first_value.to_number() {
                let limit = count as usize;
                return Ok(context.pipeline_input.into_iter().take(limit).collect());
            }
        }

        // Check for -Property parameter (select specific properties)
        if let Some(property_value) = context.get_parameter("Property") {
            // Property can be a string (single property) or array (multiple properties)
            let properties = match property_value {
                Value::String(s) => vec![s.clone()],
                Value::Array(arr) => {
                    arr.iter()
                        .filter_map(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                }
                _ => vec![],
            };

            // Select only specified properties from each object
            for item in context.pipeline_input {
                match item {
                    Value::Object(props) => {
                        let mut new_obj = HashMap::new();
                        for prop_name in &properties {
                            if let Some(value) = props.get(prop_name) {
                                new_obj.insert(prop_name.clone(), value.clone());
                            }
                        }
                        results.push(Value::Object(new_obj));
                    }
                    // For non-objects, just pass through
                    _ => results.push(item),
                }
            }
            return Ok(results);
        }

        // No parameters, pass through input
        Ok(context.pipeline_input)
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
        let result = cmdlet.execute(context).unwrap();
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

        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::Object(props) = &result[0] {
            assert_eq!(props.len(), 1);
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("Test".to_string()))
            );
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_select_object_no_params() {
        let cmdlet = SelectObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(2.0)];
        let context = CmdletContext::with_input(input.clone());
        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result, input);
    }
}
