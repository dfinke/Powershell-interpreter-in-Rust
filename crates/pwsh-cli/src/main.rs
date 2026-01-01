use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;
use std::io::{self, Write};

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

        // Lex, Parse, and Evaluate the input
        let mut lexer = Lexer::new(input);
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
