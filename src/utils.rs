use nix::sys::wait::waitpid;
use nix::unistd::{close, dup2, fork, pipe, ForkResult};
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

/// Check if the input contains a pipeline (| operator)
pub fn has_pipeline(input: &str) -> bool {
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single_quotes => {
                // Skip the next character if it's escaped
                chars.next();
            }
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '|' if !in_single_quotes && !in_double_quotes => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Split input into separate commands based on pipeline operators
pub fn split_pipeline(input: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single_quotes => {
                // Skip the next character if it's escaped
                if let Some(next_ch) = chars.next() {
                    current_command.push(ch);
                    current_command.push(next_ch);
                } else {
                    current_command.push(ch);
                }
            }
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '|' if !in_single_quotes && !in_double_quotes => {
                // Found a pipeline operator, save current command and start new one
                commands.push(current_command.trim().to_string());
                current_command.clear();
            }
            _ => current_command.push(ch),
        }
    }

    // Add the last command
    if !current_command.trim().is_empty() {
        commands.push(current_command.trim().to_string());
    }

    commands
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

/// Execute a pipeline of commands using pipes and process management
pub fn execute_pipeline(commands: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if commands.len() < 2 {
        return Err("Pipeline must have at least 2 commands".into());
    }

    let mut child_pids = Vec::new();
    let mut prev_pipe_read = None;

    // Helper function to safely close file descriptors
    fn safe_close(fd: i32) {
        if let Err(e) = close(fd) {
            eprintln!("Warning: Failed to close file descriptor {}: {}", fd, e);
        }
    }

    // Execute all commands including the last one in child processes
    for (i, command_str) in commands.iter().enumerate() {
        let args = parse_input(command_str);
        if args.is_empty() {
            continue;
        }

        let command_name = &args[0];
        let command_args = &args[1..];

        let pipe_info = if i < commands.len() - 1 {
            // Not the last command, create a pipe
            Some(pipe()?)
        } else {
            // Last command, no pipe needed
            None
        };

        // Fork a new process
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                // Parent process
                child_pids.push(child);

                // Close the write end of the pipe in parent (if any)
                if let Some((pipe_read, pipe_write)) = pipe_info {
                    safe_close(pipe_write);

                    // If we had a previous pipe, close its read end
                    if let Some(prev_read) = prev_pipe_read {
                        safe_close(prev_read);
                    }

                    prev_pipe_read = Some(pipe_read);
                } else {
                    // Last command, close the previous pipe
                    if let Some(prev_read) = prev_pipe_read {
                        safe_close(prev_read);
                    }
                }
            }
            ForkResult::Child => {
                // Child process - ensure proper cleanup on exit
                let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                    // Redirect stdin from previous pipe if it exists
                    if let Some(prev_read) = prev_pipe_read {
                        dup2(prev_read, 0)?; // Redirect stdin
                        safe_close(prev_read);
                    }

                    // Redirect stdout to the new pipe if this isn't the last command
                    if let Some((pipe_read, pipe_write)) = pipe_info {
                        dup2(pipe_write, 1)?; // Redirect stdout
                        safe_close(pipe_read);
                        safe_close(pipe_write);
                    }

                    // Execute the command
                    execute_single_command(command_name, command_args)?;
                    Ok(())
                })();

                // Exit with appropriate code
                match result {
                    Ok(_) => std::process::exit(0),
                    Err(_) => std::process::exit(1),
                }
            }
        }
    }

    // Wait for all child processes to complete and handle any errors
    for child_pid in child_pids {
        match waitpid(child_pid, None) {
            Ok(_) => {}
            Err(e) => {
                // Log error but don't fail the entire pipeline
                eprintln!(
                    "Warning: Failed to wait for child process {}: {}",
                    child_pid, e
                );
            }
        }
    }

    Ok(())
}

/// Execute a single command (built-in or external)
fn execute_single_command(
    command_name: &str,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    match command_name {
        "echo" => {
            let output = args.join(" ");
            println!("{}", output);
            Ok(())
        }
        "exit" => {
            // In a pipeline, exit should terminate the current process
            std::process::exit(0);
        }
        "pwd" => {
            let current_dir = env::current_dir()?;
            println!("{}", current_dir.to_string_lossy());
            Ok(())
        }
        "cd" => {
            // Note: cd in a pipeline doesn't affect the parent shell's directory
            // This is expected behavior for most shells
            let new_path = if args.is_empty() || args[0] == "~" {
                env::var("HOME")?
            } else {
                args[0].clone()
            };
            env::set_current_dir(&new_path)?;
            Ok(())
        }
        "type" => {
            let builtins = ["exit", "echo", "type", "pwd", "cd", "ls"];
            if builtins.contains(&args[0].as_str()) {
                println!("{} is a shell builtin", args[0]);
            } else {
                let path = check_path(&args[0]);
                if let Some(path) = path {
                    println!("{} is {}", args[0], path);
                } else {
                    eprintln!("{}: not found", args[0]);
                }
            }
            Ok(())
        }
        "ls" => {
            let mut cmd = Command::new("ls");
            cmd.args(args);
            let status = cmd.status()?;
            if !status.success() {
                return Err(format!("ls command failed with status: {}", status).into());
            }
            Ok(())
        }
        "cat" => {
            let mut cmd = Command::new("cat");
            cmd.args(args);
            let status = cmd.status()?;
            if !status.success() {
                return Err(format!("cat command failed with status: {}", status).into());
            }
            Ok(())
        }
        _ => {
            // External command - handle special cases for pipeline commands
            let path = check_path(command_name);
            match path {
                Some(_path) => {
                    // Special handling for commands that might need different behavior in pipelines
                    {
                        // Generic external command handling
                        let mut cmd = Command::new(command_name);
                        cmd.args(args);
                        let status = cmd.status()?;
                        if !status.success() {
                            return Err(format!(
                                "{} command failed with status: {}",
                                command_name, status
                            )
                            .into());
                        }
                        Ok(())
                    }
                }
                None => {
                    eprintln!("{}: command not found", command_name);
                    Err(format!("{}: command not found", command_name).into())
                }
            }
        }
    }
}
