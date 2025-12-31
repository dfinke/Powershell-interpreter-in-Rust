/// Evaluator for PowerShell AST
use crate::cmdlet::CmdletRegistry;
use crate::error::RuntimeError;
use crate::scope::ScopeStack;
use crate::value::Value;
use pwsh_parser::{
    BinaryOperator, Block, Expression, Literal, Program, Statement, StringPart, UnaryOperator,
};

/// Result type for evaluation
pub type EvalResult = Result<Value, RuntimeError>;

/// Evaluator executes PowerShell AST
pub struct Evaluator {
    scope: ScopeStack,
    cmdlet_registry: CmdletRegistry,
}

impl Evaluator {
    /// Create a new evaluator with empty scope
    pub fn new() -> Self {
        Evaluator {
            scope: ScopeStack::new(),
            cmdlet_registry: CmdletRegistry::new(),
        }
    }

    /// Create evaluator with a custom cmdlet registry
    pub fn with_registry(registry: CmdletRegistry) -> Self {
        Evaluator {
            scope: ScopeStack::new(),
            cmdlet_registry: registry,
        }
    }

    /// Get a mutable reference to the cmdlet registry
    pub fn registry_mut(&mut self) -> &mut CmdletRegistry {
        &mut self.cmdlet_registry
    }

    /// Evaluate a program (list of statements)
    pub fn eval(&mut self, program: Program) -> EvalResult {
        let mut result = Value::Null;
        for statement in program.statements {
            result = self.eval_statement(statement)?;
        }
        Ok(result)
    }

