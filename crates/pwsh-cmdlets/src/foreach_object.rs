/// ForEach-Object cmdlet - processes each object in pipeline
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

/// ForEach-Object cmdlet processes each pipeline object
pub struct ForEachObjectCmdlet;

impl Cmdlet for ForEachObjectCmdlet {
    fn name(&self) -> &str {
        "ForEach-Object"
    }

    fn execute(
        &self,
        context: CmdletContext,
        evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Check if we have a script block as the first positional argument
        if let Some(Value::ScriptBlock(script_block)) = context.arguments.first() {
            // Execute script block for each item
            let mut results = Vec::new();

            for item in context.pipeline_input {
                // Execute the script block with $_ set to the current item
                let result = evaluator.execute_script_block(script_block, item)?;
                results.push(result);
            }
            return Ok(results);
        }

        // Check if we have a -MemberName parameter (access a property)
        if let Some(member_value) = context.get_parameter("MemberName") {
            let member_name = member_value.to_string();
            let mut results = Vec::new();

            for item in context.pipeline_input {
                if let Some(prop_val) = item.get_property(&member_name) {
                    results.push(prop_val);
                } else {
                    // If property doesn't exist, push null
                    results.push(Value::Null);
                }
            }
            return Ok(results);
        }

        // For now, without script block support, just pass through
        Ok(context.pipeline_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_foreach_object_no_params() {
        let cmdlet = ForEachObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let context = CmdletContext::with_input(input.clone());
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_foreach_object_with_member_name() {
        let cmdlet = ForEachObjectCmdlet;

        let mut obj1 = HashMap::new();
        obj1.insert("Name".to_string(), Value::String("Object1".to_string()));
        obj1.insert("Value".to_string(), Value::Number(10.0));

        let mut obj2 = HashMap::new();
        obj2.insert("Name".to_string(), Value::String("Object2".to_string()));
        obj2.insert("Value".to_string(), Value::Number(20.0));

        let input = vec![Value::Object(obj1), Value::Object(obj2)];
        let context = CmdletContext::with_input(input)
            .with_parameter("MemberName".to_string(), Value::String("Name".to_string()));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Value::String("Object1".to_string()));
        assert_eq!(result[1], Value::String("Object2".to_string()));
    }
}
