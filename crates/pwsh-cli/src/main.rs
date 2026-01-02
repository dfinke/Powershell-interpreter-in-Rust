use nu_ansi_term::{Color, Style};
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;
use reedline::{
    ColumnarMenu, DefaultCompleter, FileBackedHistory, Highlighter, Prompt, PromptEditMode,
    PromptHistorySearch, Reedline, ReedlineMenu, Signal, StyledText, ValidationResult, Validator,
};
use std::borrow::Cow;

// --- Validator ---

/// Validates if the input is a complete PowerShell command
struct PowerShellValidator;

impl Validator for PowerShellValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let mut brace_count = 0;
        let mut paren_count = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            match ch {
                '\'' | '"' if !in_string => {
                    in_string = true;
                    string_char = ch;
                }
                ch if in_string && ch == string_char => {
                    // Check for escaped quote (PowerShell uses double quotes for escaping: "" or '')
                    if i + 1 < chars.len() && chars[i + 1] == string_char {
                        i += 1; // skip next char
                    } else {
                        in_string = false;
                    }
                }
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                _ => {}
            }
            i += 1;
        }

        if brace_count > 0 || paren_count > 0 || in_string {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

// --- Highlighter ---

/// Basic syntax highlighter for PowerShell
struct PowerShellHighlighter;

impl Highlighter for PowerShellHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled_text = StyledText::new();
        
        // Simple keyword-based highlighter for the REPL
        // We split by whitespace but keep track of where we are to preserve spacing
        let words = line.split_inclusive(|c: char| c.is_whitespace() || "{}()|;=.".contains(c));

        for word in words {
            let trimmed = word.trim();
            let style = if trimmed.is_empty() {
                Style::new()
            } else {
                match trimmed.to_lowercase().as_str() {
                    "if" | "else" | "elseif" | "function" | "return" => Style::new().fg(Color::Magenta).bold(),
                    w if w.starts_with('$') => Style::new().fg(Color::Cyan),
                    w if w.starts_with('-') && w.len() > 1 && w.chars().nth(1).unwrap().is_alphabetic() => Style::new().fg(Color::Yellow),
                    "{" | "}" | "(" | ")" | "|" | ";" | "=" | "." => Style::new().fg(Color::LightGray),
                    _ => Style::new().fg(Color::White),
                }
            };
            styled_text.push((style, word.to_string()));
        }

        styled_text
    }
}

// --- Prompt ---

/// Custom prompt to handle multiline continuation
struct PowerShellPrompt;

impl Prompt for PowerShellPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("PS > ")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(">> ")
    }

    fn render_prompt_history_search_indicator(&self, history_search: PromptHistorySearch) -> Cow<'_, str> {
        Cow::Owned(format!("(search: {})", history_search.term))
    }
}

fn main() -> std::io::Result<()> {
    println!("PowerShell Interpreter - Modern REPL");
    println!("Object Pipeline with 5 Cmdlets!");
    println!(
        "Available cmdlets: Write-Output, Get-Process, Where-Object, Select-Object, ForEach-Object"
    );
    println!("Type 'exit' to quit, or use Ctrl+D.\n");

    // Create evaluator and register all cmdlets
    let mut evaluator = Evaluator::new();
    pwsh_cmdlets::register_all(evaluator.registry_mut());

    // Set up reedline components
    let history = Box::new(
        FileBackedHistory::with_file(1000, "history.txt".into())
            .expect("Error creating history file"),
    );

    // Add a basic completer for cmdlets
    let commands = vec![
        "Write-Output".into(),
        "Get-Process".into(),
        "Where-Object".into(),
        "Select-Object".into(),
        "ForEach-Object".into(),
        "exit".into(),
    ];
    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands, 2));

    // Set up the line editor
    let mut line_editor = Reedline::create()
        .with_validator(Box::new(PowerShellValidator))
        .with_highlighter(Box::new(PowerShellHighlighter))
        .with_history(history)
        .with_completer(completer)
        .with_quick_completions(true)
        .with_partial_completions(true);

    // Add a menu for completions (Tab)
    line_editor = line_editor.with_menu(ReedlineMenu::EngineCompleter(Box::new(
        ColumnarMenu::default().with_name("completion_menu"),
    )));

    let prompt = PowerShellPrompt;

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(buffer)) => {
                let input = buffer.trim();

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
            Ok(Signal::CtrlC) => {
                // Just clear the line
                continue;
            }
            Ok(Signal::CtrlD) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
