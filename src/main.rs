#[allow(unused_imports)]
use std::io::{self, Write};

enum ArgPolicy {
    None,
    Exact(usize),
    AtLeast(usize),
}

struct CommandSpec {
    name: &'static str,
    arg_policy: ArgPolicy,
    arg_usage: &'static str,
    handler: fn(&[&str]) -> Result<(), String>,
}

fn main() {
    loop {
        repl();
    }
}

fn repl(){
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    process_command(command.trim());
}

fn process_command(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let command_name = parts[0];
    let args = &parts[1..];

    let commands = get_commands();
    match commands.iter().find(|cmd| cmd.name == command_name) {
        Some(cmd) => match validate_args(&cmd.arg_policy, args) {
            Ok(_) => {
                if let Err(err) = (cmd.handler)(args) {
                    eprintln!("Error: {}", err);
                }
            }
            Err(err) => eprintln!("Argument Error: {}", err),
        },
        None => eprintln!("{}: command not found", command_name),
    }
}

fn validate_args(policy: &ArgPolicy, args: &[&str]) -> Result<(), String> {
    match policy {
        ArgPolicy::None => {
            if !args.is_empty() {
                Err("This command takes no arguments.".to_string())
            } else {
                Ok(())
            }
        }
        ArgPolicy::Exact(n) => {
            if args.len() != *n {
                Err(format!("This command requires exactly {} argument(s).", n))
            } else {
                Ok(())
            }
        }
        ArgPolicy::AtLeast(n) => {
            if args.len() < *n {
                Err(format!("This command requires at least {} argument(s).", n))
            } else {
                Ok(())
            }
        }
    }
}

fn get_commands() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            name: "echo",
            arg_policy: ArgPolicy::AtLeast(1),
            arg_usage: "<message>",
            handler: echo_command,
        },
        CommandSpec {
            name: "exit",
            arg_policy: ArgPolicy::None,
            arg_usage: "",
            handler: exit_command,
        },
        CommandSpec {
            name: "type",
            arg_policy: ArgPolicy::Exact(1),
            arg_usage: "<command>",
            handler: type_command,
        }
    ]
}

fn echo_command(args: &[&str]) -> Result<(), String> {
    println!("{}", args.join(" "));
    Ok(())
}

fn exit_command(_args: &[&str]) -> Result<(), String> {
    std::process::exit(0);
}

fn type_command(args: &[&str]) -> Result<(), String> {
    let command_name = args[0];
    let commands = get_commands();
    match commands.iter().find(|cmd| cmd.name == command_name) {
        Some(cmd) => {
            println!("{} is a shell builtin", cmd.name);
            Ok(())
        }
        None => {
            println!("{}: not found", command_name);
            Ok(())
        }
    }
}