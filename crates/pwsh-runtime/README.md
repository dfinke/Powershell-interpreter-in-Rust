# PowerShell Runtime

The runtime module provides the execution environment for PowerShell code, including value representation, scope management, and expression/statement evaluation.

## Overview

The runtime is responsible for:
- **Value System**: Representing PowerShell values (numbers, strings, booleans, objects, arrays)
- **Scope Management**: Managing variable storage with proper scoping rules
- **Evaluation**: Executing AST nodes to produce results
- **Error Handling**: Providing detailed runtime error messages

## Components

### Value (`value.rs`)

The `Value` enum represents all PowerShell values:

```rust
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}
```

**Key Features:**
- Type conversions (`to_number()`, `to_bool()`)
- Display formatting via `Display` trait
- Property access for objects
- PowerShell-style truthiness rules

**Example:**
```rust
let value = Value::Number(42.0);
assert_eq!(value.to_string(), "42");
assert!(value.to_bool());
```

### Scope (`scope.rs`)

Variable storage with nested scope support:

```rust
pub struct Scope {
    variables: HashMap<String, Value>,
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}
```

**Key Features:**
- Global scope (always present)
- Nested scopes for blocks and functions
- Variable shadowing
- Lookup from innermost to outermost scope

**Example:**
```rust
let mut stack = ScopeStack::new();
stack.set_variable("x", Value::Number(5.0));
stack.push_scope();
stack.set_variable("y", Value::Number(10.0));
assert_eq!(stack.get_variable("x"), Some(Value::Number(5.0)));
stack.pop_scope();
assert_eq!(stack.get_variable("y"), None);
```

### Evaluator (`evaluator.rs`)

The main evaluation engine:

```rust
pub struct Evaluator {
    scope: ScopeStack,
}
```

**Supported Operations:**
- ✅ Literals (numbers, strings, booleans, null)
- ✅ Variables (assignment and reference)
- ✅ Binary operations (+, -, *, /, %, -eq, -ne, -gt, -lt, -ge, -le)
- ✅ Unary operations (-, !)
- ✅ If/else statements
- ✅ String interpolation
- ✅ Member access (object.property)
- ✅ Nested scopes

**Example:**
```rust
use pwsh_runtime::Evaluator;
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;

let mut lexer = Lexer::new("$x = 5\n$y = 10\n$x + $y");
let tokens = lexer.tokenize().unwrap();
let mut parser = Parser::new(tokens);
let program = parser.parse().unwrap();

let mut evaluator = Evaluator::new();
let result = evaluator.eval(program).unwrap();
// result is Value::Number(15.0)
```

### Error Handling (`error.rs`)

Comprehensive error types:

```rust
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch { expected: String, got: String, operation: String },
    DivisionByZero,
    InvalidOperation(String),
    ReturnOutsideFunction,
    InvalidPropertyAccess(String),
}
```

**Example:**
```rust
let result = eval_str("$undefined");
assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
```

## Usage

### Basic Evaluation

```rust
use pwsh_runtime::{Evaluator, Value};

let mut evaluator = Evaluator::new();

// Parse and evaluate PowerShell code
let program = parse("$x = 42"); // Using your parser
let result = evaluator.eval(program).unwrap();
assert_eq!(result, Value::Number(42.0));
```

### Working with Variables

```rust
// Assignment
eval_str("$name = \"Alice\"");

// String interpolation
let result = eval_str("\"Hello $name\"");
assert_eq!(result, Value::String("Hello Alice".to_string()));
```

### Arithmetic and Comparisons

```rust
// Arithmetic
assert_eq!(eval_str("5 + 3"), Value::Number(8.0));
assert_eq!(eval_str("10 * 2"), Value::Number(20.0));

// Comparisons
assert_eq!(eval_str("5 -gt 3"), Value::Boolean(true));
assert_eq!(eval_str("10 -eq 10"), Value::Boolean(true));
```

### Control Flow

```rust
// If statement
let result = eval_str("if (5 -gt 3) { 100 }");
assert_eq!(result, Value::Number(100.0));

// If-else statement
let result = eval_str("if (false) { 1 } else { 2 }");
assert_eq!(result, Value::Number(2.0));
```

## Design Decisions

### Value Representation

PowerShell uses **objects** as its fundamental data type. Our `Value` enum supports:
- Primitive types (numbers, strings, booleans)
- Objects with properties
- Arrays
- Null

This provides a foundation for the object pipeline while keeping the initial implementation simple.

### Scope Management

The `ScopeStack` implements PowerShell's scoping rules:
1. Variables are looked up from innermost to outermost scope
2. Assignment updates existing variables or creates new ones in current scope
3. The global scope cannot be popped
4. Nested scopes support blocks and functions

### Type Conversions

PowerShell is dynamically typed with automatic conversions:
- `to_number()`: Converts strings, booleans to numbers
- `to_bool()`: PowerShell truthiness (0, empty string, null → false)
- Implicit conversions in operators (e.g., string concatenation)

### Error Handling

All operations return `Result<Value, RuntimeError>` for proper error propagation. Errors include context about what went wrong and where.

## Testing

The runtime has comprehensive test coverage:

```bash
# Run runtime tests
cargo test -p pwsh-runtime

# Run all tests
cargo test --all
```

**Test Coverage:**
- Value conversions (to_string, to_bool, to_number)
- Scope operations (push, pop, get, set, shadowing)
- Evaluator (37 tests covering all operations)
- Error cases (division by zero, undefined variables, type mismatches)

## Performance Considerations

### Current Implementation
- HashMap-based variable storage (O(1) lookup)
- Cloning values for scope operations
- Simple recursive evaluation

### Future Optimizations
- String interning for variable names
- Copy-on-write for large values
- Bytecode compilation for repeated execution
- Value pooling to reduce allocations

## Limitations (Current Phase)

The following are not yet implemented:
- [ ] Function definitions and calls
- [ ] Pipeline execution
- [ ] Script blocks as first-class values
- [ ] Advanced operators (match, contains, etc.)
- [ ] Arrays and hashtables (syntax exists, but no array operations)
- [ ] Loops (for, foreach, while)
- [ ] Try/catch error handling

These will be implemented in subsequent phases according to the roadmap.

## Integration

The runtime integrates with:
- **pwsh-lexer**: Provides tokenization
- **pwsh-parser**: Provides AST
- **pwsh-cli**: Uses evaluator for REPL

```rust
// Complete pipeline: Source → Tokens → AST → Evaluation
let mut lexer = Lexer::new(source);
let tokens = lexer.tokenize()?;
let mut parser = Parser::new(tokens);
let program = parser.parse()?;
let mut evaluator = Evaluator::new();
let result = evaluator.eval(program)?;
```

## Examples

See `examples/week5_success_criteria.ps1` for a comprehensive test script demonstrating all implemented features.

## Development

### Adding New Value Types

To add a new value type:
1. Add variant to `Value` enum
2. Implement conversions in `to_string()`, `to_bool()`, `to_number()`
3. Add tests
4. Update evaluator to handle the type

### Adding New Operations

To add a new operation:
1. Add to `BinaryOperator` or `UnaryOperator` in parser
2. Implement evaluation in `eval_binary_op()` or `eval_unary_op()`
3. Add tests for the operation
4. Update documentation

## References

- [PowerShell Language Specification](https://docs.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-01)
- [ROADMAP.md](../../ROADMAP.md) - Implementation plan
- [TECHNICAL_DESIGN.md](../../TECHNICAL_DESIGN.md) - Architecture details
