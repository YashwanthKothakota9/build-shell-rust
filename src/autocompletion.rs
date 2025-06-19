use rustyline::completion::{Completer, Pair};

use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;
use rustyline::Helper;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};

const BUILTINS: &[&str] = &["echo", "exit", "type", "pwd", "cd", "ls"];

#[derive(Default)]
pub struct ShellCompleter {
    tab_count: RefCell<u32>,
    last_line: RefCell<String>,
}

impl ShellCompleter {
    fn ring_bell() {
        print!("\x07"); // ASCII bell character
        io::stdout().flush().unwrap();
    }

    fn print_matches(matches: &[String], current_input: &str) {
        println!();
        for (i, matche) in matches.iter().enumerate() {
            if i > 0 {
                print!("  "); // 2 spaces separator
            }
            print!("{}", matche);
        }
        println!();
        print!("$ {}", current_input);
        io::stdout().flush().unwrap();
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let prefix = &line[start..pos];

        // Check if this is a continuation of the same line
        let mut tab_count = self.tab_count.borrow_mut();
        let mut last_line = self.last_line.borrow_mut();

        if line == *last_line {
            *tab_count += 1;
        } else {
            *tab_count = 1;
            *last_line = line.to_string();
        }

        // First, check for builtin matches
        let builtin_matches: Vec<_> = BUILTINS
            .iter()
            .filter(|builtin| builtin.starts_with(prefix))
            .map(|builtin| Pair {
                display: builtin.to_string(),
                replacement: builtin.to_string() + " ",
            })
            .collect();

        // If we have exactly one builtin match, complete it immediately
        if builtin_matches.len() == 1 {
            *tab_count = 0; // Reset tab count
            return Ok((start, builtin_matches));
        }

        // If we have multiple builtin matches, complete them immediately
        if builtin_matches.len() > 1 {
            *tab_count = 0; // Reset tab count
            return Ok((start, builtin_matches));
        }

        // If no builtin matches, check external executables
        let paths = env::var_os("PATH").unwrap_or_default();
        let mut external_matches: Vec<_> = env::split_paths(&paths)
            .flat_map(|path| match fs::read_dir(&path) {
                Ok(readdir) => readdir
                    .filter_map(|entry| match entry {
                        Ok(entry) => {
                            let filename = entry.file_name().to_string_lossy().to_string();
                            match filename.starts_with(prefix) {
                                true => Some(filename),
                                false => None,
                            }
                        }
                        Err(_) => None,
                    })
                    .collect::<Vec<_>>(),
                Err(_) => vec![],
            })
            .collect();

        external_matches.sort();
        let external_pairs: Vec<_> = external_matches
            .into_iter()
            .map(|v| Pair {
                display: v.clone(),
                replacement: v + " ",
            })
            .collect();

        // Handle external executable matches
        if external_pairs.is_empty() {
            // No matches at all
            *tab_count = 0;
            return Ok((start, vec![]));
        }

        if external_pairs.len() == 1 {
            // Single external match, complete it immediately
            *tab_count = 0;
            return Ok((start, external_pairs));
        }

        // Multiple external matches - use bell/list behavior
        if *tab_count == 1 {
            // First TAB: ring bell
            Self::ring_bell();
            return Ok((start, vec![]));
        } else if *tab_count == 2 {
            // Second TAB: show all matches
            let matches: Vec<String> = external_pairs.iter().map(|p| p.display.clone()).collect();
            Self::print_matches(&matches, line);
            *tab_count = 0; // Reset tab count
            return Ok((start, vec![]));
        }

        Ok((start, external_pairs))
    }
}

impl Helper for ShellCompleter {}

impl Validator for ShellCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for ShellCompleter {}

impl Hinter for ShellCompleter {
    type Hint = String;
}
