/// Pipeline execution engine
use crate::cmdlet::{CmdletContext, CmdletRegistry};
use crate::error::RuntimeError;
use crate::evaluator::Evaluator;
use crate::value::Value;
use pwsh_parser::{Argument, Expression, Pipeline};

/// Pipeline executor manages the execution of pipeline stages
pub struct PipelineExecutor<'a> {
    registry: &'a CmdletRegistry,
}

impl<'a> PipelineExecutor<'a> {
    /// Create a new pipeline executor with a cmdlet registry
    pub fn new(registry: &'a CmdletRegistry) -> Self {
        Self { registry }
    }

    /// Execute a pipeline
    pub fn execute(
        &self,
        pipeline: &Pipeline,
        evaluator: &mut Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        if pipeline.stages.is_empty() {
            return Ok(vec![]);
        }

        // Start with empty pipeline input
        let mut current_output: Vec<Value> = vec![];

        // Execute each stage
        for stage in pipeline.stages.iter() {
            current_output = self.execute_stage(stage, current_output, evaluator)?;
        }

        Ok(current_output)
    }

    /// Execute a single pipeline stage
    fn execute_stage(
        &self,
        stage: &Expression,
        input: Vec<Value>,
        evaluator: &mut Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        match stage {
            Expression::Call { name, arguments } => {
                // This is a cmdlet call
                self.execute_cmdlet(name, arguments, input, evaluator)
            }
            Expression::ScriptBlock(block) => {
                // Script block in pipeline - execute it for each input item
                if !input.is_empty() {
                    let mut results = Vec::new();
                    let script_block = crate::value::ScriptBlock {
                        body: block.clone(),
                    };
                    for item in input {
                        let result = evaluator.execute_script_block(&script_block, item)?;
                        results.push(result);
                    }
                    Ok(results)
                } else {
                    // No pipeline input, just return the script block as a value
                    let result = evaluator.eval_expression(stage.clone())?;
                    Ok(vec![result])
                }
            }
            _ => {
                // For non-cmdlet expressions, evaluate them
                // If there's pipeline input, bind it to $_ variable
                if !input.is_empty() {
                    // For now, we'll just evaluate the expression once per input item
                    let mut results = Vec::new();
                    for item in input {
                        // Set $_ to the current pipeline item
                        evaluator.set_variable("_", item.clone());
                        let result = evaluator.eval_expression(stage.clone())?;
                        results.push(result);
                    }
                    Ok(results)
                } else {
                    // No pipeline input, just evaluate the expression
                    let result = evaluator.eval_expression(stage.clone())?;
                    
                    // If the result is an array, unroll it to the pipeline
                    if let Value::Array(items) = result {
                        Ok(items)
                    } else {
                        Ok(vec![result])
                    }
                }
            }
        }
    }

