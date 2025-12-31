use pwsh_lexer::{Lexer, Token};
use std::io::{self, Write};

fn main() {
    println!("PowerShell Interpreter - Phase 0 REPL");
    println!("This REPL tokenizes your input and displays the tokens.");
    println!("Type 'exit' to quit.\n");

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

        // Tokenize the input
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => {
                println!("Tokens:");
                for (i, located_token) in tokens.iter().enumerate() {
                    if located_token.token != Token::Eof {
                        println!(
                            "  [{}] {} (line: {}, col: {})",
                            i,
                            located_token.token,
                            located_token.position.line,
                            located_token.position.column
                        );
                    }
                }
                println!();
            }
            Err(e) => {
                eprintln!("Lexer error: {}\n", e);
            }
        }
    }
}
