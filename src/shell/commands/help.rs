// src/shell/commands/help.rs
use super::Command;
use crate::shell::commands::CommandRegistry;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }
    fn about(&self) -> &'static str {
        "Affiche l’aide ou le détail d’une commande."
    }
    fn usage(&self) -> &'static str {
        "help [commande]"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["h"]
    }

    fn execute(&self, args: &[&str], registry: &CommandRegistry) {
        if let Some(cmd_name) = args.get(0).copied() {
            // détail pour une commande précise
            if let Some(md) = registry
                .list_metadata()
                .into_iter()
                .find(|(n, _, _)| n == cmd_name)
            {
                println!("{} — {}", md.0, md.1);
                println!("Usage: {}", md.2);
                return;
            }
            println!("Commande inconnue: {cmd_name}");
            if let Some(s) = registry.suggest(cmd_name) {
                println!("Vouliez-vous dire: {} ?", s);
            }
            return;
        }

        // sinon, liste des commandes
        println!("Commandes disponibles:");
        for (name, about, usage) in registry.list_metadata() {
            println!("  - {:<12} {:<40}  (usage: {})", name, about, usage);
        }
        println!("\nAstuce: `help <commande>` pour le détail.");
    }
}
