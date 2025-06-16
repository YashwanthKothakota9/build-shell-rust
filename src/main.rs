#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, path::Path, process::exit};

const BUILT_IN_COMMANDS: [&str; 3] = ["exit", "echo", "type"];

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
        let mut input_command_name = "";
        let mut input_command_args = "";

        if input_command.split_once(" ").is_some() {
            (input_command_name, input_command_args) = input_command.split_once(" ").unwrap();
        }

        if input_command_name == "exit" {
            exit(0);
        } else if input_command_name == "echo" {
            println!("{}", input_command_args);
        } else if input_command_name == "type" {
            if BUILT_IN_COMMANDS.contains(&input_command_args) {
                println!("{} is a shell builtin", input_command_args);
            } else if let Some(full_path) = check_path(input_command_args) {
                println!("{} is {}", input_command_args, full_path);
            } else {
                println!("{}: not found", input_command_args);
            }
        } else {
            println!("{}: command not found", input_command);
        }
    }
}
