# PowerShell Interpreter - Technical Design Document

## 1. System Architecture

### 1.1 High-Level Design

The interpreter follows a classic multi-stage architecture:

```
Source Code → Lexer → Tokens → Parser → AST → Evaluator → Result
                                                    ↓
                                              Runtime Environment
                                              (Scope, Cmdlets, Objects)
```

### 1.2 Module Structure (Rust Workspace)

```
powershell-interpreter/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── pwsh-lexer/           # Tokenization
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── token.rs
│   │   │   └── lexer.rs
│   │   └── tests/
│   ├── pwsh-parser/          # Parsing and AST
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ast.rs
│   │   │   ├── parser.rs
│   │   │   └── error.rs
│   │   └── tests/
│   ├── pwsh-runtime/         # Execution environment
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── value.rs
│   │   │   ├── scope.rs
│   │   │   ├── pipeline.rs
│   │   │   └── evaluator.rs
│   │   └── tests/
│   ├── pwsh-cmdlets/         # Built-in cmdlets
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── core/        # Core cmdlets
│   │   │   ├── utility/     # Utility cmdlets
│   │   │   └── management/  # Management cmdlets
│   │   └── tests/
│   └── pwsh-cli/             # REPL and CLI interface
│       ├── src/
│       │   ├── main.rs
│       │   ├── repl.rs
│       │   └── executor.rs
│       └── tests/
├── examples/                  # Example PowerShell scripts
├── docs/                      # Documentation
└── benches/                   # Performance benchmarks
```

## 2. Detailed Component Design

### 2.1 Lexer (pwsh-lexer)

**Purpose:** Convert source text into a stream of tokens

**Core Types:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    
    // Identifiers and Variables
    Identifier(String),
    Variable(String),           // $varName
    
    // Operators
    Plus,                       // +
    Minus,                      // -
    Multiply,                   // *
    Divide,                     // /
    Equal,                      // -eq
    NotEqual,                   // -ne
    Greater,                    // -gt
    Less,                       // -lt
    GreaterOrEqual,             // -ge
    LessOrEqual,                // -le
    
    // Keywords
    If,
    Else,
    ElseIf,
    Function,
    Return,
    
    // Syntax
    LeftParen,                  // (
    RightParen,                 // )
    LeftBrace,                  // {
    RightBrace,                 // }
    LeftBracket,                // [
    RightBracket,               // ]
    Comma,                      // ,
    Dot,                        // .
    Pipeline,                   // |
    Assignment,                 // =
    Semicolon,                  // ;
    Newline,
    
    // Special
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self;
    pub fn next_token(&mut self) -> Result<Token, LexError>;
    pub fn peek_token(&self) -> Result<Token, LexError>;
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError>;
}
```

**Key Features:**
- String interpolation handling (`"Hello $name"`)
- Multi-line string support
- Comment handling (`# comment`)
- Position tracking for error reporting

### 2.2 Parser (pwsh-parser)

**Purpose:** Build Abstract Syntax Tree from tokens