    /// Set a variable in the current scope
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.scope.set_variable(name, value);
    }

    /// Get a variable from the current scope
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        self.scope.get_variable(name)
    }

    /// Evaluate a single statement
    pub fn eval_statement(&mut self, statement: Statement) -> EvalResult {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr),

            Statement::Assignment { variable, value } => {
                let val = self.eval_expression(value)?;
                self.scope.set_variable(&variable, val);
                Ok(Value::Null)
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_expression(condition)?;
                if cond_value.to_bool() {
                    self.eval_block(then_branch)
                } else if let Some(else_block) = else_branch {
                    self.eval_block(else_block)
                } else {
                    Ok(Value::Null)
                }
            }

            Statement::Return(expr) => {
                // For now, we'll just evaluate and return the value
                // In a full implementation, this would need special handling to break out of functions
                if let Some(expression) = expr {
                    self.eval_expression(expression)
                } else {
                    Ok(Value::Null)
                }
            }

            Statement::FunctionDef { .. } => {
                // Function definitions will be implemented in Phase 2
                // For now, just return null
                Ok(Value::Null)
            }

            Statement::Pipeline(pipeline) => {
                // Execute the pipeline
                let results = self.execute_pipeline(&pipeline)?;

                // For display purposes, return the last value or Null
                Ok(results.last().cloned().unwrap_or(Value::Null))
            }
        }
    }

    /// Execute a pipeline
    fn execute_pipeline(
        &mut self,
        pipeline: &pwsh_parser::Pipeline,
    ) -> Result<Vec<Value>, RuntimeError> {
        if pipeline.stages.is_empty() {
            return Ok(vec![]);
        }

        // Start with empty pipeline input
        let mut current_output: Vec<Value> = vec![];

        // Execute each stage
        for stage in pipeline.stages.iter() {
            current_output = self.execute_pipeline_stage(stage, current_output)?;
        }

        Ok(current_output)
    }

    /// Execute a single pipeline stage
    fn execute_pipeline_stage(
        &mut self,
        stage: &Expression,
        input: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        match stage {
            Expression::Call { name, arguments } => {
                // This is a cmdlet call
                self.execute_cmdlet_call(name, arguments, input)
            }
            _ => {
                // For non-cmdlet expressions, evaluate them
                // If there's pipeline input, bind it to $_ variable
                if !input.is_empty() {
                    let mut results = Vec::new();
                    for item in input {
                        // Set $_ to the current pipeline item
                        self.set_variable("_", item.clone());
                        let result = self.eval_expression(stage.clone())?;
                        results.push(result);
                    }
                    Ok(results)
                } else {
                    // No pipeline input, just evaluate the expression
                    let result = self.eval_expression(stage.clone())?;
                    Ok(vec![result])
                }
            }
        }
    }

    /// Execute a cmdlet call
    fn execute_cmdlet_call(
        &mut self,
        name: &str,
        arguments: &[pwsh_parser::Argument],
        input: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        use crate::cmdlet::CmdletContext;

        // First, check if cmdlet exists
        if !self.cmdlet_registry.contains(name) {
            return Err(RuntimeError::UndefinedFunction(name.to_string()));
        }

        // Build cmdlet context by evaluating arguments first
        let mut context = CmdletContext::with_input(input);
        let mut positional_args = Vec::new();

        for arg in arguments {
            match arg {
                pwsh_parser::Argument::Positional(expr) => {
                    let value = self.eval_expression(expr.clone())?;
                    positional_args.push(value);
                }
                pwsh_parser::Argument::Named {
                    name: param_name,
                    value,
                } => {
                    let val = self.eval_expression(value.clone())?;
                    context.parameters.insert(param_name.clone(), val);
                }
            }
        }
        context.arguments = positional_args;

        // Now we can get the cmdlet and execute it
        let cmdlet = self.cmdlet_registry.get(name).ok_or_else(|| {
            // This should never happen as we checked earlier, but handle it gracefully
            RuntimeError::UndefinedFunction(name.to_string())
        })?;

        // Execute the cmdlet
        cmdlet.execute(context)
    }

    /// Evaluate a block of statements
    fn eval_block(&mut self, block: Block) -> EvalResult {
        self.scope.push_scope();
        let mut result = Value::Null;

        for statement in block.statements {
            result = self.eval_statement(statement)?;
        }

        self.scope.pop_scope();
        Ok(result)
    }

    /// Evaluate an expression
    pub fn eval_expression(&mut self, expr: Expression) -> EvalResult {
        match expr {
            Expression::Literal(lit) => self.eval_literal(lit),

            Expression::Variable(name) => {
                Ok(self.scope.get_variable(&name).unwrap_or(Value::Number(0.0)))
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_val = self.eval_expression(*left)?;
                let right_val = self.eval_expression(*right)?;
                self.eval_binary_op(left_val, operator, right_val)
            }

            Expression::UnaryOp { operator, operand } => {
                let operand_val = self.eval_expression(*operand)?;
                self.eval_unary_op(operator, operand_val)
            }

            Expression::MemberAccess { object, member } => {
                let obj_val = self.eval_expression(*object)?;
                obj_val.get_property(&member).ok_or_else(|| {
                    RuntimeError::InvalidPropertyAccess(format!("Property '{}' not found", member))
                })
            }

            Expression::Call { name, arguments } => {
                // This is a cmdlet call - execute it with empty pipeline input
                let results = self.execute_cmdlet_call(&name, &arguments, vec![])?;
                // Return the last value or Null
                Ok(results.last().cloned().unwrap_or(Value::Null))
            }

            Expression::ScriptBlock(_) => {
                // Script blocks will be implemented later with full evaluation
                Ok(Value::Null)
            }
        }
    }

    /// Evaluate a literal value
    fn eval_literal(&mut self, literal: Literal) -> EvalResult {
        match literal {
            Literal::Number(n) => Ok(Value::Number(n)),
            Literal::String(s) => Ok(Value::String(s)),
            Literal::Boolean(b) => Ok(Value::Boolean(b)),
            Literal::Null => Ok(Value::Null),
            Literal::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(&s),
                        StringPart::Variable(name) => {
                            let value = self
                                .scope
                                .get_variable(&name)
                                .unwrap_or(Value::String("".to_string()));
                            result.push_str(&value.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }

    /// Evaluate a binary operation
    fn eval_binary_op(&self, left: Value, operator: BinaryOperator, right: Value) -> EvalResult {
        match operator {
            // Arithmetic operators
            BinaryOperator::Add => match (&left, &right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                (Value::String(l), r) => Ok(Value::String(format!("{}{}", l, r))),
                (l, Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "number or string".to_string(),
                    got: format!("{:?} and {:?}", left, right),
                    operation: "addition".to_string(),
                }),
            },

            BinaryOperator::Subtract => {
                self.numeric_binary_op(left, right, "subtraction", |l, r| l - r)
            }

            BinaryOperator::Multiply => {
                self.numeric_binary_op(left, right, "multiplication", |l, r| l * r)
            }

            BinaryOperator::Divide => {
                let l = left.to_number().ok_or_else(|| RuntimeError::TypeMismatch {
                    expected: "number".to_string(),
                    got: format!("{:?}", left),
                    operation: "division".to_string(),
                })?;
                let r = right
                    .to_number()
                    .ok_or_else(|| RuntimeError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", right),
                        operation: "division".to_string(),
                    })?;

                if r == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Number(l / r))
            }

            BinaryOperator::Modulo => self.numeric_binary_op(left, right, "modulo", |l, r| l % r),

            // Comparison operators
            BinaryOperator::Equal => Ok(Value::Boolean(self.values_equal(&left, &right))),

            BinaryOperator::NotEqual => Ok(Value::Boolean(!self.values_equal(&left, &right))),

            BinaryOperator::Greater => {
                self.comparison_op(left, right, "greater than", |l, r| l > r)
            }

            BinaryOperator::Less => self.comparison_op(left, right, "less than", |l, r| l < r),

            BinaryOperator::GreaterOrEqual => {
                self.comparison_op(left, right, "greater or equal", |l, r| l >= r)
            }

            BinaryOperator::LessOrEqual => {
                self.comparison_op(left, right, "less or equal", |l, r| l <= r)
            }
        }
    }

    /// Helper for numeric binary operations
    fn numeric_binary_op<F>(&self, left: Value, right: Value, op_name: &str, f: F) -> EvalResult
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let l = left.to_number().ok_or_else(|| RuntimeError::TypeMismatch {
            expected: "number".to_string(),
            got: format!("{:?}", left),
            operation: op_name.to_string(),
        })?;
        let r = right
            .to_number()
            .ok_or_else(|| RuntimeError::TypeMismatch {
                expected: "number".to_string(),
                got: format!("{:?}", right),
                operation: op_name.to_string(),
            })?;

        Ok(Value::Number(f(l, r)))
    }

    /// Helper for comparison operations
    fn comparison_op<F>(&self, left: Value, right: Value, op_name: &str, f: F) -> EvalResult
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let l = left.to_number().ok_or_else(|| RuntimeError::TypeMismatch {
            expected: "number".to_string(),
            got: format!("{:?}", left),
            operation: op_name.to_string(),
        })?;
        let r = right
            .to_number()
            .ok_or_else(|| RuntimeError::TypeMismatch {
                expected: "number".to_string(),
                got: format!("{:?}", right),
                operation: op_name.to_string(),
            })?;

        Ok(Value::Boolean(f(l, r)))
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            _ => false,
        }
    }

    /// Evaluate a unary operation
    fn eval_unary_op(&self, operator: UnaryOperator, operand: Value) -> EvalResult {
        match operator {
            UnaryOperator::Negate => {
                let n = operand
                    .to_number()
                    .ok_or_else(|| RuntimeError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", operand),
                        operation: "negation".to_string(),
                    })?;
                Ok(Value::Number(-n))
            }
            UnaryOperator::Not => Ok(Value::Boolean(!operand.to_bool())),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pwsh_lexer::Lexer;
    use pwsh_parser::Parser;

    fn eval_str(input: &str) -> EvalResult {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let mut evaluator = Evaluator::new();
        evaluator.eval(program)
    }

    #[test]
    fn test_eval_number() {
        let result = eval_str("42").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_string() {
        let result = eval_str("\"hello\"").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_boolean() {
        let result = eval_str("true").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_addition() {
        let result = eval_str("5 + 3").unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_eval_subtraction() {
        let result = eval_str("10 - 3").unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_eval_multiplication() {
        let result = eval_str("6 * 7").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_division() {
        let result = eval_str("20 / 4").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_modulo() {
        let result = eval_str("10 % 3").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let result = eval_str("10 / 0");
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_eval_operator_precedence() {
        let result = eval_str("10 + 20 * 2").unwrap();
        assert_eq!(result, Value::Number(50.0)); // 10 + (20 * 2)
    }

    #[test]
    fn test_eval_parentheses() {
        let result = eval_str("(10 + 20) * 2").unwrap();
        assert_eq!(result, Value::Number(60.0));
    }

    #[test]
    fn test_eval_unary_minus() {
        let result = eval_str("-5").unwrap();
        assert_eq!(result, Value::Number(-5.0));
    }

    #[test]
    fn test_eval_comparison_equal() {
        let result = eval_str("5 -eq 5").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_str("5 -eq 3").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_comparison_not_equal() {
        let result = eval_str("5 -ne 3").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_comparison_greater() {
        let result = eval_str("10 -gt 5").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eval_str("3 -gt 5").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_comparison_less() {
        let result = eval_str("3 -lt 5").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_assignment() {
        let result = eval_str("$x = 5").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_eval_variable_reference() {
        let mut lexer = Lexer::new("$x = 5\n$x");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let mut evaluator = Evaluator::new();
        let result = evaluator.eval(program).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_variable_in_expression() {
        let result = eval_str("$x = 5\n$y = 10\n$x + $y").unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_eval_undefined_variable() {
        let result = eval_str("$undefined").unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_eval_undefined_variable_in_expression() {
        let result = eval_str("$x = 6\n$r = $x + $y").unwrap();
        assert_eq!(result, Value::Null); // assignment returns null
                                         // But to check $r, need to eval $r
        let result_r = eval_str("$x = 6\n$r = $x + $y\n$r").unwrap();
        assert_eq!(result_r, Value::Number(6.0));
    }

    #[test]
    fn test_eval_string_interpolation() {
        let result = eval_str("$name = \"World\"\n\"Hello $name\"").unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_eval_if_statement_true() {
        let result = eval_str("if (true) { 42 }").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_if_statement_false() {
        let result = eval_str("if (false) { 42 }").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_eval_if_else_statement() {
        let result = eval_str("if (false) { 1 } else { 2 }").unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_eval_if_with_condition() {
        let result = eval_str("$x = 5\nif ($x -eq 5) { 100 }").unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_eval_complex_expression() {
        let result = eval_str("$x = 5\n$y = 10\n$z = ($x + $y) * 2\n$z").unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[test]
    fn test_eval_string_concatenation() {
        let result = eval_str("\"Hello \" + \"World\"").unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_eval_nested_scopes() {
        let result = eval_str("$x = 1\nif (true) { $y = 2\n$x + $y }").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }
}
