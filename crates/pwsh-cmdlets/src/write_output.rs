/// Write-Output cmdlet - outputs values to the pipeline
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

/// Write-Output cmdlet sends objects to the output stream
pub struct WriteOutputCmdlet;

impl Cmdlet for WriteOutputCmdlet {
    fn name(&self) -> &str {
        "Write-Output"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let mut output = Vec::new();

        // If there's pipeline input, output it
        if !context.pipeline_input.is_empty() {
            for value in context.pipeline_input {
                // Unroll arrays to the pipeline
                if let Value::Array(items) = value {
                    output.extend(items);
                } else {
                    output.push(value);
                }
            }
            return Ok(output);
        }

        // Otherwise, output the arguments
        if !context.arguments.is_empty() {
            for value in context.arguments {
                // Unroll arrays to the pipeline
                if let Value::Array(items) = value {
                    output.extend(items);
                } else {
                    output.push(value);
                }
            }
            return Ok(output);
        }

        // If no input and no arguments, output nothing
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_output_with_argument() {
        let cmdlet = WriteOutputCmdlet;
        let context = CmdletContext::new().with_arguments(vec![Value::Number(42.0)]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(42.0)]);
    }

    #[test]
    fn test_write_output_with_pipeline_input() {
        let cmdlet = WriteOutputCmdlet;
        let input = vec![Value::String("Hello".to_string())];
        let context = CmdletContext::with_input(input.clone());
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_write_output_multiple_values() {
        let cmdlet = WriteOutputCmdlet;
        let values = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let context = CmdletContext::new().with_arguments(values.clone());
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, values);
    }
}
