/// Write-Output cmdlet - outputs values to the pipeline
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

/// Write-Output cmdlet sends objects to the output stream
pub struct WriteOutputCmdlet;

impl Cmdlet for WriteOutputCmdlet {
    fn name(&self) -> &str {
        "Write-Output"
    }

    fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError> {
        // If there's pipeline input, output it
        if !context.pipeline_input.is_empty() {
            return Ok(context.pipeline_input);
        }

        // Otherwise, output the arguments
        if !context.arguments.is_empty() {
            return Ok(context.arguments);
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
        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result, vec![Value::Number(42.0)]);
    }

    #[test]
    fn test_write_output_with_pipeline_input() {
        let cmdlet = WriteOutputCmdlet;
        let input = vec![Value::String("Hello".to_string())];
        let context = CmdletContext::with_input(input.clone());
        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_write_output_multiple_values() {
        let cmdlet = WriteOutputCmdlet;
        let values = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let context = CmdletContext::new().with_arguments(values.clone());
        let result = cmdlet.execute(context).unwrap();
        assert_eq!(result, values);
    }
}
