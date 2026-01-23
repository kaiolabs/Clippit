use anyhow::Result;
use clippit_core::{Config, HistoryManager};
use clippit_ipc::{AppContext, Suggestion, SuggestionSource};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info};

/// Buffer de digitação por aplicação
struct TypingState {
    current_word: String,
    last_update: Instant,
    app_context: AppContext,
}

/// Monitor de eventos de digitação do IBus
pub struct TypingMonitor {
    history_manager: Arc<Mutex<HistoryManager>>,
    /// Buffers de digitação por app (app_name -> estado)
    typing_states: Arc<Mutex<HashMap<String, TypingState>>>,
    /// Cache de palavras frequentes
    frequent_words_cache: Arc<Mutex<HashMap<String, i64>>>,
}

impl TypingMonitor {
    pub fn new(history_manager: Arc<Mutex<HistoryManager>>) -> Self {
        Self {
            history_manager,
            typing_states: Arc::new(Mutex::new(HashMap::new())),
            frequent_words_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Inicia o monitor
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting typing monitor...");

        // Carregar palavras frequentes do histórico para cache
        self.load_frequent_words_cache().await?;

        // Task de limpeza de buffers antigos
        let typing_states = Arc::clone(&self.typing_states);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                
                let mut states = typing_states.lock().unwrap();
                states.retain(|_, state| {
                    state.last_update.elapsed() < Duration::from_secs(10)
                });
            }
        });

        info!("Typing monitor ready");
        
        // O monitor responde via IPC (será integrado no daemon main)
        Ok(())
    }

    /// Processa uma palavra parcial e retorna sugestões
    pub async fn get_suggestions(
        &self,
        partial_word: &str,
        context: &AppContext,
        max_results: usize,
    ) -> Result<Vec<Suggestion>> {
        let config = Config::load().unwrap_or_default();

        // Verificar se autocomplete está habilitado
        if !config.autocomplete.enabled {
            return Ok(vec![]);
        }

        // Verificar caracteres mínimos
        if partial_word.len() < config.autocomplete.min_chars {
            return Ok(vec![]);
        }

        // Verificar se app está na lista de ignorados
        if config.autocomplete.ignored_apps.contains(&context.app_name) {
            debug!("App {} is in ignored list", context.app_name);
            return Ok(vec![]);
        }

        // Verificar se é campo de senha
        if !config.autocomplete.show_in_passwords {
            if let Some(field_type) = &context.input_field_type {
                if field_type == "password" {
                    debug!("Password field detected, skipping autocomplete");
                    return Ok(vec![]);
                }
            }
        }

        debug!("Getting suggestions for: {}", partial_word);

        let mut suggestions = Vec::new();

        // 1. Buscar no histórico
        suggestions.extend(self.get_history_suggestions(partial_word).await?);

        // 2. Buscar no cache de palavras frequentes
        suggestions.extend(self.get_frequent_suggestions(partial_word).await);

        // 3. Ordenar por score (decrescente)
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));

        // 4. Remover duplicatas (manter o de maior score)
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.word.clone()));

        // 5. Limitar ao máximo
        suggestions.truncate(max_results.min(config.autocomplete.max_suggestions));

        info!("Returning {} suggestions for '{}'", suggestions.len(), partial_word);
        Ok(suggestions)
    }

    /// Busca sugestões no histórico do clipboard
    async fn get_history_suggestions(&self, partial: &str) -> Result<Vec<Suggestion>> {
        let manager = self.history_manager.lock().unwrap();
        
        // Buscar entradas de texto que começam com a palavra parcial
        let entries = manager.search(partial.to_string())?;

        let mut suggestions = Vec::new();
        let mut word_scores: HashMap<String, i64> = HashMap::new();

        for entry in entries.iter().take(100) {
            if let Some(text) = &entry.content_text {
                // Extrair palavras do texto
                for word in text.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    if word_lower.starts_with(&partial.to_lowercase()) && word_lower.len() > partial.len() {
                        // Score baseado em frequência e recência
                        let score = word_scores.entry(word.to_string()).or_insert(0);
                        *score += 10; // +10 por cada aparição
                    }
                }
            }
        }

        for (word, score) in word_scores {
            suggestions.push(Suggestion {
                word,
                score,
                source: SuggestionSource::History,
            });
        }

        Ok(suggestions)
    }

    /// Busca sugestões no cache de palavras frequentes
    async fn get_frequent_suggestions(&self, partial: &str) -> Vec<Suggestion> {
        let cache = self.frequent_words_cache.lock().unwrap();
        let partial_lower = partial.to_lowercase();

        cache
            .iter()
            .filter(|(word, _)| word.to_lowercase().starts_with(&partial_lower))
            .map(|(word, &count)| Suggestion {
                word: word.clone(),
                score: count,
                source: SuggestionSource::Frequency,
            })
            .collect()
    }

    /// Carrega palavras frequentes do histórico para cache
    async fn load_frequent_words_cache(&mut self) -> Result<()> {
        info!("Loading frequent words cache...");

        let manager = self.history_manager.lock().unwrap();
        let entries = manager.get_recent(1000)?;

        let mut word_counts: HashMap<String, i64> = HashMap::new();

        for entry in entries {
            if let Some(text) = entry.content_text {
                for word in text.split_whitespace() {
                    if word.len() >= 3 && word.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        *word_counts.entry(word.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Manter apenas as 1000 mais frequentes
        let mut sorted: Vec<_> = word_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(1000);

        let mut cache = self.frequent_words_cache.lock().unwrap();
        *cache = sorted.into_iter().collect();

        info!("Loaded {} frequent words into cache", cache.len());
        Ok(())
    }

    /// Registra que uma sugestão foi aceita (para aprendizado futuro)
    pub async fn record_suggestion_accepted(&self, suggestion: String, partial: String) {
        debug!("Suggestion accepted: {} (was: {})", suggestion, partial);
        
        // TODO: Implementar aprendizado/tracking
        // - Incrementar contador de uso dessa palavra
        // - Associar com contexto de app
        // - Atualizar cache de frequentes
    }
}
