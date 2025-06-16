#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
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
        } else {
            println!("{}: command not found", input_command);
        }
    }
}
