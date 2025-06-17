#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    path::{Path, PathBuf},
    process::{exit, Command},
};

const BUILT_IN_COMMANDS: [&str; 4] = ["exit", "echo", "type", "pwd"];

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

        let (command_name, args) = match input_command.split_once(" ") {
            Some((cmd, args)) => (cmd, args),
            None => (input_command, ""),
        };

        if command_name == "exit" {
            exit(0);
        } else if command_name == "echo" {
            println!("{}", args);
        } else if command_name == "type" {
            if BUILT_IN_COMMANDS.contains(&args) {
                println!("{} is a shell builtin", args);
            } else if let Some(full_path) = check_path(args) {
                println!("{} is {}", args, full_path);
            } else {
                println!("{}: not found", args);
            }
        } else if command_name == "pwd" {
            println!("{}", env::current_dir().unwrap().display())
        } else if command_name == "cd" && !args.is_empty() {
            let path = if args == "~" {
                let home_path = env::var("HOME").unwrap();
                PathBuf::from(home_path)
            } else {
                PathBuf::from(args)
            };
            if path.exists() {
                env::set_current_dir(path).unwrap();
            } else {
                println!("cd: {}: No such file or directory", args);
            }
        } else {
            match check_path(command_name) {
                Some(_) => {
                    let output = Command::new(command_name)
                        .args(args.split_whitespace())
                        .output()
                        .expect("Failed to execute command");

                    if !output.stdout.is_empty() {
                        print!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                None => {
                    println!("{}: command not found", command_name);
                }
            }
        }
    }
}
