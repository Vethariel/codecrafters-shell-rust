use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

enum ArgPolicy {
    None,
    Exact(usize),
    AtLeast(usize),
}

struct CommandSpec {
    name: String,
    arg_policy: ArgPolicy,
    handler: fn(Vec<String>) -> Result<(), String>,
    source: Option<String>,
}

fn main() {
    loop {
        repl();
    }
}

fn repl() {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    process_command(command.trim());
}

fn process_command(input: &str) {
    let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return;
    }

    let command_name = parts[0].clone();
    let args = parts[1..].to_vec();

    let commands = get_commands();
    match commands.iter().find(|cmd| cmd.name == command_name) {
        Some(cmd) => match validate_args(&cmd.arg_policy, &args) {
            Ok(_) => {
                let mut args = args.clone();
                if cmd.source != Some("builtin".to_string()) {
                    args.insert(0, command_name.clone());
                }
                if let Err(err) = (cmd.handler)(args) {
                    eprintln!("Error: {}", err);
                }
            }
            Err(err) => eprintln!("Argument Error: {}", err),
        },
        None => eprintln!("{}: command not found", command_name),
    }
}

fn validate_args(policy: &ArgPolicy, args: &Vec<String>) -> Result<(), String> {
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
    let mut commands = vec![
        CommandSpec {
            name: "echo".to_string(),
            arg_policy: ArgPolicy::AtLeast(1),
            handler: echo_command,
            source: Some("builtin".to_string()),
        },
        CommandSpec {
            name: "exit".to_string(),
            arg_policy: ArgPolicy::None,
            handler: exit_command,
            source: Some("builtin".to_string()),
        },
        CommandSpec {
            name: "type".to_string(),
            arg_policy: ArgPolicy::Exact(1),
            handler: type_command,
            source: Some("builtin".to_string()),
        },
    ];
    commands.extend(get_commands_path());
    commands
}

fn get_commands_path() -> Vec<CommandSpec> {
    let path_eval = env::var_os("PATH").unwrap_or_default();
    let paths: Vec<String> = env::split_paths(&path_eval)
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect();
    let mut commands = Vec::new();
    for path in paths {
        let entries = std::fs::read_dir(&path);
        if !entries.is_ok() {
            continue;
        }
        let entries = entries.unwrap();
        for entry in entries.flatten() {
            let file_type = entry.file_type();
            if !file_type.is_ok() {
                continue;
            }
            let file_type = file_type.unwrap();
            if !file_type.is_file() {
                continue;
            }

            let metadata = entry.metadata();
            if !metadata.is_ok() {
                continue;
            }
            let metadata = metadata.unwrap();
            let permissions = metadata.permissions();
            if (permissions.mode() & 0o111) == 0 {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                commands.push(CommandSpec {
                    name: name.to_string(),
                    arg_policy: ArgPolicy::AtLeast(0),
                    handler: program_command,
                    source: Some(path.clone()),
                });
            }
        }
    }
    commands
}

fn echo_command(args: Vec<String>) -> Result<(), String> {
    println!("{}", args.join(" "));
    Ok(())
}

fn exit_command(_args: Vec<String>) -> Result<(), String> {
    std::process::exit(0);
}

fn type_command(args: Vec<String>) -> Result<(), String> {
    let command_name = args[0].clone();
    let commands = get_commands();
    match commands.iter().find(|cmd| cmd.name == command_name) {
        Some(cmd) => {
            match &cmd.source {
                Some(source) => {
                    if source == "builtin" {
                        println!("{} is a shell builtin", command_name);
                    } else {
                        println!("{} is {}/{}", command_name, source, command_name);
                    }
                }
                None => {
                    println!("{}: found", command_name);
                }
            }
            Ok(())
        }
        None => {
            println!("{}: not found", command_name);
            Ok(())
        }
    }
}

fn program_command(args: Vec<String>) -> Result<(), String> {
    std::process::Command::new(args[0].clone())
        .args(&args[1..])
        .status()
        .map_err(|e| format!("Failed to execute {}: {}", args[0], e))?;
    Ok(())
}
