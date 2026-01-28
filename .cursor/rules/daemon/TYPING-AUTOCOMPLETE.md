# clippit-daemon - Typing Monitor & Autocomplete

## 📍 Localização
`crates/clippit-daemon/src/typing_monitor.rs`
`crates/clippit-daemon/src/autocomplete_manager.rs`

## 🎯 Responsabilidade

Monitora digitação global via `rdev` e gera sugestões de autocomplete baseadas no histórico.

## 🔄 Fluxo

### TypingMonitor

```rust
pub async fn start(history: Arc<Mutex<HistoryManager>>) {
    let buffer = Arc::new(Mutex::new(TypingBuffer::new()));
    let autocomplete = Arc::new(AutocompleteManager::new());
    
    // Thread separada para rdev (blocking)
    let buf_clone = buffer.clone();
    thread::spawn(move || {
        rdev::listen(move |event| {
            process_event(event, &buf_clone, &autocomplete);
        }).unwrap();
    });
    
    // Task de limpeza de buffer
    tokio::spawn(async move {
        loop {
            buffer.lock().unwrap().clean_stale();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn process_event(
    event: Event,
    buffer: &Arc<Mutex<TypingBuffer>>,
    autocomplete: &Arc<AutocompleteManager>
) {
    match event.event_type {
        EventType::KeyPress(key) => {
            let mut buf = buffer.lock().unwrap();
            
            match key {
                Key::Char(c) => {
                    buf.push_char(c);
                    
                    if let Some(word) = buf.current_word() {
                        if word.len() >= 2 {
                            let suggestions = get_suggestions(&word);
                            autocomplete.show(suggestions);
                        }
                    }
                }
                Key::Space => buf.clear(),
                Key::Backspace => buf.pop_char(),
                Key::Tab => autocomplete.accept_current(),
                // ...
            }
        }
        _ => {}
    }
}
```

### TypingBuffer

```rust
pub struct TypingBuffer {
    buffer: VecDeque<char>,  // Circular buffer (50 chars)
    last_update: Instant,
}

impl TypingBuffer {
    pub fn current_word(&self) -> Option<String> {
        // Extrai palavra atual (reverso até espaço/pontuação)
        let chars: Vec<char> = self.buffer.iter()
            .rev()
            .take_while(|c| c.is_alphanumeric())
            .collect();
        
        if chars.is_empty() {
            None
        } else {
            Some(chars.iter().rev().collect())
        }
    }
    
    pub fn is_stale(&self) -> bool {
        self.last_update.elapsed() > Duration::from_secs(5)
    }
}
```

### AutocompleteManager

```rust
pub struct AutocompleteManager {
    current_suggestions: Arc<Mutex<Vec<Suggestion>>>,
    selected_index: Arc<Mutex<usize>>,
    popup_pid: Arc<Mutex<Option<u32>>>,
}

impl AutocompleteManager {
    pub fn show(&self, suggestions: Vec<Suggestion>) {
        // Kill popup anterior
        if let Some(pid) = *self.popup_pid.lock().unwrap() {
            Command::new("kill").arg(pid.to_string()).status().ok();
        }
        
        // Mostra novo popup
        let popup = Command::new("yad")
            .arg("--text-info")
            .arg("--no-focus")
            .arg("--timeout=3")
            .stdin(Stdio::piped())
            .spawn()
            .ok()?;
        
        // Escreve sugestões
        let mut stdin = popup.stdin.as_ref()?;
        for s in &suggestions {
            writeln!(stdin, "{}", s.word).ok();
        }
        
        *self.popup_pid.lock().unwrap() = Some(popup.id());
    }
    
    pub fn accept_current(&self) {
        let suggestions = self.current_suggestions.lock().unwrap();
        let index = *self.selected_index.lock().unwrap();
        
        if let Some(suggestion) = suggestions.get(index) {
            // Apaga palavra parcial
            for _ in 0..partial_word.len() {
                Command::new("xdotool")
                    .arg("key").arg("BackSpace")
                    .status().ok();
            }
            
            // Injeta palavra completa
            Command::new("xdotool")
                .arg("type").arg(&suggestion.word)
                .status().ok();
        }
    }
}
```

## 📝 Geração de Sugestões

```rust
fn get_suggestions(partial: &str) -> Vec<Suggestion> {
    let history = HISTORY.lock().unwrap();
    let entries = history.search(partial).unwrap();
    
    let mut words = HashSet::new();
    for entry in entries {
        if let Some(text) = &entry.content_text {
            for word in text.split_whitespace() {
                if word.to_lowercase().starts_with(&partial.to_lowercase()) {
                    words.insert(word.to_string());
                }
            }
        }
    }
    
    words.into_iter()
        .take(5)
        .map(|w| Suggestion { word: w, score: 100 })
        .collect()
}
```

## 🔗 Links
- [Daemon Overview](./DAEMON-OVERVIEW.md)
- [Autocomplete Feature](../features/AUTOCOMPLETE-GLOBAL.md)
- [IBus Engine](../infrastructure/IBUS-ENGINE.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
