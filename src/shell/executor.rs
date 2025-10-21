// src/shell/executor.rs
use crate::shell::commands::CommandRegistry;
use std::process::Command as SysCommand;

pub fn execute_command(input: &str, registry: &CommandRegistry) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    let args = &parts[1..];

    // Essai commandes internes
    if registry.execute(cmd, args) {
        return;
    }

    // Sinon, essai système
    match SysCommand::new(cmd).args(args).output() {
        Ok(out) => {
            if !out.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&out.stdout));
            }
            if !out.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&out.stderr));
            }
        }
        Err(_) => {
            eprintln!("❌ Command not found: {}", cmd);
            if let Some(s) = registry.suggest(cmd) {
                eprintln!("   Did you mean: {} ?", s);
            }
        }
    }
}
