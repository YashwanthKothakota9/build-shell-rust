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
        let input_value = input.trim();
        if input_value == "exit 0" {
            exit(0);
        }
        println!("{}: command not found", input_value);
    }
}
