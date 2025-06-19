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

    /// Find the longest common prefix among a list of strings
    /// Returns (common_prefix, unique_matches_count)
    fn find_longest_common_prefix(strings: &[String]) -> (String, usize) {
        if strings.is_empty() {
            return (String::new(), 0);
        }

        if strings.len() == 1 {
            return (strings[0].clone(), 1);
        }

        // Find the shortest string length to avoid out-of-bounds access
        let min_len = strings.iter().map(|s| s.len()).min().unwrap_or(0);

        // Find the longest common prefix
        let mut common_prefix_len = 0;
        'outer: for i in 0..min_len {
            let char_at_pos = strings[0].chars().nth(i);
            for string in strings.iter() {
                if string.chars().nth(i) != char_at_pos {
                    break 'outer;
                }
            }
            common_prefix_len = i + 1;
        }

        let common_prefix = if common_prefix_len > 0 {
            strings[0][..common_prefix_len].to_string()
        } else {
            String::new()
        };

        // Count how many unique strings we have after the common prefix
        let mut unique_suffixes = std::collections::HashSet::new();
        for string in strings {
            if string.len() > common_prefix_len {
                unique_suffixes.insert(&string[common_prefix_len..]);
            } else {
                unique_suffixes.insert(""); // Empty suffix for exact matches
            }
        }

        (common_prefix, unique_suffixes.len())
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

        // Multiple external matches - use progressive completion
        let external_names: Vec<String> =
            external_pairs.iter().map(|p| p.display.clone()).collect();
        let (common_prefix, unique_count) = Self::find_longest_common_prefix(&external_names);

        // If common prefix is longer than current input, complete to it
        if common_prefix.len() > prefix.len() {
            let completion = Pair {
                display: common_prefix.clone(),
                replacement: common_prefix,
            };
            *tab_count = 0; // Reset tab count after successful completion
            return Ok((start, vec![completion]));
        }

        // If common prefix equals current input and there's only one unique option, complete it
        if common_prefix.len() == prefix.len() && unique_count == 1 {
            // Find the full match - this should always succeed since unique_count == 1
            if let Some(full_match) = external_names.iter().find(|name| name.starts_with(prefix)) {
                let completion = Pair {
                    display: full_match.clone(),
                    replacement: full_match.clone() + " ",
                };
                *tab_count = 0; // Reset tab count after successful completion
                return Ok((start, vec![completion]));
            }
        }

        // If common prefix equals current input and there are multiple options, use bell/list behavior
        if *tab_count == 1 {
            // First TAB: ring bell
            Self::ring_bell();
            return Ok((start, vec![]));
        } else if *tab_count == 2 {
            // Second TAB: show all matches
            Self::print_matches(&external_names, line);
            *tab_count = 0; // Reset tab count
            return Ok((start, vec![]));
        }

        // Fallback: return all matches
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
