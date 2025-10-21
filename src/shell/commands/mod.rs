// src/shell/commands/mod.rs
use std::collections::HashMap;

pub mod cd;
pub mod clear;
pub mod hello;
pub mod help;
pub mod theme;

/// Contrat minimal d’une commande interne.
pub trait Command: Send + Sync {
    /// Nom canonique (clé d’invocation), ex: "help".
    fn name(&self) -> &'static str;

    /// Brève description (pour liste/overview).
    fn about(&self) -> &'static str;

    /// Syntaxe d’utilisation (ex: "help [command]").
    fn usage(&self) -> &'static str {
        self.name()
    }

    /// Alias éventuels (ex: ["h"] pour help).
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    /// Point d’entrée : exécute la commande.
    /// `registry` est passé pour les commandes qui veulent introspecter (ex: help).
    fn execute(&self, args: &[&str], registry: &CommandRegistry);
}

/// Registre central des commandes internes.
pub struct CommandRegistry {
    /// commandes par nom canonique
    commands: HashMap<String, Box<dyn Command>>,
    /// alias -> nom canonique
    alias_map: HashMap<String, String>,
}

impl CommandRegistry {
    /// Construit le registre de base (sans dépendances particulières).
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            alias_map: HashMap::new(),
        };

        // Enregistre ici toutes les commandes "simples"
        registry.register(hello::HelloCommand);
        registry.register(clear::ClearCommand);
        registry.register(cd::CdCommand);
        // `help` utilise le registry en lecture, mais on lui passe `&registry` à l'exécution
        registry.register(help::HelpCommand);
        // `theme` nécessitera l’accès au Prompt => voir new_with_prompt dans ton code si besoin

        registry
    }

    /// Si tu as besoin d’injecter un Prompt (Arc<Mutex<Prompt>>) pour certaines commandes,
    /// ajoute ici leur enregistrement (ex: ThemeCommand { prompt }).
    pub fn new_with_prompt(
        prompt: std::sync::Arc<std::sync::Mutex<crate::shell::prompt::Prompt>>,
    ) -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            alias_map: HashMap::new(),
        };

        registry.register(hello::HelloCommand);
        registry.register(clear::ClearCommand);
        registry.register(cd::CdCommand);
        registry.register(help::HelpCommand);
        registry.register(theme::ThemeCommand { prompt });

        registry
    }

    /// Enregistre une commande et renseigne ses alias.
    pub fn register<C: Command + 'static>(&mut self, cmd: C) {
        let name = cmd.name().to_string();
        let aliases = cmd.aliases();

        self.commands.insert(name.clone(), Box::new(cmd));
        for &al in aliases {
            self.alias_map.insert(al.to_string(), name.clone());
        }
    }

    /// Résout un nom (ou alias) vers la commande interne.
    fn resolve(&self, name_or_alias: &str) -> Option<&Box<dyn Command>> {
        if let Some(c) = self.commands.get(name_or_alias) {
            return Some(c);
        }
        if let Some(real) = self.alias_map.get(name_or_alias) {
            return self.commands.get(real);
        }
        None
    }

    /// Exécute si c’est une commande interne, sinon retourne false pour laisser la main au système.
    pub fn execute(&self, cmd: &str, args: &[&str]) -> bool {
        if let Some(c) = self.resolve(cmd) {
            c.execute(args, self);
            true
        } else {
            false
        }
    }

    /// Liste (triée) des noms *canoniques* (pour autocomplétion & affichage).
    pub fn list_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.commands.keys().cloned().collect();
        v.sort();
        v
    }

    /// Récupère (nom, about, usage) pour affichage type `help`.
    pub fn list_metadata(&self) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        for (name, cmd) in &self.commands {
            out.push((
                name.clone(),
                cmd.about().to_string(),
                cmd.usage().to_string(),
            ));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Proposition simple si commande inconnue (distance d’édition minimale).
    pub fn suggest(&self, unknown: &str) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for name in self.commands.keys() {
            let d = levenshtein(unknown, name);
            if best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
                best = Some((d, name.clone()));
            }
        }
        best.and_then(|(d, s)| if d <= 2 { Some(s) } else { None })
    }
}

/// Levenshtein minimaliste (pour une proposition "Did you mean ...?")
fn levenshtein(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}