**AST Definition:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Assignment {
        variable: String,
        value: Expression,
    },
    FunctionDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Block,
    },
    If {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    Return(Expression),
    Pipeline(Pipeline),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Argument>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    ScriptBlock(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub stages: Vec<PipelineStage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStage {
    pub command: Expression,
    pub is_beginning: bool,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self;
    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError>;
    fn parse_statement(&mut self) -> Result<Statement, ParseError>;
    fn parse_expression(&mut self) -> Result<Expression, ParseError>;
    fn parse_pipeline(&mut self) -> Result<Pipeline, ParseError>;
}
```

**Parsing Strategy:**
- Recursive descent parser
- Pratt parsing for expressions (operator precedence)
- Error recovery with synchronization points
- Rich error messages with context

### 2.3 Runtime (pwsh-runtime)

**Purpose:** Execution environment for PowerShell code

**Value System:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(Object),
    Array(Vec<Value>),
    Hashtable(HashMap<String, Value>),
    ScriptBlock(Block),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String;
    pub fn to_bool(&self) -> bool;
    pub fn to_number(&self) -> Option<f64>;
    pub fn get_property(&self, name: &str) -> Option<Value>;
    pub fn set_property(&mut self, name: &str, value: Value);
}

#[derive(Debug, Clone)]
pub struct Object {
    type_name: String,
    properties: HashMap<String, Value>,
    methods: HashMap<String, Box<dyn Fn(&Object, Vec<Value>) -> Value>>,
}
```

**Scope Management:**

```rust
pub struct Scope {
    variables: HashMap<String, Value>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self;
    pub fn with_parent(parent: Scope) -> Self;
    pub fn get(&self, name: &str) -> Option<&Value>;
    pub fn set(&mut self, name: &str, value: Value);
    pub fn define(&mut self, name: &str, value: Value);
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn push_scope(&mut self);
    pub fn pop_scope(&mut self);
    pub fn get_variable(&self, name: &str) -> Option<&Value>;
    pub fn set_variable(&mut self, name: &str, value: Value);
}
```

**Evaluator:**

```rust
pub struct Evaluator {
    scope: ScopeStack,
    cmdlet_registry: CmdletRegistry,
}

impl Evaluator {
    pub fn new() -> Self;
    pub fn eval(&mut self, statements: Vec<Statement>) -> Result<Value, RuntimeError>;
    pub fn eval_statement(&mut self, stmt: Statement) -> Result<Value, RuntimeError>;
    pub fn eval_expression(&mut self, expr: Expression) -> Result<Value, RuntimeError>;
    pub fn eval_pipeline(&mut self, pipeline: Pipeline) -> Result<Vec<Value>, RuntimeError>;
}
```

**Pipeline Execution:**

```rust
pub struct PipelineExecutor {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl PipelineExecutor {
    pub fn execute(&mut self, input: Vec<Value>) -> Result<Vec<Value>, RuntimeError> {
        let mut data = input;
        
        for stage in &mut self.stages {
            data = stage.process(data)?;
        }
        
        Ok(data)
    }
}

pub trait PipelineStage {
    fn process(&mut self, input: Vec<Value>) -> Result<Vec<Value>, RuntimeError>;
}
```

### 2.4 Cmdlets (pwsh-cmdlets)

**Purpose:** Built-in PowerShell commands

**Cmdlet Interface:**

```rust
pub trait Cmdlet: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, params: Parameters, input: Option<Vec<Value>>) 
        -> Result<Vec<Value>, CmdletError>;
    fn supports_pipeline(&self) -> bool { false }
}

pub struct Parameters {
    named: HashMap<String, Value>,
    positional: Vec<Value>,
}

pub struct CmdletRegistry {
    cmdlets: HashMap<String, Box<dyn Cmdlet>>,
}

impl CmdletRegistry {
    pub fn new() -> Self;
    pub fn register<C: Cmdlet + 'static>(&mut self, cmdlet: C);
    pub fn get(&self, name: &str) -> Option<&Box<dyn Cmdlet>>;
}
```

**Example Cmdlet Implementation:**

```rust
pub struct WriteOutput;

impl Cmdlet for WriteOutput {
    fn name(&self) -> &str {
        "Write-Output"
    }
    
    fn execute(&self, params: Parameters, input: Option<Vec<Value>>) 
        -> Result<Vec<Value>, CmdletError> {
        
        let values = if let Some(positional) = params.positional.first() {
            vec![positional.clone()]
        } else if let Some(input) = input {
            input
        } else {
            vec![]
        };
        
        // Pass through for pipeline
        Ok(values)
    }
    
    fn supports_pipeline(&self) -> bool {
        true
    }
}

pub struct WhereObject;

impl Cmdlet for WhereObject {
    fn name(&self) -> &str {
        "Where-Object"
    }
    
    fn execute(&self, params: Parameters, input: Option<Vec<Value>>) 
        -> Result<Vec<Value>, CmdletError> {
        
        let input = input.ok_or(CmdletError::NoPipelineInput)?;
        let script_block = params.positional.first()
            .ok_or(CmdletError::MissingParameter("FilterScript"))?;
        
        let mut results = Vec::new();
        
        for value in input {
            // Execute script block with $_ bound to current value
            if self.eval_filter(script_block, &value)? {
                results.push(value);
            }
        }
        
        Ok(results)
    }
    
    fn supports_pipeline(&self) -> bool {
        true
    }
}
```

### 2.5 REPL (pwsh-cli)

**Purpose:** Interactive shell interface

```rust
pub struct Repl {
    evaluator: Evaluator,
    history: Vec<String>,
    prompt: String,
}

impl Repl {
    pub fn new() -> Self;
    
    pub fn run(&mut self) -> Result<(), Error> {
        let mut rl = Editor::<()>::new()?;
        
        loop {
            let readline = rl.readline(&self.prompt);
            
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    
                    match self.eval_line(&line) {
                        Ok(value) => {
                            if value != Value::Null {
                                println!("{}", value.to_string());
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn eval_line(&mut self, line: &str) -> Result<Value, Error> {
        let mut lexer = Lexer::new(line);
        let tokens = lexer.tokenize()?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        self.evaluator.eval(ast)
    }
}
```

## 3. Implementation Details

### 3.1 String Interpolation

PowerShell strings support variable interpolation:

```powershell
$name = "World"
Write-Output "Hello $name"  # Output: Hello World
```

**Implementation Approach:**

```rust
fn parse_interpolated_string(&mut self, content: &str) -> Expression {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = content.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '$' {
            if !current.is_empty() {
                parts.push(Expression::Literal(Literal::String(current.clone())));
                current.clear();
            }
            
            // Parse variable name
            let var_name = self.parse_var_name(&mut chars);
            parts.push(Expression::Variable(var_name));
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        parts.push(Expression::Literal(Literal::String(current)));
    }
    
    Expression::StringInterpolation(parts)
}
```

### 3.2 Object Pipeline

The object pipeline is PowerShell's killer feature:

```powershell
Get-Process | Where-Object {$_.CPU -gt 10} | Select-Object Name, CPU
```

**Execution Flow:**

1. `Get-Process` produces list of process objects
2. Each object flows through `Where-Object` filter
3. Filtered objects flow through `Select-Object` projection
4. Final projected objects returned

**Implementation:**

```rust
pub fn eval_pipeline(&mut self, pipeline: Pipeline) -> Result<Vec<Value>, RuntimeError> {
    let mut current_input: Option<Vec<Value>> = None;
    
    for (i, stage) in pipeline.stages.iter().enumerate() {
        let result = match &stage.command {
            Expression::FunctionCall { name, arguments } => {
                if let Some(cmdlet) = self.cmdlet_registry.get(name) {
                    let params = self.eval_arguments(arguments)?;
                    cmdlet.execute(params, current_input)?
                } else {
                    return Err(RuntimeError::CmdletNotFound(name.clone()));
                }
            }
            _ => return Err(RuntimeError::InvalidPipelineStage),
        };
        
        current_input = Some(result);
    }
    
    current_input.ok_or(RuntimeError::EmptyPipeline)
}
```

### 3.3 Parameter Binding

PowerShell supports flexible parameter binding:

```powershell
Get-Process -Name "chrome"     # Named parameter
Get-Process "chrome"           # Positional parameter
Get-Process                    # No parameters
```

**Simple Implementation (MVP):**

```rust
pub fn bind_parameters(
    cmdlet_params: &[ParameterDefinition],
    args: &[Argument],
) -> Result<Parameters, BindingError> {
    let mut named = HashMap::new();
    let mut positional = Vec::new();
    
    for arg in args {
        match arg {
            Argument::Named { name, value } => {
                named.insert(name.clone(), value.clone());
            }
            Argument::Positional(value) => {
                positional.push(value.clone());
            }
        }
    }
    
    Ok(Parameters { named, positional })
}
```

### 3.4 Error Handling

Rich error messages are crucial for usability:

```rust
#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch { expected: String, got: String },
    DivisionByZero,
    CmdletNotFound(String),
    InvalidOperation(String),
    PipelineError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "Variable '{}' is not defined", name)
            }
            RuntimeError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            RuntimeError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            RuntimeError::CmdletNotFound(name) => {
                write!(f, "Cmdlet '{}' not found", name)
            }
            RuntimeError::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
            RuntimeError::PipelineError(msg) => {
                write!(f, "Pipeline error: {}", msg)
            }
        }
    }
}
```

## 4. Testing Strategy

### 4.1 Unit Tests

**Lexer Tests:**

```rust
#[test]
fn test_tokenize_variable() {
    let mut lexer = Lexer::new("$myVar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens, vec![Token::Variable("myVar".to_string())]);
}

#[test]
fn test_tokenize_pipeline() {
    let mut lexer = Lexer::new("Get-Process | Where-Object");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1], Token::Pipeline);
}
```

**Parser Tests:**

```rust
#[test]
fn test_parse_assignment() {
    let tokens = vec![
        Token::Variable("x".to_string()),
        Token::Assignment,
        Token::Number(42.0),
    ];
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Statement::Assignment { variable, value } => {
            assert_eq!(variable, "x");
        }
        _ => panic!("Expected assignment"),
    }
}
```

**Runtime Tests:**

```rust
#[test]
fn test_eval_arithmetic() {
    let mut eval = Evaluator::new();
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::Number(5.0))),
        operator: BinaryOperator::Plus,
        right: Box::new(Expression::Literal(Literal::Number(3.0))),
    };
    
    let result = eval.eval_expression(expr).unwrap();
    assert_eq!(result, Value::Number(8.0));
}
```

### 4.2 Integration Tests

```rust
#[test]
fn test_pipeline_execution() {
    let script = r#"
        $data = @(1, 2, 3, 4, 5)
        $data | Where-Object { $_ -gt 2 } | ForEach-Object { $_ * 2 }
    "#;
    
    let result = execute_script(script).unwrap();
    assert_eq!(result, vec![
        Value::Number(6.0),
        Value::Number(8.0),
        Value::Number(10.0),
    ]);
}
```

### 4.3 End-to-End Tests

```rust
#[test]
fn test_full_script() {
    let script = include_str!("../examples/basic_pipeline.ps1");
    let result = execute_script(script);
    assert!(result.is_ok());
}
```

## 5. Performance Considerations

### 5.1 Optimization Opportunities

1. **String Interning**: Reduce memory for repeated strings
2. **AST Caching**: Cache parsed scripts
3. **Pipeline Streaming**: Process objects one at a time
4. **Lazy Evaluation**: Don't compute unless needed
5. **Object Pooling**: Reuse object allocations

### 5.2 Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lexer(c: &mut Criterion) {
    c.bench_function("lexer: simple script", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box("$x = 5; $y = 10; $x + $y"));
            lexer.tokenize()
        });
    });
}

