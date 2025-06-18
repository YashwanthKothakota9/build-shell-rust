use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
pub struct ShellCompleter;

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let completions = vec!["echo ".to_string(), "exit ".to_string()];
        let mut candidates = Vec::new();

        for completion in completions {
            if completion.starts_with(line) {
                candidates.push(completion);
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
