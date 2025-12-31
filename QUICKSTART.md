# PowerShell Interpreter - Quick Start Guide

## Getting Started in 30 Minutes

This guide will help you set up the PowerShell interpreter project and create your first working prototype.

## Prerequisites

- Rust (latest stable version)
- Git
- A code editor (VS Code with rust-analyzer recommended)

## Step 1: Project Setup (5 minutes)

### Initialize Rust Workspace

```bash
# Create project directory
mkdir powershell-interpreter
cd powershell-interpreter

# Initialize git
git init

# Create workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/pwsh-lexer",
    "crates/pwsh-parser",
    "crates/pwsh-runtime",
    "crates/pwsh-cmdlets",
    "crates/pwsh-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT"

[workspace.dependencies]
anyhow = "1.0"
thiserror = "1.0"
EOF

# Create .gitignore
cat > .gitignore << 'EOF'
/target
Cargo.lock
*.swp
*.swo
*~
.DS_Store
EOF
```

### Create Module Structure

```bash
# Create crate directories
mkdir -p crates/pwsh-lexer/src
mkdir -p crates/pwsh-parser/src
mkdir -p crates/pwsh-runtime/src
mkdir -p crates/pwsh-cmdlets/src
mkdir -p crates/pwsh-cli/src

# Create examples and tests directories
mkdir -p examples
mkdir -p tests
```

## Step 2: Create Lexer (10 minutes)

### Create pwsh-lexer/Cargo.toml

```bash
cat > crates/pwsh-lexer/Cargo.toml << 'EOF'
[package]
name = "pwsh-lexer"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror = { workspace = true }
EOF
```

### Create Basic Lexer

```bash
cat > crates/pwsh-lexer/src/lib.rs << 'EOF'
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    
    // Variables
    Variable(String),
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,      // -eq
    
    // Syntax
    Pipeline,   // |
    Assignment, // =
    
    // Keywords
    Identifier(String),
    
    // Control
    Newline,
    Eof,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer error at position {}: {}", self.position, self.message)
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }
            
            tokens.push(self.next_token()?);
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.current_char().ok_or_else(|| LexError {
            message: "Unexpected end of input".to_string(),
            position: self.position,
        })?;
        
        let token = match ch {
            '$' => {
                self.advance();
                let name = self.read_identifier();
                Token::Variable(name)
            }
            '"' => self.read_string()?,
            '0'..='9' => self.read_number()?,
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                if self.peek_char() == Some('e') && self.peek_ahead(1) == Some('q') {
                    self.advance(); // consume 'e'
                    self.advance(); // consume 'q'
                    Token::Equal
                } else {
                    Token::Minus
                }
            }
            '*' => {
                self.advance();
                Token::Multiply
            }
            '/' => {
                self.advance();
                Token::Divide
            }
            '|' => {
                self.advance();
                Token::Pipeline
            }
            '=' => {
                self.advance();
                Token::Assignment
            }
            '\n' => {
                self.advance();
                Token::Newline
            }
            _ if ch.is_alphabetic() => {
                let id = self.read_identifier();
                Token::Identifier(id)
            }
            _ => {
                return Err(LexError {
                    message: format!("Unexpected character: '{}'", ch),
                    position: self.position,
                });
            }
        };
        
        Ok(token)
    }
    
    fn read_string(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume opening "
        let mut value = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // consume closing "
                return Ok(Token::String(value));
            }
            value.push(ch);
            self.advance();
        }
        
        Err(LexError {
            message: "Unterminated string".to_string(),
            position: self.position,
        })
    }
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut num_str = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let value = num_str.parse::<f64>().map_err(|_| LexError {
            message: format!("Invalid number: {}", num_str),
            position: self.position,
        })?;
        
        Ok(Token::Number(value))
    }
    
    fn read_identifier(&mut self) -> String {
        let mut id = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                id.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        id
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    fn peek_char(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn peek_ahead(&self, n: usize) -> Option<char> {
        if self.position + n < self.input.len() {
            Some(self.input[self.position + n])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_variable() {
        let mut lexer = Lexer::new("$x");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Variable("x".to_string()));
    }
    
    #[test]
    fn test_tokenize_number() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(42.0));
    }
    
    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new(r#""hello""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }
    
    #[test]
    fn test_tokenize_assignment() {
        let mut lexer = Lexer::new("$x = 5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Variable("x".to_string()));
        assert_eq!(tokens[1], Token::Assignment);
        assert_eq!(tokens[2], Token::Number(5.0));
    }
    
    #[test]
    fn test_tokenize_pipeline() {
        let mut lexer = Lexer::new("Get-Process | Where-Object");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[1], Token::Pipeline);
    }
}
EOF
```

