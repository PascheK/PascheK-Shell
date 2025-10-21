use crate::shell::{commands::CommandRegistry, executor::execute_command, prompt::Prompt};
use dirs::home_dir;
use reedline::{
    DefaultCompleter, DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn start_repl() {
    let prompt = Arc::new(Mutex::new(Prompt::new()));
    let registry = CommandRegistry::new_with_prompt(prompt.clone());

    // Historique
    let history_path = home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".paschek_history");

    // Récupère la liste des commandes internes (ex: ["help","cd","clear","theme","hello"])
    let command_names: Vec<String> = registry.list_names();

    // (Optionnel) Petit debug pour vérifier qu’on a bien des mots à compléter
    eprintln!("(debug) completions: {:?}", command_names);

    // Seuil à 1 caractère (au lieu de 2) pour voir des suggestions dès la 1ère lettre
    let completer = reedline::DefaultCompleter::new_with_wordlen(command_names, 1);

    // Historique Reedline
    let mut file_history = FileBackedHistory::with_file(1000, history_path.clone()).unwrap();
    // Initialisation de l’éditeur
    let mut line_editor = Reedline::create()
        .with_history(Box::new(file_history))
        .with_completer(Box::new(completer));

    println!("🦀 Welcome to PascheK Shell");
    println!("Type 'help' for a list of commands.\n");

    loop {
        // Prompt dynamique coloré
        let prompt_text = prompt.lock().unwrap().render();
        let custom_prompt = DefaultPrompt::new(
            DefaultPromptSegment::Basic(prompt_text.into()),
            DefaultPromptSegment::Empty,
        );

        // Lecture via Reedline
        let sig = line_editor.read_line(&custom_prompt);

        match sig {
            Ok(Signal::Success(cmd)) => {
                let trimmed = cmd.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "ui" {
                    crate::shell::tui::start_tui().unwrap();
                    continue;
                }
                if trimmed == "exit" {
                    println!("👋 Goodbye!");
                    break;
                }

                execute_command(trimmed, &registry);
            }
            Ok(Signal::CtrlD) => {
                println!();
                break;
            }
            Ok(Signal::CtrlC) => {
                println!("^C");
                continue;
            }
            Err(e) => {
                eprintln!("❌ Input error: {}", e);
                break;
            }
        }
    }
}
