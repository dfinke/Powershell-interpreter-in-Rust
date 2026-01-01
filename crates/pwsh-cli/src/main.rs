use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;
use std::io::{self, Write};

/// Check if input appears to be incomplete (has unclosed braces/parens/brackets)
fn is_input_incomplete(input: &str) -> bool {
    let mut brace_count = 0;
    let mut paren_count = 0;
    let mut bracket_count = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '`' => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => brace_count -= 1,
            '(' if !in_string => paren_count += 1,
            ')' if !in_string => paren_count -= 1,
            '[' if !in_string => bracket_count += 1,
            ']' if !in_string => bracket_count -= 1,
            _ => {}
        }
    }

    // Input is incomplete if there are unclosed braces, parens, brackets, or strings
    in_string || brace_count > 0 || paren_count > 0 || bracket_count > 0
}

fn main() {
    println!("PowerShell Interpreter - Week 6 MVP");
    println!("Object Pipeline with 5 Cmdlets!");
    println!(
        "Available cmdlets: Write-Output, Get-Process, Where-Object, Select-Object, ForEach-Object"
    );
    println!("Type 'exit' to quit.\n");

    // Create evaluator and register all cmdlets
    let mut evaluator = Evaluator::new();
    pwsh_cmdlets::register_all(evaluator.registry_mut());

    loop {
        print!("PS> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        // Accumulate multiline input if needed
        let mut accumulated_input = input.to_string();
        while is_input_incomplete(&accumulated_input) {
            print!(">> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            accumulated_input.push('\n');
            accumulated_input.push_str(line.trim_end());
        }

        // Lex, Parse, and Evaluate the input
        let mut lexer = Lexer::new(&accumulated_input);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(program) => match evaluator.eval(program) {
                        Ok(value) => {
                            // Handle arrays by printing each element
                            match value {
                                pwsh_runtime::Value::Array(items) => {
                                    for item in items {
                                        if item != pwsh_runtime::Value::Null {
                                            println!("{}", item);
                                        }
                                    }
                                }
                                pwsh_runtime::Value::Null => {
                                    // Don't print null values
                                }
                                _ => {
                                    println!("{}", value);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Runtime error: {}\n", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse error: {}\n", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Lexer error: {}\n", e);
            }
        }
    }
}