fn bench_pipeline(c: &mut Criterion) {
    c.bench_function("pipeline: filter 1000 objects", |b| {
        b.iter(|| {
            let script = "1..1000 | Where-Object { $_ % 2 -eq 0 }";
            execute_script(black_box(script))
        });
    });
}

criterion_group!(benches, bench_lexer, bench_pipeline);
criterion_main!(benches);
```

## 6. Future Enhancements

### 6.1 Bytecode Compilation

After MVP, consider bytecode compilation:

```
AST → Compiler → Bytecode → VM → Result
```

**Benefits:**
- Faster execution (10-100x)
- Optimization passes
- Smaller memory footprint

### 6.2 JIT Compilation

For hot paths, JIT compilation:

```rust
pub struct JitCompiler {
    hot_paths: HashMap<FunctionId, CompiledFunction>,
}

impl JitCompiler {
    pub fn compile(&mut self, ast: &Block) -> CompiledFunction;
    pub fn should_compile(&self, execution_count: usize) -> bool;
}
```

### 6.3 Parallel Pipeline

```powershell
1..1000 | ForEach-Object -Parallel { $_ * 2 }
```

Use Rayon for data parallelism:

```rust
use rayon::prelude::*;

pub fn execute_parallel_pipeline(&self, input: Vec<Value>) -> Vec<Value> {
    input.par_iter()
        .map(|v| self.process(v))
        .collect()
}
```

## 7. Development Tools

### 7.1 Debugging Support

```rust
pub struct Debugger {
    breakpoints: HashSet<Location>,
    step_mode: bool,
}

