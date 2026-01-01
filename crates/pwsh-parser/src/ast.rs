/// Abstract Syntax Tree definitions for PowerShell
/// A complete PowerShell program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A statement in PowerShell
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// An expression statement
    Expression(Expression),
    /// Variable assignment: $x = value
    Assignment { variable: String, value: Expression },
    /// Function definition
    FunctionDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Block,
    },
    /// If/else conditional
    If {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    /// Return statement
    Return(Option<Expression>),
    /// Pipeline expression
    Pipeline(Pipeline),
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default_value: Option<Expression>,
}

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// An expression in PowerShell
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Variable reference: $varName
    Variable(String),
    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary operation: op operand
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    /// Function/cmdlet call
    Call {
        name: String,
        arguments: Vec<Argument>,
    },
    /// Member access: object.member
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    /// Script block: { statements }
    ScriptBlock(Block),
    /// Hashtable: @{key1=value1; key2=value2}
    Hashtable(Vec<(String, Expression)>),
    /// Array: @(item1, item2, ...)
    Array(Vec<Expression>),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    InterpolatedString(Vec<StringPart>),
    Number(f64),
    Boolean(bool),
    Null,
}

/// Parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %

    // Comparison
    Equal,          // -eq
    NotEqual,       // -ne
    Greater,        // -gt
    Less,           // -lt
    GreaterOrEqual, // -ge
    LessOrEqual,    // -le
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate, // -
    Not,    // !
}

/// Function/cmdlet argument
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// Positional argument
    Positional(Expression),
    /// Named parameter: -Name value
    Named { name: String, value: Expression },
}

/// Pipeline of commands
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub stages: Vec<Expression>,
}
