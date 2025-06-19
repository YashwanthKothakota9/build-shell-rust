pub mod autocompletion;
pub mod utils;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::exit;

use autocompletion::ShellCompleter;
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::Editor;
use utils::*;

const BUILT_IN_COMMANDS: [&str; 7] = ["exit", "echo", "type", "pwd", "cd", "ls", "history"];

fn main() {
    let mut editor = Editor::new().expect("Unable to initiate the prompt.");
    editor.set_helper(Some(ShellCompleter::default()));
    if let Ok(history_on_startup) = env::var("HISTFILE") {
        let history_file = File::open(history_on_startup).unwrap();
        let mut history_reader = BufReader::new(history_file);
        let mut line = String::new();
        while history_reader.read_line(&mut line).unwrap() > 0 {
            let _ = editor.add_history_entry(line.trim());
            line.clear();
        }
    }

    loop {
        let readline = editor.readline("$ ");
        match readline {
            Ok(line) => {
                let input_command = line.trim();

                if input_command.is_empty() {
                    continue;
                }

                // Add command to history
                let _ = editor.add_history_entry(input_command);

                // Check if this is a pipeline
                if has_pipeline(input_command) {
                    let commands = split_pipeline(input_command);
                    if commands.len() >= 2 {
                        // Execute pipeline
                        if let Err(e) = execute_pipeline(&commands) {
                            eprintln!("Pipeline execution error: {}", e);
                        }
                        continue;
                    }
                }

                let inputs = parse_input(input_command);
                let command_name = &inputs[0];
                let args = &inputs[1..];

                let stdout_redirect_index = check_for_stdout_redirect(args);
                let stderr_redirect_index = check_for_stderr_redirect(args);
                let append_stdout_index = check_for_append_stdout(args);
                let append_stderr_index = check_for_append_stderr(args);

                match command_name.as_str() {
                    "exit" => exit(0),
                    "history" => {
                        let history = editor.history();
                        if !args.is_empty() && !args[0].is_empty() {
                            if args[0] == "-r" {
                                let history_file_name = args[1].clone();
                                let history_file = File::open(history_file_name).unwrap();
                                let mut history_reader = BufReader::new(history_file);
                                let mut line = String::new();
                                while history_reader.read_line(&mut line).unwrap() > 0 {
                                    let _ = editor.add_history_entry(line.trim());
                                    line.clear();
                                }
                                continue;
                            } else if args[0] == "-w" {
                                let history_file_name = args[1].clone();
                                let history_file = File::create(history_file_name).unwrap();
                                let mut history_writer = BufWriter::new(history_file);
                                for entry in history.iter() {
                                    writeln!(history_writer, "{}", entry).unwrap();
                                }
                                continue;
                            } else if args[0] == "-a" {
                                let history_file_name = args[1].clone();
                                let history_file = OpenOptions::new()
                                    .append(true)
                                    .open(history_file_name)
                                    .unwrap();
                                let mut history_writer = BufWriter::new(history_file);
                                for entry in history.iter() {
                                    writeln!(history_writer, "{}", entry).unwrap();
                                }
                                let _ = editor.clear_history();
                                continue;
                            } else {
                                let start_index = history.len() - args[0].parse::<usize>().unwrap();
                                for i in start_index..history.len() {
                                    println!("    {}  {}", i + 1, history[i]);
                                }
                            }
                        } else {
                            for (i, entry) in history.iter().enumerate() {
                                println!("    {}  {}", i + 1, entry);
                            }
                        }
                    }
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
                        } else if append_stdout_index != args.len() - 1 {
                            let file_name = args[append_stdout_index + 1].clone();
                            run_command_with_append_stdout_redirect(
                                command_name,
                                &args[0..append_stdout_index],
                                &file_name,
                            );
                        } else if append_stderr_index != args.len() - 1 {
                            let file_name = args[append_stderr_index + 1].clone();
                            run_command_with_append_stderr_redirect(
                                command_name,
                                &args[0..append_stderr_index],
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
                        } else if append_stdout_index != args.len() - 1 {
                            let file_name = args[append_stdout_index + 1].clone();
                            run_command_with_append_stdout_redirect(
                                command_name,
                                &args[0..append_stdout_index],
                                &file_name,
                            );
                        } else if append_stderr_index != args.len() - 1 {
                            let file_name = args[append_stderr_index + 1].clone();
                            run_command_with_append_stderr_redirect(
                                command_name,
                                &args[0..append_stderr_index],
                                &file_name,
                            );
                        } else {
                            run_command(command_name, args);
                        }
                    }
                    "cat" => {
                        if stdout_redirect_index != args.len() - 1 {
                            let destination_file_name = args[stdout_redirect_index + 1].clone();
                            run_command_with_stdout_redirect(
                                command_name,
                                &args[0..stdout_redirect_index],
                                &destination_file_name,
                            );
                        } else if stderr_redirect_index != args.len() - 1 {
                            let destination_file_name = args[stderr_redirect_index + 1].clone();
                            run_command_with_stderr_redirect(
                                command_name,
                                &args[0..stderr_redirect_index],
                                &destination_file_name,
                            );
                        } else if append_stdout_index != args.len() - 1 {
                            let file_name = args[append_stdout_index + 1].clone();
                            run_command_with_append_stdout_redirect(
                                command_name,
                                &args[0..append_stdout_index],
                                &file_name,
                            );
                        } else if append_stderr_index != args.len() - 1 {
                            let file_name = args[append_stderr_index + 1].clone();
                            run_command_with_append_stderr_redirect(
                                command_name,
                                &args[0..append_stderr_index],
                                &file_name,
                            );
                        } else {
                            run_command(command_name, args);
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
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
}
