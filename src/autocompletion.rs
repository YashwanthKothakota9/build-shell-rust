use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct ShellCompleter;

impl ShellCompleter {
    fn get_executables_from_path() -> Vec<String> {
        let mut executables = Vec::new();

        if let Ok(path_var) = env::var("PATH") {
            for path_dir in path_var.split(':') {
                if let Ok(entries) = fs::read_dir(path_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && Self::is_executable(&path) {
                            if let Some(name) = path.file_name() {
                                if let Some(name_str) = name.to_str() {
                                    executables.push(name_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        executables
    }

    fn is_executable(path: &Path) -> bool {
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(path) {
                let mode = metadata.permissions().mode();
                return (mode & 0o111) != 0;
            }
        }

        #[cfg(windows)]
        {
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    return matches!(
                        ext_str.to_lowercase().as_str(),
                        "exe" | "bat" | "cmd" | "com"
                    );
                }
            }
        }

        false
    }
}

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut candidates = Vec::new();

        let builtins = ["exit", "echo", "type", "pwd", "cd", "ls"];

        let executables = Self::get_executables_from_path();

        let mut all_commands = builtins.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        all_commands.extend(executables);

        for command in all_commands {
            if command.starts_with(line) {
                candidates.push(format!("{} ", command));
            }
        }

        Ok((0, candidates))
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
