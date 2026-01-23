use anyhow::Result;
use clippit_ipc::protocol::Suggestion;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Gerencia sugestões ativas e injeção de texto
pub struct AutocompleteManager {
    /// Sugestões atuais
    current_suggestions: Arc<Mutex<Vec<Suggestion>>>,
    /// Índice da sugestão selecionada
    selected_index: Arc<Mutex<usize>>,
    /// Palavra parcial atual
    current_partial: Arc<Mutex<String>>,
}

impl AutocompleteManager {
    pub fn new() -> Self {
        Self {
            current_suggestions: Arc::new(Mutex::new(Vec::new())),
            selected_index: Arc::new(Mutex::new(0)),
            current_partial: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Atualiza sugestões e mostra notificação
    pub fn show_suggestions(&self, suggestions: Vec<Suggestion>, partial_word: String) -> Result<()> {
        if suggestions.is_empty() {
            return Ok(());
        }

        // Salvar sugestões atuais
        *self.current_suggestions.lock().unwrap() = suggestions.clone();
        *self.selected_index.lock().unwrap() = 0;
        *self.current_partial.lock().unwrap() = partial_word.clone();

        // Salvar em arquivo temporário para acesso externo
        self.save_suggestions_file(&suggestions, &partial_word)?;

        // Mostrar notificação
        let text = suggestions
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, s)| {
                if i == 0 {
                    format!("➜ {}", s.word)
                } else {
                    format!("  {}", s.word)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let hint = format!("💡 Sugestões (Tab para aceitar)\n{}", text);

        Command::new("notify-send")
            .arg("Clippit Autocomplete")
            .arg(&hint)
            .arg("-t")
            .arg("3000")
            .arg("-u")
            .arg("low")
            .spawn()
            .ok();

        info!("📋 {} sugestões mostradas para '{}'", suggestions.len(), partial_word);
        Ok(())
    }

    /// Aceita a sugestão atual (injetar texto)
    pub fn accept_current_suggestion(&self) -> Result<()> {
        let suggestions = self.current_suggestions.lock().unwrap();
        let index = *self.selected_index.lock().unwrap();
        let partial = self.current_partial.lock().unwrap();

        if suggestions.is_empty() {
            debug!("❌ Nenhuma sugestão ativa");
            return Ok(());
        }

        let suggestion = &suggestions[index];
        let word_to_inject = &suggestion.word;

        info!("✅ Aceitando sugestão: '{}' (substituindo '{}')", word_to_inject, partial);

        // Apagar palavra parcial (Backspace N vezes)
        let backspaces = partial.len();
        if backspaces > 0 {
            for _ in 0..backspaces {
                Command::new("xdotool")
                    .arg("key")
                    .arg("BackSpace")
                    .output()?;
            }
        }

        // Digitar palavra completa
        Command::new("xdotool")
            .arg("type")
            .arg("--")
            .arg(word_to_inject)
            .output()?;

        info!("✅ Texto injetado: '{}'", word_to_inject);

        // Limpar sugestões
        self.clear_suggestions();

        Ok(())
    }

    /// Navega para próxima sugestão
    pub fn next_suggestion(&self) {
        let suggestions = self.current_suggestions.lock().unwrap();
        if suggestions.is_empty() {
            return;
        }

        let mut index = self.selected_index.lock().unwrap();
        *index = (*index + 1) % suggestions.len();
        debug!("⬇️ Próxima sugestão: {}/{}", *index + 1, suggestions.len());

        // Remo strar notificação com nova seleção
        drop(index);
        drop(suggestions);
        self.refresh_notification();
    }

    /// Navega para sugestão anterior
    pub fn previous_suggestion(&self) {
        let suggestions = self.current_suggestions.lock().unwrap();
        if suggestions.is_empty() {
            return;
        }

        let mut index = self.selected_index.lock().unwrap();
        if *index == 0 {
            *index = suggestions.len() - 1;
        } else {
            *index -= 1;
        }
        debug!("⬆️ Sugestão anterior: {}/{}", *index + 1, suggestions.len());

        // Remostrar notificação com nova seleção
        drop(index);
        drop(suggestions);
        self.refresh_notification();
    }

    /// Limpa sugestões atuais
    pub fn clear_suggestions(&self) {
        self.current_suggestions.lock().unwrap().clear();
        *self.selected_index.lock().unwrap() = 0;
        self.current_partial.lock().unwrap().clear();
        
        // Remover arquivo temporário
        if let Ok(path) = self.get_suggestions_file_path() {
            let _ = fs::remove_file(path);
        }
    }

    /// Salva sugestões em arquivo temporário
    fn save_suggestions_file(&self, suggestions: &[Suggestion], partial: &str) -> Result<()> {
        let path = self.get_suggestions_file_path()?;
        let content = format!(
            "PARTIAL:{}\n{}",
            partial,
            suggestions
                .iter()
                .map(|s| s.word.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
        fs::write(path, content)?;
        Ok(())
    }

    /// Atualiza notificação com nova seleção
    fn refresh_notification(&self) {
        let suggestions = self.current_suggestions.lock().unwrap();
        let index = *self.selected_index.lock().unwrap();

        if suggestions.is_empty() {
            return;
        }

        let text = suggestions
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, s)| {
                if i == index {
                    format!("➜ {}", s.word)
                } else {
                    format!("  {}", s.word)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let hint = format!("💡 Sugestões ({}/{})\n{}", index + 1, suggestions.len(), text);

        Command::new("notify-send")
            .arg("Clippit Autocomplete")
            .arg(&hint)
            .arg("-t")
            .arg("3000")
            .arg("-u")
            .arg("low")
            .spawn()
            .ok();
    }

    fn get_suggestions_file_path(&self) -> Result<PathBuf> {
        let mut path = std::env::temp_dir();
        path.push("clippit-autocomplete-suggestions.txt");
        Ok(path)
    }
}
