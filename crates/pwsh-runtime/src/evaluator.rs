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
        self.scope.set_variable_qualified(name, value);
    }

    /// Get a variable from the current scope
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        self.scope.get_variable_qualified(name)
    }

    /// Evaluate a single statement
    pub fn eval_statement(&mut self, statement: Statement) -> EvalResult {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr),

            Statement::Assignment { variable, value } => {
                let val = self.eval_expression(value)?;
                self.scope.set_variable_qualified(&variable, val);
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
                // Throw an EarlyReturn error to propagate up the call stack
                let value = if let Some(expression) = expr {
                    self.eval_expression(expression)?
                } else {
                    Value::Null
                };
                Err(RuntimeError::EarlyReturn(value))
            }

            Statement::FunctionDef {
                name,
                parameters,
                body,
            } => {
                // Store the function as a value in the current scope
                let func = crate::value::Function {
                    name: name.clone(),
                    parameters,
                    body,
                };
                self.scope.set_variable(&name, Value::Function(func));
                Ok(Value::Null)
            }

            Statement::Pipeline(pipeline) => {
                // Execute the pipeline
                let results = self.execute_pipeline(&pipeline)?;

                // Return all results as an array if multiple, or single value if one, or null if none
                match results.len() {
                    0 => Ok(Value::Null),
                    1 => Ok(results[0].clone()),
                    _ => Ok(Value::Array(results)),
                }
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
            Expression::ScriptBlock(block) => {
                // Script block in pipeline - execute it for each input item
                if !input.is_empty() {
                    let mut results = Vec::new();
                    let script_block = crate::value::ScriptBlock {
                        body: block.clone(),
                    };
                    for item in input {
                        let result = self.execute_script_block(&script_block, item)?;
                        results.push(result);
                    }
                    Ok(results)
                } else {
                    // No pipeline input, just return the script block as a value
                    let result = self.eval_expression(stage.clone())?;
                    Ok(vec![result])
                }
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

    /// Execute a cmdlet call
    fn execute_cmdlet_call(
        &mut self,
        name: &str,
        arguments: &[pwsh_parser::Argument],
        input: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        // First, check if this is a user-defined function
        if let Some(Value::Function(func)) = self.scope.get_variable(name) {
            // Call the user-defined function
            let result = self.call_function(&func, arguments)?;
            return Ok(vec![result]);
        }

        // If not a function, try cmdlets
        use crate::cmdlet::CmdletContext;

        // Check if cmdlet exists
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

        // Create a raw pointer to self to work around borrow checker
        // SAFETY: We ensure that the cmdlet execution doesn't invalidate the registry reference
        // The cmdlet will only mutate the scope/state, not the registry itself
        let self_ptr = self as *mut Evaluator;
        let cmdlet = self
            .cmdlet_registry
            .get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?;

        // Execute the cmdlet
        // SAFETY: The cmdlet reference and the mutable self reference don't overlap in memory
        // as cmdlet points into the registry and the mutable operations affect scope/state
        unsafe { cmdlet.execute(context, &mut *self_ptr) }
    }

    /// Call a user-defined function
    fn call_function(
        &mut self,
        func: &crate::value::Function,
        arguments: &[pwsh_parser::Argument],
    ) -> EvalResult {
        // Create a new scope for the function
        self.scope.push_scope();

        // Evaluate arguments
        let mut positional_args = Vec::new();
        for arg in arguments {
            match arg {
                pwsh_parser::Argument::Positional(expr) => {
                    let value = self.eval_expression(expr.clone())?;
                    positional_args.push(value);
                }
                pwsh_parser::Argument::Named { name, .. } => {
                    // Named parameters not yet supported for user functions
                    // Log a warning and skip this argument
                    eprintln!("Warning: Named parameter '-{}' is not yet supported for user-defined functions and will be ignored", name);
                }
            }
        }

        // Bind parameters to arguments
        for (i, param) in func.parameters.iter().enumerate() {
            let value = if i < positional_args.len() {
                // Use provided argument
                positional_args[i].clone()
            } else if let Some(default_expr) = &param.default_value {
                // Use default value
                self.eval_expression(default_expr.clone())?
            } else {
                // No value provided and no default - use Null
                Value::Null
            };
            self.scope.define_variable(&param.name, value);
        }

        // Execute the function body
        let result = self.eval_function_body(&func.body);

        // Pop the function scope
        self.scope.pop_scope();

        result
    }

    /// Evaluate a function body (handles return statements specially)
    fn eval_function_body(&mut self, block: &Block) -> EvalResult {
        let mut result = Value::Null;

        for statement in &block.statements {
            match self.eval_statement(statement.clone()) {
                Ok(val) => result = val,
                Err(RuntimeError::EarlyReturn(return_value)) => {
                    // Catch early return and return the value
                    return Ok(return_value);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
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

    /// Execute a script block with a specific value for $_
    /// This is used by cmdlets like Where-Object and ForEach-Object
    pub fn execute_script_block(
        &mut self,
        script_block: &crate::value::ScriptBlock,
        pipeline_value: Value,
    ) -> EvalResult {
        self.scope.push_scope();

        // Set $_ to the current pipeline value
        self.scope.set_variable_qualified("_", pipeline_value);

        let mut result = Value::Null;
        for statement in &script_block.body.statements {
            result = self.eval_statement(statement.clone())?;
        }

        self.scope.pop_scope();
        Ok(result)
    }

    /// Evaluate an expression
    pub fn eval_expression(&mut self, expr: Expression) -> EvalResult {
        match expr {
            Expression::Literal(lit) => self.eval_literal(lit),

            Expression::Variable(name) => Ok(self
                .scope
                .get_variable_qualified(&name)
                .unwrap_or(Value::Number(0.0))),

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

            Expression::ScriptBlock(block) => {
                // Create a script block value
                Ok(Value::ScriptBlock(crate::value::ScriptBlock {
                    body: block.clone(),
                }))
            }

            Expression::Hashtable(pairs) => {
                // Create a hashtable (Object with properties)
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.eval_expression(value_expr.clone())?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }

            Expression::Array(items) => {
                // Create an array
                let mut values = Vec::new();
                for item_expr in items {
                    let value = self.eval_expression(item_expr.clone())?;
                    values.push(value);
                }
                Ok(Value::Array(values))
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

    /// Check if two values are equal (case-insensitive for strings, PowerShell default)
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            // PowerShell string comparison is case-insensitive by default
            (Value::String(l), Value::String(r)) => l.to_lowercase() == r.to_lowercase(),
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

    #[test]
    fn test_function_definition() {
        let result = eval_str("function Add($a, $b) { $a + $b }").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_function_call_simple() {
        let result = eval_str("function Add($a, $b) { $a + $b }\nAdd 5 10").unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_function_call_with_return() {
        let result = eval_str("function Double($x) { return $x * 2 }\nDouble 21").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_function_with_explicit_return() {
        let result = eval_str("function GetFive() { return 5 }\nGetFive").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_function_with_implicit_return() {
        let result = eval_str("function GetTen() { 10 }\nGetTen").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_function_with_multiple_statements() {
        let result =
            eval_str("function Calculate($x) { $y = $x * 2\n$z = $y + 10\n$z }\nCalculate 5")
                .unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_function_with_default_parameter() {
        let result = eval_str("function Greet($name = \"World\") { $name }\nGreet").unwrap();
        assert_eq!(result, Value::String("World".to_string()));
    }

    #[test]
    fn test_function_override_default_parameter() {
        let result =
            eval_str("function Greet($name = \"World\") { $name }\nGreet \"Alice\"").unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_function_no_parameters() {
        let result = eval_str("function GetAnswer() { 42 }\nGetAnswer").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_function_with_variables() {
        let result = eval_str("function Test() { $x = 1\n$y = 2\n$x + $y }\nTest").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_function_scope_isolation() {
        let result =
            eval_str("$x = 100\nfunction Test() { $x = 1\n$x }\n$result = Test\n$result").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_function_can_access_outer_scope() {
        let result = eval_str("$x = 100\nfunction Test() { $x }\nTest").unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_function_early_return() {
        let result =
            eval_str("function Test($x) { if ($x -eq 5) { return 100 }\nreturn 200 }\nTest 5")
                .unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_function_return_without_value() {
        let result = eval_str("function Test() { return }\nTest").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_nested_function_calls() {
        let result = eval_str(
            "function Double($x) { $x * 2 }\nfunction Quad($x) { Double (Double $x) }\nQuad 5",
        )
        .unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_function_with_conditional() {
        let result =
            eval_str("function Max($a, $b) { if ($a -gt $b) { $a } else { $b } }\nMax 10 5")
                .unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_function_call_case_insensitive() {
        // PowerShell is case-insensitive
        let result = eval_str("function Add($a, $b) { $a + $b }\nadd 5 10").unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_function_call_mixed_case() {
        // Test with mixed case variations
        let result = eval_str("function MyFunc($x) { $x * 2 }\nmYfUnC 21").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_string_comparison_case_insensitive() {
        // PowerShell string comparisons are case-insensitive by default
        let result = eval_str("\"hello\" -eq \"HELLO\"").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_string_comparison_mixed_case() {
        let result = eval_str("\"PowerShell\" -eq \"powershell\"").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_string_not_equal_case_insensitive() {
        let result = eval_str("\"hello\" -ne \"WORLD\"").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_variable_case_insensitive() {
        // Variables should also be case-insensitive
        let result = eval_str("$MyVar = 42\n$myvar").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    // Week 8: Scope qualifier tests
    #[test]
    fn test_global_scope_variable() {
        let result = eval_str("$global:x = 5\n$global:x").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_global_scope_from_nested_scope() {
        let result = eval_str(
            r#"
            $global:x = 5
            function Test {
                $global:x
            }
            Test
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_global_scope_modification_from_function() {
        let result = eval_str(
            r#"
            $global:counter = 0
            function Increment {
                $global:counter = $global:counter + 1
            }
            Increment
            Increment
            $global:counter
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_local_scope_variable() {
        let result = eval_str(
            r#"
            $x = 1
            function Test {
                $local:x = 2
                $local:x
            }
            Test
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_local_vs_global_scope() {
        let result = eval_str(
            r#"
            $x = 100
            function Test {
                $local:x = 200
                $local:x + $global:x
            }
            Test
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(300.0));
    }

    #[test]
    fn test_week8_success_criteria() {
        // From ROADMAP Week 8 success criteria
        let result = eval_str(
            r#"
            $global:x = 5
            function Test {
                $local:y = 10
                $x + $y
            }
            Test
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_script_scope_variable() {
        let result = eval_str("$script:z = 100\n$script:z").unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_scope_qualifier_case_insensitive() {
        let result = eval_str("$GLOBAL:x = 42\n$global:x").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_mixed_scope_qualifiers() {
        let result = eval_str(
            r#"
            $global:a = 1
            $local:b = 2
            $script:c = 3
            $global:a + $local:b + $script:c
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_script_block_creation() {
        let result = eval_str("{ 5 + 10 }").unwrap();
        assert!(matches!(result, Value::ScriptBlock(_)));
    }

    #[test]
    fn test_script_block_with_variable() {
        let result = eval_str(
            r#"
            $x = 10
            $sb = { $x + 5 }
            $sb
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::ScriptBlock(_)));
    }

    #[test]
    fn test_script_block_execution_with_underscore() {
        let mut evaluator = Evaluator::new();

        // Parse a script block
        let mut lexer = Lexer::new("{ $_ + 10 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        // Evaluate to get the script block value
        let result = evaluator.eval(program).unwrap();

        // Extract the script block
        if let Value::ScriptBlock(sb) = result {
            // Execute it with $_ = 5
            let execution_result = evaluator
                .execute_script_block(&sb, Value::Number(5.0))
                .unwrap();
            assert_eq!(execution_result, Value::Number(15.0));
        } else {
            panic!("Expected script block value");
        }
    }

    #[test]
    fn test_script_block_with_comparison() {
        let mut evaluator = Evaluator::new();

        // Parse a script block with comparison
        let mut lexer = Lexer::new("{ $_ -gt 5 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        // Evaluate to get the script block value
        let result = evaluator.eval(program).unwrap();

        // Extract the script block
        if let Value::ScriptBlock(sb) = result {
            // Test with value > 5
            let result1 = evaluator
                .execute_script_block(&sb, Value::Number(10.0))
                .unwrap();
            assert_eq!(result1, Value::Boolean(true));

            // Test with value <= 5
            let result2 = evaluator
                .execute_script_block(&sb, Value::Number(3.0))
                .unwrap();
            assert_eq!(result2, Value::Boolean(false));
        } else {
            panic!("Expected script block value");
        }
    }

    #[test]
    fn test_script_block_with_string_operation() {
        let mut evaluator = Evaluator::new();

        // Parse a script block with string interpolation
        let mut lexer = Lexer::new(r#"{ "Value: $_" }"#);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        // Evaluate to get the script block value
        let result = evaluator.eval(program).unwrap();

        // Extract the script block
        if let Value::ScriptBlock(sb) = result {
            let result = evaluator
                .execute_script_block(&sb, Value::Number(42.0))
                .unwrap();
            assert_eq!(result, Value::String("Value: 42".to_string()));
        } else {
            panic!("Expected script block value");
        }
    }

    #[test]
    fn test_week9_success_criteria() {
        // Week 9 Success Criteria from ROADMAP.md:
        // $filter = { $_ -gt 5 }
        // Can pass script blocks to cmdlets

        let result = eval_str(
            r#"
            $filter = { $_ -gt 5 }
            $filter
            "#,
        )
        .unwrap();

        // Verify we can create and assign script blocks
        assert!(matches!(result, Value::ScriptBlock(_)));
    }

    // Week 10: Hashtable tests
    #[test]
    fn test_empty_hashtable() {
        let result = eval_str("@{}").unwrap();
        match result {
            Value::Object(map) => {
                assert_eq!(map.len(), 0);
            }
            _ => panic!("Expected Object value"),
        }
    }

    #[test]
    fn test_hashtable_creation() {
        let result = eval_str("@{Name=\"John\"; Age=30}").unwrap();
        match result {
            Value::Object(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("Name"), Some(&Value::String("John".to_string())));
                assert_eq!(map.get("Age"), Some(&Value::Number(30.0)));
            }
            _ => panic!("Expected Object value"),
        }
    }

    #[test]
    fn test_hashtable_property_access() {
        let result = eval_str(
            r#"
            $obj = @{Name="John"; Age=30}
            $obj.Name
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::String("John".to_string()));
    }

    #[test]
    fn test_hashtable_multiple_property_access() {
        let mut evaluator = Evaluator::new();

        // Create hashtable and access multiple properties
        eval_str_with_evaluator(
            &mut evaluator,
            r#"
            $person = @{Name="Alice"; Age=25; City="NYC"}
            "#,
        )
        .unwrap();

        let name = eval_str_with_evaluator(&mut evaluator, "$person.Name").unwrap();
        assert_eq!(name, Value::String("Alice".to_string()));

        let age = eval_str_with_evaluator(&mut evaluator, "$person.Age").unwrap();
        assert_eq!(age, Value::Number(25.0));

        let city = eval_str_with_evaluator(&mut evaluator, "$person.City").unwrap();
        assert_eq!(city, Value::String("NYC".to_string()));
    }

    #[test]
    fn test_week10_success_criteria() {
        // Week 10 Success Criteria from ROADMAP.md:
        // $obj = @{Name="John"; Age=30}
        // $obj.Name  # "John"
        // $obj.Age   # 30

        let mut evaluator = Evaluator::new();

        eval_str_with_evaluator(&mut evaluator, r#"$obj = @{Name="John"; Age=30}"#).unwrap();

        let name = eval_str_with_evaluator(&mut evaluator, "$obj.Name").unwrap();
        assert_eq!(name, Value::String("John".to_string()));

        let age = eval_str_with_evaluator(&mut evaluator, "$obj.Age").unwrap();
        assert_eq!(age, Value::Number(30.0));
    }

    // Helper function for tests that need to maintain state
    fn eval_str_with_evaluator(evaluator: &mut Evaluator, input: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| e.to_string())?;
        evaluator.eval(program).map_err(|e| e.to_string())
    }
}

#[test]
fn test_array_literal() {
    let mut evaluator = Evaluator::new();
    let arr_expr = Expression::Array(vec![
        Expression::Literal(Literal::Number(1.0)),
        Expression::Literal(Literal::Number(2.0)),
        Expression::Literal(Literal::Number(3.0)),
    ]);

    let result = evaluator.eval_expression(arr_expr).unwrap();
    match result {
        Value::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        }
        _ => panic!("Expected array value"),
    }
}

#[test]
fn test_empty_array() {
    let mut evaluator = Evaluator::new();
    let arr_expr = Expression::Array(vec![]);

    let result = evaluator.eval_expression(arr_expr).unwrap();
    match result {
        Value::Array(items) => {
            assert_eq!(items.len(), 0);
        }
        _ => panic!("Expected array value"),
    }
}
