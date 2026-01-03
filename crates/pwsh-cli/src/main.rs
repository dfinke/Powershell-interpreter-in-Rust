use nu_ansi_term::{Color, Style};
use pwsh_lexer::Lexer;
use pwsh_parser::Parser;
use pwsh_runtime::Evaluator;
use reedline::{
    ColumnarMenu, Completer, Emacs, FileBackedHistory, Highlighter, KeyCode, KeyModifiers, Prompt,
    PromptEditMode, PromptHistorySearch, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span,
    StyledText, Suggestion, ValidationResult, Validator,
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

// --- Completer ---

/// Case-insensitive completer for PowerShell cmdlets
struct PowerShellCompleter {
    commands: Vec<String>,
}

impl PowerShellCompleter {
    fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

impl Completer for PowerShellCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Extract the word at the cursor position
        // Find the start of the current word by scanning backwards
        let bytes_before = &line.as_bytes()[..pos];
        let mut start = pos;

        // Scan backwards through byte positions to find word boundary
        for (i, &byte) in bytes_before.iter().enumerate().rev() {
            let c = byte as char;
            if c.is_whitespace() || c == '|' || c == ';' {
                start = i + 1;
                break;
            }
            if i == 0 {
                start = 0;
            }
        }

        let partial = &line[start..pos];

        let partial_lower = partial.to_lowercase();

        // Find matching commands (case-insensitive)
        // If partial is empty, return all commands
        // Otherwise, filter by prefix match
        let mut completions: Vec<Suggestion> = self
            .commands
            .iter()
            .filter(|cmd| {
                if partial.is_empty() {
                    true
                } else {
                    cmd.to_lowercase().starts_with(&partial_lower)
                }
            })
            .map(|cmd| Suggestion {
                value: cmd.clone(),
                description: None,
                extra: None,
                span: Span::new(start, pos),
                append_whitespace: true,
            })
            .collect();

        // Sort completions for consistent ordering
        completions.sort_by(|a, b| a.value.cmp(&b.value));

        completions
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
                    "if" | "else" | "elseif" | "function" | "return" => {
                        Style::new().fg(Color::Magenta).bold()
                    }
                    w if w.starts_with('$') => Style::new().fg(Color::Cyan),
                    w if w.starts_with('-')
                        && w.len() > 1
                        && w.chars().nth(1).unwrap().is_alphabetic() =>
                    {
                        Style::new().fg(Color::Yellow)
                    }
                    "{" | "}" | "(" | ")" | "|" | ";" | "=" | "." => {
                        Style::new().fg(Color::LightGray)
                    }
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

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Owned(format!("(search: {})", history_search.term))
    }
}

fn main() -> std::io::Result<()> {
    println!("PowerShell Interpreter - Modern REPL");
    println!("Object Pipeline with 6 Cmdlets!");
    println!(
        "Available cmdlets: Write-Output, Get-Process, Get-ChildItem, Where-Object, Select-Object, ForEach-Object"
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

    let mut keybindings = reedline::default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char(' '),
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::BackTab,
        ReedlineEvent::MenuPrevious,
    );

    // Add a case-insensitive completer for cmdlets
    let commands = vec![
        "Write-Output".to_string(),
        "Get-Process".to_string(),
        "Get-ChildItem".to_string(),
        "Where-Object".to_string(),
        "Select-Object".to_string(),
        "ForEach-Object".to_string(),
        "exit".to_string(),
    ];
    let completer = Box::new(PowerShellCompleter::new(commands));

    // Set up the line editor
    let mut line_editor = Reedline::create()
        .with_validator(Box::new(PowerShellValidator))
        .with_highlighter(Box::new(PowerShellHighlighter))
        .with_history(history)
        .with_completer(completer)
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_edit_mode(Box::new(Emacs::new(keybindings)));

    // Add a menu for completions (Tab)
    let completion_menu = ColumnarMenu::default()
        .with_name("completion_menu")
        .with_text_style(Style::new().fg(Color::Cyan))
        .with_selected_text_style(Style::new().fg(Color::Black).on(Color::Cyan));

    line_editor = line_editor.with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_completion() {
        let commands = vec![
            "Write-Output".to_string(),
            "Get-Process".to_string(),
            "Where-Object".to_string(),
            "Select-Object".to_string(),
            "ForEach-Object".to_string(),
        ];
        let mut completer = PowerShellCompleter::new(commands);

        // Test lowercase "select" should match "Select-Object"
        let completions = completer.complete("select", 6);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Select-Object");

        // Test uppercase "SELECT" should also match
        let completions = completer.complete("SELECT", 6);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Select-Object");

        // Test mixed case "SeLeCt" should also match
        let completions = completer.complete("SeLeCt", 6);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Select-Object");

        // Test partial "wh" should match "Where-Object"
        let completions = completer.complete("wh", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Where-Object");

        // Test "get" should match "Get-Process"
        let completions = completer.complete("get", 3);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Get-Process");

        // Test in pipeline context: "$a | select"
        let completions = completer.complete("$a | select", 11);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Select-Object");

        // Test that single character now completes
        let completions = completer.complete("s", 1);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Select-Object");

        // Test empty partial after pipe shows all commands
        let completions = completer.complete("1 | ", 4);
        assert_eq!(completions.len(), 5);
        // Verify they are sorted
        assert_eq!(completions[0].value, "ForEach-Object");
        assert_eq!(completions[1].value, "Get-Process");
        assert_eq!(completions[2].value, "Select-Object");
        assert_eq!(completions[3].value, "Where-Object");
        assert_eq!(completions[4].value, "Write-Output");
    }

    #[test]
    fn test_multiple_matches() {
        let commands = vec![
            "Write-Output".to_string(),
            "Where-Object".to_string(),
            "Select-Object".to_string(),
        ];
        let mut completer = PowerShellCompleter::new(commands);

        // Test "w" prefix should match both Write-Output and Where-Object
        let completions = completer.complete("w", 1);
        // Should now return both matches
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0].value, "Where-Object");
        assert_eq!(completions[1].value, "Write-Output");

        // Test "wr" should match Write-Output
        let completions = completer.complete("wr", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Write-Output");

        // Test "wh" should match Where-Object
        let completions = completer.complete("wh", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].value, "Where-Object");
    }
}
