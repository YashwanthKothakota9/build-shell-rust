use regex::Regex;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::{exit, Command},
};

const BUILT_IN_COMMANDS: [&str; 6] = ["exit", "echo", "type", "pwd", "cd", "ls"];

fn check_path(command: &str) -> Option<String> {
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

fn parse_input(input: &str) -> Vec<String> {
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

fn check_for_stdout_redirect(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == ">" || arg == "1>" {
            return i;
        }
    }
    args.len() - 1
}

fn check_for_stderr_redirect(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == "2>" {
            return i;
        }
    }
    args.len() - 1
}

fn run_command(command_name: &str, args: &[String]) {
    let output = Command::new(command_name).args(args).output().unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn run_command_with_stdout_redirect(command_name: &str, args: &[String], file_name: &str) {
    let output = Command::new(command_name).args(args).output().unwrap();
    let mut file = File::create(file_name).unwrap();
    file.write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn run_command_with_stderr_redirect(command_name: &str, args: &[String], file_name: &str) {
    let output = Command::new(command_name).args(args).output().unwrap();
    let mut file = File::create(file_name).unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    file.write_all(&output.stderr).unwrap();
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input_command = input.trim();

        if input_command.is_empty() {
            continue;
        }

        let inputs = parse_input(input_command);
        let command_name = &inputs[0];
        let args = &inputs[1..];

        let stdout_redirect_index = check_for_stdout_redirect(args);
        let stderr_redirect_index = check_for_stderr_redirect(args);

        match command_name.as_str() {
            "exit" => exit(0),
            "echo" => {
                if stdout_redirect_index != args.len() - 1 {
                    let file_name = args[stdout_redirect_index + 1].clone();
                    run_command_with_stdout_redirect(
                        command_name,
                        &args[0..stdout_redirect_index],
                        &file_name,
                    );
                } else if stderr_redirect_index != args.len() - 1 {
                    let file_name = args[stderr_redirect_index + 1].clone();
                    run_command_with_stderr_redirect(
                        command_name,
                        &args[0..stderr_redirect_index],
                        &file_name,
                    );
                } else {
                    run_command(command_name, args);
                }
            }
            "pwd" => println!("{}", env::current_dir().unwrap().to_string_lossy()),
            "cd" => {
                if args.is_empty() || args[0] == "~" {
                    env::set_current_dir(env::var("HOME").unwrap()).unwrap();
                } else {
                    let new_path = args[0].clone();
                    if let Err(_e) = env::set_current_dir(&new_path) {
                        println!("cd: {}: No such file or directory", new_path);
                    }
                }
            }
            "type" => {
                if BUILT_IN_COMMANDS.contains(&args[0].as_str()) {
                    println!("{} is a shell builtin", args[0]);
                } else {
                    let path = check_path(args[0].as_str());
                    if let Some(path) = path {
                        println!("{} is {}", args[0], path);
                    } else {
                        eprintln!("{}: not found", args[0]);
                    }
                }
            }
            "ls" => {
                if stdout_redirect_index != args.len() - 1 {
                    let file_name = args[stdout_redirect_index + 1].clone();
                    run_command_with_stdout_redirect(
                        command_name,
                        &args[0..stdout_redirect_index],
                        &file_name,
                    );
                } else if stderr_redirect_index != args.len() - 1 {
                    let file_name = args[stderr_redirect_index + 1].clone();
                    run_command_with_stderr_redirect(
                        command_name,
                        &args[0..stderr_redirect_index],
                        &file_name,
                    );
                } else {
                    run_command(command_name, args);
                }
            }
            "cat" => {
                if stdout_redirect_index == args.len() - 1
                    && stderr_redirect_index == args.len() - 1
                {
                    run_command(command_name, args);
                } else if stdout_redirect_index != args.len() - 1 {
                    // println!("stdout_redirect_index: {}", stdout_redirect_index);
                    let destination_file_name = args[stdout_redirect_index + 1].clone();
                    run_command_with_stdout_redirect(
                        command_name,
                        &args[0..stdout_redirect_index],
                        &destination_file_name,
                    );
                } else if stderr_redirect_index != args.len() - 1 {
                    // println!("stderr_redirect_index: {}", stderr_redirect_index);
                    let destination_file_name = args[stderr_redirect_index + 1].clone();
                    // println!("cat_destination_file_name: {}", destination_file_name);
                    run_command_with_stderr_redirect(
                        command_name,
                        &args[0..stderr_redirect_index],
                        &destination_file_name,
                    );
                }
            }
            _ => {
                let path = check_path(command_name);
                match path {
                    Some(_path) => {
                        run_command(command_name, args);
                    }
                    None => {
                        eprintln!("{}: command not found", command_name);
                    }
                }
            }
        }
    }
}