impl Debugger {
    pub fn set_breakpoint(&mut self, line: usize);
    pub fn step_into(&mut self);
    pub fn step_over(&mut self);
    pub fn continue_execution(&mut self);
}
```

### 7.2 Profiling

```rust
pub struct Profiler {
    call_counts: HashMap<String, usize>,
    execution_times: HashMap<String, Duration>,
}

impl Profiler {
    pub fn record_call(&mut self, function: &str, duration: Duration);
    pub fn report(&self) -> ProfileReport;
}
```

## 8. Security Considerations

1. **Script Injection**: Validate all input
2. **Resource Limits**: Prevent infinite loops, excessive memory
3. **Sandboxing**: Restrict file system access (future)
4. **Code Signing**: Verify script authenticity (future)

## 9. Documentation Plan

1. **API Documentation**: rustdoc comments
2. **User Guide**: How to use the interpreter
3. **Developer Guide**: How to contribute
4. **Architecture Guide**: This document
5. **PowerShell Compatibility**: What's supported/not supported

## 10. Conclusion

This technical design provides a solid foundation for building a PowerShell interpreter in Rust. The modular architecture allows for incremental development, testing, and optimization while maintaining code quality and performance.

Key success factors:
- Clean separation of concerns
- Comprehensive testing
- Iterative development
- Performance-conscious design
- Rich error messages
- Object-based pipeline as core feature
