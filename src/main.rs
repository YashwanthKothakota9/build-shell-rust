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

        match command_name.as_str() {
            "exit" => exit(0),
            "echo" => {
                if args.len() > 1 && (args[1] == ">" || args[1] == "1>") {
                    let destination_file_name = args[2].clone();
                    let output = Command::new("echo").args(&args[0..1]).output().unwrap();
                    let mut file = File::create(destination_file_name).unwrap();
                    file.write_all(&output.stdout).unwrap();
                } else {
                    println!("{}", args.join(" "))
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
                if args.len() > 2 && (args[2] == ">" || args[2] == "1>") {
                    let destination_file_name = args[3].clone();
                    let output = Command::new("ls").args(&args[0..2]).output().unwrap();
                    let mut file = File::create(destination_file_name).unwrap();
                    file.write_all(&output.stdout).unwrap();
                } else {
                    let output = Command::new("ls").args(&args[0..2]).output().unwrap();
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                }
            }
            "cat" => {
                let mut i = 0;
                for (j, arg) in args.iter().enumerate() {
                    if arg == ">" || arg == "1>" {
                        i = j;
                    }
                }
                if i == 0 {
                    let output = Command::new("cat").args(&args[0..]).output().unwrap();
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                } else {
                    let destination_file_name = args[i + 1].clone();
                    let mut new_args = Vec::new();
                    for j in 0..i {
                        if Path::new(&args[j]).exists() {
                            new_args.push(args[j].clone());
                        } else {
                            println!("{}: {}: No such file or directory", command_name, args[j]);
                        }
                    }
                    let output = Command::new("cat").args(new_args).output().unwrap();
                    let mut file = File::create(destination_file_name).unwrap();
                    file.write_all(&output.stdout).unwrap();
                }
            }

            _ => {
                let path = check_path(command_name);
                match path {
                    Some(path) => {
                        let output = Command::new(command_name).args(args).output().unwrap();
                        io::stdout().write_all(&output.stdout).unwrap();
                        io::stderr().write_all(&output.stderr).unwrap();
                    }
                    None => {
                        eprintln!("{}: command not found", command_name);
                    }
                }
            }
        }
    }
}
