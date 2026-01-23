use anyhow::Result;
use clippit_core::{Config, HistoryManager};
use clippit_ipc::protocol::{Suggestion, SuggestionSource};
use rdev::{listen, Event, EventType, Key};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

use crate::autocomplete_manager::AutocompleteManager;

/// Buffer de digitação atual
#[derive(Debug, Clone)]
struct TypingBuffer {
    chars: VecDeque<char>,
    last_update: Instant,
    max_size: usize,
}

impl TypingBuffer {
    fn new() -> Self {
        Self {
            chars: VecDeque::new(),
            last_update: Instant::now(),
            max_size: 50, // Máximo de 50 caracteres no buffer
        }
    }

    fn push_char(&mut self, c: char) {
        if self.chars.len() >= self.max_size {
            self.chars.pop_front();
        }
        self.chars.push_back(c);
        self.last_update = Instant::now();
    }

    fn pop_char(&mut self) {
        self.chars.pop_back();
        self.last_update = Instant::now();
    }

    fn clear(&mut self) {
        self.chars.clear();
        self.last_update = Instant::now();
    }

    /// Obtém a palavra atual (últimos caracteres até encontrar espaço)
    fn current_word(&self) -> String {
        self.chars
            .iter()
            .rev()
            .take_while(|&&c| !c.is_whitespace())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    fn is_stale(&self) -> bool {
        self.last_update.elapsed() > Duration::from_secs(5)
    }
}

/// Monitor de eventos de teclado global usando rdev
pub struct TypingMonitor {
    history_manager: Arc<Mutex<HistoryManager>>,
    typing_buffer: Arc<Mutex<TypingBuffer>>,
    config: Arc<Mutex<Config>>,
    autocomplete_manager: Arc<AutocompleteManager>,
}

impl TypingMonitor {
    pub fn new(history_manager: Arc<Mutex<HistoryManager>>) -> Self {
        let config = Config::load().unwrap_or_default();
        
        Self {
            history_manager,
            typing_buffer: Arc::new(Mutex::new(TypingBuffer::new())),
            config: Arc::new(Mutex::new(config)),
            autocomplete_manager: Arc::new(AutocompleteManager::new()),
        }
    }

    /// Inicia o monitor de teclado
    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("🎹 Iniciando monitor de teclado para autocompletar...");

        // Verificar se autocompletar está habilitado
        let config = self.config.lock().unwrap();
        if !config.autocomplete.enabled {
            info!("⚠️  Autocompletar desabilitado na configuração");
            return Ok(());
        }
        drop(config);