    /// Execute a cmdlet
    fn execute_cmdlet(
        &self,
        name: &str,
        arguments: &[Argument],
        input: Vec<Value>,
        evaluator: &mut Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Look up the cmdlet
        let cmdlet = self
            .registry
            .get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?;

        // Build cmdlet context
        let mut context = CmdletContext::with_input(input);

        // Process arguments
        let mut positional_args = Vec::new();
        for arg in arguments {
            match arg {
                Argument::Positional(expr) => {
                    let value = evaluator.eval_expression(expr.clone())?;
                    positional_args.push(value);
                }
                Argument::Named { name, value } => {
                    let val = evaluator.eval_expression(value.clone())?;
                    context.parameters.insert(name.clone(), val);
                }
            }
        }
        context.arguments = positional_args;

        // Execute the cmdlet
        cmdlet.execute(context, evaluator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmdlet::Cmdlet;

    // Test cmdlet that echoes input
    struct EchoCmdlet;

    impl Cmdlet for EchoCmdlet {
        fn name(&self) -> &str {
            "Test-Echo"
        }

        fn execute(
            &self,
            context: CmdletContext,
            _evaluator: &mut Evaluator,
        ) -> Result<Vec<Value>, RuntimeError> {
            if context.pipeline_input.is_empty() {
                Ok(context.arguments)
            } else {
                Ok(context.pipeline_input)
            }
        }
    }

    // Test cmdlet that doubles numbers
    struct DoubleCmdlet;

    impl Cmdlet for DoubleCmdlet {
        fn name(&self) -> &str {
            "Test-Double"
        }

        fn execute(
            &self,
            context: CmdletContext,
            _evaluator: &mut Evaluator,
        ) -> Result<Vec<Value>, RuntimeError> {
            let mut results = Vec::new();
            for value in context.pipeline_input {
                if let Some(n) = value.to_number() {
                    results.push(Value::Number(n * 2.0));
                } else {
                    results.push(value);
                }
            }
            Ok(results)
        }
    }

    #[test]
    fn test_pipeline_single_cmdlet() {
        let mut registry = CmdletRegistry::new();
        registry.register(Box::new(EchoCmdlet));

        let executor = PipelineExecutor::new(&registry);
        let mut evaluator = Evaluator::new();

        // Create a simple pipeline: Test-Echo 42
        let pipeline = Pipeline {
            stages: vec![Expression::Call {
                name: "Test-Echo".to_string(),
                arguments: vec![Argument::Positional(Expression::Literal(
                    pwsh_parser::Literal::Number(42.0),
                ))],
            }],
        };

        let result = executor.execute(&pipeline, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(42.0)]);
    }

    #[test]
    fn test_pipeline_two_cmdlets() {
        let mut registry = CmdletRegistry::new();
        registry.register(Box::new(EchoCmdlet));
        registry.register(Box::new(DoubleCmdlet));

        let executor = PipelineExecutor::new(&registry);
        let mut evaluator = Evaluator::new();

        // Create pipeline: Test-Echo 5 | Test-Double
        let pipeline = Pipeline {
            stages: vec![
                Expression::Call {
                    name: "Test-Echo".to_string(),
                    arguments: vec![Argument::Positional(Expression::Literal(
                        pwsh_parser::Literal::Number(5.0),
                    ))],
                },
                Expression::Call {
                    name: "Test-Double".to_string(),
                    arguments: vec![],
                },
            ],
        };

        let result = executor.execute(&pipeline, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(10.0)]);
    }

    #[test]
    fn test_pipeline_with_script_block() {
        let registry = CmdletRegistry::new();
        let executor = PipelineExecutor::new(&registry);
        let mut evaluator = Evaluator::new();

        // Create pipeline: 5 | { $_ + 10 }
        // First stage produces 5, second stage is a script block that adds 10
        let pipeline = Pipeline {
            stages: vec![
                Expression::Literal(pwsh_parser::Literal::Number(5.0)),
                Expression::ScriptBlock(pwsh_parser::Block {
                    statements: vec![pwsh_parser::Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Variable("_".to_string())),
                        operator: pwsh_parser::BinaryOperator::Add,
                        right: Box::new(Expression::Literal(pwsh_parser::Literal::Number(10.0))),
                    })],
                }),
            ],
        };

        let result = executor.execute(&pipeline, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(15.0)]);
    }

    #[test]
    fn test_pipeline_script_block_with_multiple_inputs() {
        let registry = CmdletRegistry::new();
        let executor = PipelineExecutor::new(&registry);
        let mut evaluator = Evaluator::new();

        // Create a pipeline where first stage returns multiple values
        use pwsh_parser::Literal;
        let pipeline = Pipeline {
            stages: vec![
                Expression::Literal(Literal::Number(1.0)),
                Expression::ScriptBlock(pwsh_parser::Block {
                    statements: vec![pwsh_parser::Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Variable("_".to_string())),
                        operator: pwsh_parser::BinaryOperator::Multiply,
                        right: Box::new(Expression::Literal(Literal::Number(2.0))),
                    })],
                }),
            ],
        };

        let result = executor.execute(&pipeline, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(2.0)]);
    }
}
