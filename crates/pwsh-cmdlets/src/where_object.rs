/// Where-Object cmdlet - filters objects based on conditions
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

/// Where-Object cmdlet filters pipeline input based on conditions
pub struct WhereObjectCmdlet;

impl Cmdlet for WhereObjectCmdlet {
    fn name(&self) -> &str {
        "Where-Object"
    }

    fn execute(
        &self,
        context: CmdletContext,
        evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Check if we have a script block as the first positional argument
        if let Some(first_arg) = context.arguments.first() {
            if let Value::ScriptBlock(script_block) = first_arg {
                // Filter using script block
                let mut results = Vec::new();

                for item in context.pipeline_input {
                    // Execute the script block with $_ set to the current item
                    // We clone here because execute_script_block takes ownership
                    let result = evaluator.execute_script_block(script_block, item.clone())?;

                    // If the result is truthy, include the item
                    if result.to_bool() {
                        results.push(item);
                    }
                }
                return Ok(results);
            }
        }

        // Check if we have a -Property parameter (simple name match)
        if let Some(property_value) = context.get_parameter("Property") {
            // Filter objects that have this property set to a truthy value
            let property_name = property_value.to_string();
            let mut results = Vec::new();

            for item in context.pipeline_input {
                if let Some(prop_val) = item.get_property(&property_name) {
                    if prop_val.to_bool() {
                        results.push(item);
                    }
                }
            }
            return Ok(results);
        }

        // For now, if no parameters, just pass through
        Ok(context.pipeline_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_where_object_no_filter() {
        let cmdlet = WhereObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let context = CmdletContext::with_input(input.clone());
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_where_object_with_property_filter() {
        let cmdlet = WhereObjectCmdlet;

        // Create objects with properties
        let mut obj1 = HashMap::new();
        obj1.insert("Active".to_string(), Value::Boolean(true));
        obj1.insert("Name".to_string(), Value::String("Object1".to_string()));

        let mut obj2 = HashMap::new();
        obj2.insert("Active".to_string(), Value::Boolean(false));
        obj2.insert("Name".to_string(), Value::String("Object2".to_string()));

        let mut obj3 = HashMap::new();
        obj3.insert("Active".to_string(), Value::Boolean(true));
        obj3.insert("Name".to_string(), Value::String("Object3".to_string()));

        let input = vec![
            Value::Object(obj1.clone()),
            Value::Object(obj2),
            Value::Object(obj3.clone()),
        ];

        let context = CmdletContext::with_input(input)
            .with_parameter("Property".to_string(), Value::String("Active".to_string()));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Value::Object(obj1));
        assert_eq!(result[1], Value::Object(obj3));
    }
}
