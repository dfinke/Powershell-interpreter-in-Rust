use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;
use std::io::{self, Write};

fn main() {
    println!("PowerShell Interpreter - Phase 1 REPL");
    println!("Now with Runtime & Evaluator!");
    println!("Type 'exit' to quit.\n");

    let mut evaluator = Evaluator::new();

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
                            // Only print non-null values
                            if value != pwsh_runtime::Value::Null {
                                println!("{}", value);
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
