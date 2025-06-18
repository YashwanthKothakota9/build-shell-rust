use std::io::{self, Write};
use std::{env, fs::File, path::Path, process::Command};

pub fn check_path(command: &str) -> Option<String> {
    let key = "PATH";
    let value = env::var(key).unwrap();
    let paths = value.split(":").collect::<Vec<&str>>();
    for path in paths {
        let full_path = format!("{}/{}", path, command);
        if Path::new(&full_path).exists() {
            return Some(full_path);
        }
    }
    None
}

pub fn parse_input(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single_quotes => {
                if let Some(&next_ch) = chars.peek() {
                    if in_double_quotes {
                        match next_ch {
                            '"' | '$' | '`' | '\\' | '\n' => {
                                chars.next();
                                current_arg.push(next_ch);
                            }
                            _ => current_arg.push('\\'),
                        }
                    } else {
                        chars.next();
                        current_arg.push(next_ch);
                    }
                } else {
                    current_arg.push('\\');
                }
            }
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            ch if ch.is_whitespace() && !in_single_quotes && !in_double_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => current_arg.push(ch),
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    args
}

pub fn check_for_stdout_redirect(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == ">" || arg == "1>" {
            return i;
        }
    }
    args.len() - 1
}

pub fn check_for_append_stdout(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == ">>" || arg == "1>>" {
            return i;
        }
    }
    args.len() - 1
}

pub fn check_for_stderr_redirect(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == "2>" {
            return i;
        }
    }
    args.len() - 1
}

pub fn check_for_append_stderr(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == "2>>" {
            return i;
        }
    }
    args.len() - 1
}

pub fn run_command(command_name: &str, args: &[String]) {
    let output = Command::new(command_name).args(args).output().unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

pub fn run_command_with_stdout_redirect(command_name: &str, args: &[String], file_name: &str) {
    let output = Command::new(command_name).args(args).output().unwrap();
    let mut file = File::create(file_name).unwrap();
    file.write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

pub fn run_command_with_stderr_redirect(command_name: &str, args: &[String], file_name: &str) {
    let output = Command::new(command_name).args(args).output().unwrap();
    let mut file = File::create(file_name).unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    file.write_all(&output.stderr).unwrap();
}

pub fn run_command_with_append_stdout_redirect(
    command_name: &str,
    args: &[String],
    file_name: &str,
) {
    let output = Command::new(command_name).args(args).output().unwrap();
    if Path::new(file_name).exists() {
        let mut file = File::options().append(true).open(file_name).unwrap();
        file.write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    } else {
        let mut file = File::create(file_name).unwrap();
        file.write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}

pub fn run_command_with_append_stderr_redirect(
    command_name: &str,
    args: &[String],
    file_name: &str,
) {
    let output = Command::new(command_name).args(args).output().unwrap();
    if Path::new(file_name).exists() {
        let mut file = File::options().append(true).open(file_name).unwrap();
        file.write_all(&output.stderr).unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
    } else {
        let mut file = File::create(file_name).unwrap();
        file.write_all(&output.stderr).unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
    }
}
