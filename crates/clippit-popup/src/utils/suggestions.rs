use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::collections::HashMap;
use clippit_ipc::HistoryEntry;

pub struct SuggestionEngine {
    history_words: HashMap<String, u32>,  // Palavra -> Frequência
    fuzzy_matcher: SkimMatcherV2,         // Fuzzy matcher
}

impl SuggestionEngine {
    pub fn new() -> Self {
        eprintln!("✅ Suggestion engine initialized with fuzzy matching");
        
        Self {
            history_words: HashMap::new(),
            fuzzy_matcher: SkimMatcherV2::default(),
        }
    }
    
    pub fn update_history_words(&mut self, entries: &[HistoryEntry]) {
        eprintln!("📚 Carregando palavras do histórico para sugestões...");
        eprintln!("📥 Recebidas {} entradas do histórico", entries.len());
        
        // Lista de palavras genéricas/lixo para ignorar
        let blacklist = [
            "resultado", "resultados", "entry", "entries", "image", "text", 
            "content", "clipboard", "timestamp", "type", "true", "false",
            "has", "processing", "created", "removed", "added", "loaded"
        ];
        
        // Extrair palavras e contar frequência
        for entry in entries {
            if let Some(ref text) = entry.content_text {
                // Ignorar textos muito longos (provavelmente logs/stack traces)
                if text.len() > 1000 {
                    continue;
                }
                
                for word in text.split_whitespace() {
                    // URLs são úteis - adicionar inteiras (sem limpar)
                    let is_url = word.starts_with("http://") || word.starts_with("https://") 
                                 || word.starts_with("www.");
                    
                    if is_url {
                        // URLs válidas - adicionar completas (case-insensitive)
                        let url_lower = word.to_lowercase();
                        if url_lower.len() >= 3 && url_lower.len() <= 100 {
                            *self.history_words.entry(url_lower).or_insert(0) += 1;
                        }
                        continue;
                    }
                    
                    // Para palavras normais, limpar pontuação
                    let cleaned = word.trim_matches(|c: char| !c.is_alphabetic());
                    let word_lower = cleaned.to_lowercase();
                    
                    // Filtros de qualidade
                    if word_lower.len() < 3 || word_lower.len() > 30 {
                        continue; // Muito curta ou muito longa
                    }
                    
                    // Ignorar se contém caracteres técnicos (stack traces, paths do sistema)
                    // MAS permitir URLs (já tratadas acima)
                    if word.contains("::") || word.contains("->") || word.contains("__") 
                       || word.contains("()") || word.contains("{}") || word.contains("[]") {
                        continue;
                    }
                    
                    // Ignorar paths do sistema (/, \) mas não URLs
                    if (word.contains("/") || word.contains("\\")) && !is_url {
                        continue;
                    }
                    
                    // Ignorar palavras da blacklist
                    if blacklist.contains(&word_lower.as_str()) {
                        continue;
                    }
                    
                    // Ignorar se contém muitos números (provavelmente ID ou hash)
                    let digit_count = word_lower.chars().filter(|c| c.is_numeric()).count();
                    if digit_count > word_lower.len() / 2 {
                        continue;
                    }
                    
                    // ✅ Palavra válida - adicionar ao histórico
                    *self.history_words.entry(word_lower).or_insert(0) += 1;
                }
            }
        }
        eprintln!("📚 Loaded {} unique words from history", self.history_words.len());
    }
    
    pub fn get_suggestions(&self, partial_word: &str, max_results: usize) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Fuzzy match contra histórico
        for (word, freq) in &self.history_words {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(word, partial_word) {
                let boosted_score = score + (*freq as i64 * 100);  // Boost por frequência
                suggestions.push(Suggestion {
                    word: word.clone(),
                    score: boosted_score,
                    source: SuggestionSource::History,
                });
            }
        }
        
        // Ordenar por score e retornar top N
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));
        suggestions.truncate(max_results);
        suggestions
    }
}

#[derive(Clone)]
pub struct Suggestion {
    pub word: String,
    pub score: i64,
    pub source: SuggestionSource,
}

#[derive(Clone, Copy)]
pub enum SuggestionSource {
    History,        // Do histórico de clipboard (ícone verde)
}
