# PowerShell Lexer Documentation

## Overview

The `pwsh-lexer` crate provides tokenization (lexical analysis) for PowerShell code. It converts raw source text into a stream of tokens that can be consumed by the parser.

## Architecture

The lexer follows a single-pass design with position tracking for accurate error reporting:

```
Source Text → Lexer → Token Stream (with positions)
```

## Token Types

### Literals
- `String(String)` - String literals with single or double quotes
- `Number(f64)` - Numeric literals (integers and floats)
- `Boolean(bool)` - Boolean literals (`true`, `false`)

### Identifiers and Variables
- `Identifier(String)` - Function names, cmdlets, keywords
- `Variable(String)` - Variables starting with `$`

### Operators

**Arithmetic:**
- `Plus` (+)
- `Minus` (-)
- `Multiply` (*)
- `Divide` (/)
- `Modulo` (%)

**Comparison:**
- `Equal` (-eq)
- `NotEqual` (-ne)
- `Greater` (-gt)
- `Less` (-lt)
- `GreaterOrEqual` (-ge)
- `LessOrEqual` (-le)

### Keywords
- `If`
- `Else`
- `ElseIf`
- `Function`
- `Return`

### Syntax Elements
- `LeftParen`, `RightParen` - `()` for grouping
- `LeftBrace`, `RightBrace` - `{}` for blocks
- `LeftBracket`, `RightBracket` - `[]` for arrays/types
- `Comma` - `,` separator
- `Dot` - `.` member access
- `Pipeline` - `|` for pipelines
- `Assignment` - `=` for assignments
- `Semicolon` - `;` statement separator
- `Newline` - Line breaks (significant in PowerShell)
- `Eof` - End of input

## Usage

### Basic Tokenization

```rust
use pwsh_lexer::Lexer;

let mut lexer = Lexer::new("$x = 5");
let tokens = lexer.tokenize().unwrap();

for token in tokens {
    println!("{}", token.token);
}
```

### Token-by-Token Processing

```rust
use pwsh_lexer::Lexer;

let mut lexer = Lexer::new("$x = 5");

loop {
    let located_token = lexer.next_token().unwrap();
    println!("Token: {} at line {}, col {}", 
        located_token.token,
        located_token.position.line,
        located_token.position.column
    );
    
    if located_token.token == Token::Eof {
        break;
    }
}
```

## Position Tracking

Every token includes position information for error reporting:

```rust
pub struct Position {
    pub line: usize,    // 1-based line number
    pub column: usize,  // 1-based column number
}

pub struct LocatedToken {
    pub token: Token,
    pub position: Position,
}
```

## Error Handling

The lexer provides detailed error messages:

```rust
pub enum LexError {
    UnexpectedCharacter { ch: char, position: Position },
    UnterminatedString { position: Position },
    InvalidNumber { text: String, position: Position },
    InvalidToken { text: String, position: Position },
}
```

Example:
```rust
let mut lexer = Lexer::new("\"unterminated string");
match lexer.tokenize() {
    Ok(tokens) => { /* ... */ },
    Err(e) => eprintln!("Lexer error: {}", e),
}
```

## Features

### String Literals
Supports both single and double-quoted strings:
```powershell
"double quotes"
'single quotes'
```

### Escape Sequences
Handles common escape sequences in double-quoted strings:
- `\n` - newline
- `\r` - carriage return
- `\t` - tab
- `\\` - backslash
- `\"` - double quote
- `\'` - single quote

### Comments
Single-line comments starting with `#`:
```powershell
$x = 5  # this is a comment
```

### Cmdlet Names
Recognizes PowerShell cmdlet naming convention (Verb-Noun):
```powershell
Get-Process
Where-Object
Select-Object
```

### Comparison Operators
PowerShell-style comparison operators:
```powershell
$x -eq 5   # equal
$x -ne 10  # not equal
$x -gt 3   # greater than
$x -lt 7   # less than
$x -ge 5   # greater or equal
$x -le 10  # less or equal
```

## Examples

### Variable Assignment
```powershell
$x = 5
$name = "John"
```

Tokens:
```
Variable($x), Assignment, Number(5)
Variable($name), Assignment, String("John")
```

### Pipeline Expression
```powershell
Get-Process | Where-Object
```

Tokens:
```
Identifier(Get-Process), Pipeline, Identifier(Where-Object)
```

### Complex Expression
```powershell
if ($x -eq 5) { Write-Output "Five" }
```

Tokens:
```
If, LeftParen, Variable($x), Equal, Number(5), RightParen,
LeftBrace, Identifier(Write-Output), String("Five"), RightBrace
```

## Implementation Details

### Character-by-Character Processing
The lexer processes input character by character with lookahead capability:

- `peek()` - Look at current character without consuming
- `peek_ahead(n)` - Look ahead n characters
- `advance()` - Consume current character and move to next

### Whitespace Handling
- Spaces and tabs are skipped
- Newlines are significant and tokenized
- Comments are treated as whitespace

### Number Parsing
Supports integers and floating-point numbers:
```powershell
42      # integer
3.5     # float
0.5     # decimal
```

### Operator Parsing
The lexer distinguishes between:
- Minus operator (`-`) in expressions like `5 - 3`
- Comparison operators (`-eq`, `-ne`, etc.) in conditions

## Test Coverage

The lexer includes 27 comprehensive tests covering:
- Basic token types (variables, numbers, strings)
- All operators (arithmetic and comparison)
- Keywords and identifiers
- Comments and whitespace
- Error conditions
- Position tracking
- Edge cases

Run tests with:
```bash
cargo test -p pwsh-lexer
```

## Performance Considerations

- Single-pass design for efficiency
- No backtracking required
- Minimal memory allocation
- Character-level processing with lookahead

## Future Enhancements

Planned for Week 2 (Phase 0):
- String interpolation support (`"Hello $name"`)
- Multi-line string handling
- Enhanced error recovery
- More comprehensive operator support

## Related Modules

- `pwsh-parser` - Consumes tokens to build AST
- `pwsh-runtime` - Executes parsed code
- `pwsh-cli` - Interactive REPL interface
