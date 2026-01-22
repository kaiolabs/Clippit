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
        // Extrair palavras e contar frequência
        for entry in entries {
            if let Some(ref text) = entry.content_text {
                for word in text.split_whitespace() {
                    let cleaned = word.trim_matches(|c: char| !c.is_alphabetic());
                    let word_lower = cleaned.to_lowercase();
                    if word_lower.len() >= 3 {
                        *self.history_words.entry(word_lower).or_insert(0) += 1;
                    }
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