        // Task de limpeza de buffer antigo
        let buffer_clone = Arc::clone(&self.typing_buffer);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                let mut buffer = buffer_clone.lock().unwrap();
                if buffer.is_stale() && !buffer.chars.is_empty() {
                    debug!("🧹 Limpando buffer antigo");
                    buffer.clear();
                }
            }
        });

        // Capturar eventos de teclado em thread separada (rdev é blocking)
        let self_clone = Arc::clone(&self);
        std::thread::spawn(move || {
            if let Err(e) = listen(move |event| {
                self_clone.handle_keyboard_event(event);
            }) {
                error!("❌ Erro no monitor de teclado: {:?}", e);
            }
        });

        info!("✅ Monitor de teclado iniciado!");
        Ok(())
    }

    /// Processa evento de teclado
    fn handle_keyboard_event(&self, event: Event) {
        // Apenas processar eventos de tecla pressionada
        if let EventType::KeyPress(key) = event.event_type {
            self.process_key_press(key);
        }
    }

    /// Processa tecla pressionada
    fn process_key_press(&self, key: Key) {
        let mut buffer = self.typing_buffer.lock().unwrap();
        
        match key {
            // Caracteres alfanuméricos
            Key::KeyA => buffer.push_char('a'),
            Key::KeyB => buffer.push_char('b'),
            Key::KeyC => buffer.push_char('c'),
            Key::KeyD => buffer.push_char('d'),
            Key::KeyE => buffer.push_char('e'),
            Key::KeyF => buffer.push_char('f'),
            Key::KeyG => buffer.push_char('g'),
            Key::KeyH => buffer.push_char('h'),
            Key::KeyI => buffer.push_char('i'),
            Key::KeyJ => buffer.push_char('j'),
            Key::KeyK => buffer.push_char('k'),
            Key::KeyL => buffer.push_char('l'),
            Key::KeyM => buffer.push_char('m'),
            Key::KeyN => buffer.push_char('n'),
            Key::KeyO => buffer.push_char('o'),
            Key::KeyP => buffer.push_char('p'),
            Key::KeyQ => buffer.push_char('q'),
            Key::KeyR => buffer.push_char('r'),
            Key::KeyS => buffer.push_char('s'),
            Key::KeyT => buffer.push_char('t'),
            Key::KeyU => buffer.push_char('u'),
            Key::KeyV => buffer.push_char('v'),
            Key::KeyW => buffer.push_char('w'),
            Key::KeyX => buffer.push_char('x'),
            Key::KeyY => buffer.push_char('y'),
            Key::KeyZ => buffer.push_char('z'),
            
            // Números
            Key::Num0 => buffer.push_char('0'),
            Key::Num1 => buffer.push_char('1'),
            Key::Num2 => buffer.push_char('2'),
            Key::Num3 => buffer.push_char('3'),
            Key::Num4 => buffer.push_char('4'),
            Key::Num5 => buffer.push_char('5'),
            Key::Num6 => buffer.push_char('6'),
            Key::Num7 => buffer.push_char('7'),
            Key::Num8 => buffer.push_char('8'),
            Key::Num9 => buffer.push_char('9'),
            
            // Espaço e caracteres especiais
            Key::Space => {
                buffer.push_char(' ');
                // Limpar buffer após espaço (nova palavra)
                buffer.clear();
            }
            
            // Backspace
            Key::Backspace => {
                buffer.pop_char();
            }
            
            // Tab - aceitar sugestão
            Key::Tab => {
                drop(buffer);
                if let Err(e) = self.autocomplete_manager.accept_current_suggestion() {
                    error!("❌ Erro ao aceitar sugestão: {}", e);
                }
                return;
            }
            
            // Setas - navegar sugestões
            Key::DownArrow => {
                self.autocomplete_manager.next_suggestion();
                return;
            }
            Key::UpArrow => {
                self.autocomplete_manager.previous_suggestion();
                return;
            }
            
            // Enter (nova linha = nova palavra)
            Key::Return | Key::ControlLeft | Key::ControlRight => {
                buffer.clear();
                self.autocomplete_manager.clear_suggestions();
            }
            
            _ => {
                // Outras teclas não nos interessam para autocompletar
                return;
            }
        }

        // Obter palavra atual
        let current_word = buffer.current_word();
        drop(buffer); // Liberar lock antes de chamar async

        // Verificar se deve buscar sugestões
        let config = self.config.lock().unwrap();
        let min_chars = config.autocomplete.min_chars;
        let max_suggestions = config.autocomplete.max_suggestions;
        drop(config);

        if current_word.len() >= min_chars {
            debug!("🔍 Buscando sugestões para: '{}'", current_word);
            
            // Buscar sugestões do histórico
            if let Ok(suggestions) = self.get_suggestions(&current_word, max_suggestions) {
                if !suggestions.is_empty() {
                    info!("💡 Encontradas {} sugestões para '{}'", suggestions.len(), current_word);
                    self.show_suggestions(suggestions, current_word.clone());
                } else {
                    // Limpar sugestões se não houver nenhuma
                    self.autocomplete_manager.clear_suggestions();
                }
            } else {
                self.autocomplete_manager.clear_suggestions();
            }
        }
    }

    /// Busca sugestões do histórico
    fn get_suggestions(&self, partial_word: &str, max_results: usize) -> Result<Vec<Suggestion>> {
        let manager = self.history_manager.lock().unwrap();
        let entries = manager.search(partial_word)?;

        let mut suggestions: Vec<Suggestion> = Vec::new();
        let partial_lower = partial_word.to_lowercase();

        for entry in entries.iter().take(max_results * 3) {
            if let Some(text) = &entry.content_text {
                // Extrair palavras do texto
                for word in text.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    
                    // Filtrar palavras muito curtas ou que não começam com o prefixo
                    if word.len() < 3 || !word_lower.starts_with(&partial_lower) {
                        continue;
                    }
                    
                    // Filtrar palavras técnicas/lixo
                    if word.contains("::") || word.contains("->") || 
                       word.contains("__") || word.len() > 30 {
                        continue;
                    }
                    
                    // Calcular score (quanto mais próximo do início, maior o score)
                    let score = 100 - (word.len() as i64 - partial_word.len() as i64).abs();
                    
                    suggestions.push(Suggestion {
                        word: word.to_string(),
                        score,
                        source: SuggestionSource::History,
                    });
                    
                    if suggestions.len() >= max_results {
                        break;
                    }
                }
                
                if suggestions.len() >= max_results {
                    break;
                }
            }
        }

        // Ordenar por score
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));
        suggestions.truncate(max_results);

        Ok(suggestions)
    }

    /// Mostra sugestões via gerenciador
    fn show_suggestions(&self, suggestions: Vec<Suggestion>, partial_word: String) {
        // Log para debug
        for (i, sugg) in suggestions.iter().enumerate() {
            debug!("  {}. {} (score: {})", i + 1, sugg.word, sugg.score);
        }

        // Usar gerenciador de autocomplete
        if let Err(e) = self.autocomplete_manager.show_suggestions(suggestions, partial_word) {
            error!("❌ Erro ao mostrar sugestões: {}", e);
        }
    }
}
