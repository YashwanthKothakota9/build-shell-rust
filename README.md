[![progress-banner](https://backend.codecrafters.io/progress/shell/c466fcb3-1c63-4da1-8a60-9ae181b6e839)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

<h1 align="center">Shell from scratch in Rust</h1>

<div align="center">
    <img src="/image.png" alt="Project progress image">
</div>

### Stages:
1. Print a Prompt
2. Handle invalid commands
3. Implement REPL (Read Eval Print Loop)
4. Implement `exit` builtin
5. Implement `echo` builtin
6. Implement `type` builtin along with executables
7. Run a program
8. Implement `pwd` builtin
9. Implement `cd` builtin - absolute paths, relative paths and home directory
10. Support single quotes and double quotes
11. Support backslash - outside quotes, within single quotes, within double quotes
12. Implement `quoted` executable
13. Implement `redirection` - `stdout` and `stderr`
14. Append - `stdout` and `stderr`
15. Builtin completion
    1.  Needs to replace basic input system with input system that can
        1.  Detect when `TAB` is pressed
        2.  Show what the user is typing in real-time
        3.  Complete the word when `TAB` pressed
        4.  Handle backspace, arrow keys, etc.
    2.  `rustyline` - rust implementation of `readline`, library that handles all complex terminal input automatically
16. Handle missing, multiple and partial completions
17. Handling Pipelines
18. Implement `history`
19. Implement `history` persistence through file.