## Step 3: Create Simple CLI (10 minutes)

### Create pwsh-cli/Cargo.toml

```bash
cat > crates/pwsh-cli/Cargo.toml << 'EOF'
[package]
name = "pwsh-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "pwsh"
path = "src/main.rs"

[dependencies]
pwsh-lexer = { path = "../pwsh-lexer" }
EOF
```

### Create Basic REPL

```bash
cat > crates/pwsh-cli/src/main.rs << 'EOF'
use pwsh_lexer::{Lexer, Token};
use std::io::{self, Write};

fn main() {
    println!("PowerShell Interpreter v0.1.0");
    println!("Type 'exit' to quit\n");
    
    loop {
        print!("PS> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        
        if input == "exit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        // Tokenize input
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => {
                println!("Tokens:");
                for token in tokens {
                    if token != Token::Eof {
                        println!("  {:?}", token);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        
        println!();
    }
}
EOF
```

## Step 4: Test Your Work (5 minutes)

### Run Tests

```bash
# Test the lexer
cargo test -p pwsh-lexer

# Expected output:
# running 5 tests
# test tests::test_tokenize_assignment ... ok
# test tests::test_tokenize_number ... ok
# test tests::test_tokenize_pipeline ... ok
# test tests::test_tokenize_string ... ok
# test tests::test_tokenize_variable ... ok
```

### Run the REPL

```bash
# Build and run
cargo run -p pwsh-cli

# Try these commands in the REPL:
# PS> $x = 5
# PS> $y = 10
# PS> $x + $y
# PS> "hello world"
# PS> Get-Process | Where-Object
# PS> exit
```

## Next Steps

Congratulations! You now have a working tokenizer and REPL. Here's what to build next:

### Week 1-2: Complete the Parser

1. Create `pwsh-parser` crate
2. Define AST structures
3. Implement recursive descent parser
4. Add parser tests

### Week 3-4: Build the Runtime

1. Create `pwsh-runtime` crate
2. Implement value system
3. Build expression evaluator
4. Add scope management

### Week 5-6: First Cmdlets

1. Create `pwsh-cmdlets` crate
2. Define cmdlet trait
3. Implement `Write-Output`
4. Build pipeline executor
5. Test end-to-end pipeline

## Example Development Workflow

```bash
# 1. Make changes to lexer
vim crates/pwsh-lexer/src/lib.rs

# 2. Run tests
cargo test -p pwsh-lexer

# 3. Run the REPL to manually test
cargo run -p pwsh-cli

# 4. Check code with clippy
cargo clippy

# 5. Format code
cargo fmt

# 6. Commit changes
git add .
git commit -m "Add support for XYZ"
```

## Useful Commands

```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Build optimized release
cargo build --release

# Check code without building
cargo check

# Generate documentation
cargo doc --open

# Run benchmarks (after adding criterion)
cargo bench
```

## Debugging Tips

### Enable Debug Logging

```rust
// In your code
println!("Debug: tokens = {:?}", tokens);
eprintln!("Error: {}", error);
```

### Use Rust Debugger

```bash
# Install lldb or gdb
# VS Code: Install CodeLLDB extension
# Set breakpoints in your code
# Press F5 to start debugging
```

### Print AST

```rust
println!("{:#?}", ast);  // Pretty print with indentation
```

## Common Issues and Solutions

### Issue: Cargo command not found
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Issue: Compilation errors
```bash
# Update Rust to latest version
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### Issue: Tests failing
```bash
# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Crafting Interpreters**: https://craftinginterpreters.com/
- **PowerShell Docs**: https://docs.microsoft.com/powershell/
- **This Project's Docs**: See INVESTIGATION.md and TECHNICAL_DESIGN.md

## Getting Help

1. Read the error messages carefully
2. Check the Rust documentation
3. Look at the test cases for examples
4. Refer to TECHNICAL_DESIGN.md for architecture
5. Ask in Rust community forums

Happy coding! 🚀
