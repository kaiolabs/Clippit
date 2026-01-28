use std::time::{Duration, Instant};

/// Buffer que mantém o contexto de digitação atual
#[allow(dead_code)] // Código preparado para uso futuro
#[derive(Debug, Clone)]
pub struct TypingBuffer {
    /// Texto atualmente sendo digitado
    buffer: String,
    /// Timestamp da última modificação
    last_update: Instant,
    /// Posição do cursor no buffer
    cursor_pos: usize,
}

#[allow(dead_code)] // Métodos preparados para uso futuro com IBus
impl TypingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            last_update: Instant::now(),
            cursor_pos: 0,
        }
    }

    /// Adiciona um caractere ao buffer
    pub fn push_char(&mut self, c: char) {
        self.buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.last_update = Instant::now();
    }

    /// Remove o último caractere (backspace)
    pub fn pop_char(&mut self) -> Option<char> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let removed = self.buffer.remove(self.cursor_pos);
            self.last_update = Instant::now();
            Some(removed)
        } else {
            None
        }
    }

    /// Limpa o buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
        self.last_update = Instant::now();
    }

    /// Retorna a palavra atual sendo digitada
    ///
    /// Uma palavra é definida como uma sequência de caracteres alfanuméricos
    /// sem espaços ou pontuação
    pub fn current_word(&self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }

        // Encontrar o início da palavra atual (última posição antes do cursor que é whitespace/punctuation)
        let start = self.buffer[..self.cursor_pos]
            .rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|i| i + 1)
            .unwrap_or(0);

        // Encontrar o fim da palavra (próxima posição após cursor que é whitespace/punctuation)
        let end = self.buffer[self.cursor_pos..]
            .find(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|i| self.cursor_pos + i)
            .unwrap_or(self.buffer.len());

        let word = &self.buffer[start..end];

        if word.is_empty() {
            None
        } else {
            Some(word.to_string())
        }
    }

    /// Retorna o buffer completo
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Retorna o tempo desde a última atualização
    pub fn time_since_update(&self) -> Duration {
        self.last_update.elapsed()
    }

    /// Verifica se o buffer deve ser considerado inativo (nenhuma digitação há mais de 5s)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.time_since_update() > timeout
    }

    /// Retorna o número de caracteres da palavra atual
    pub fn current_word_len(&self) -> usize {
        self.current_word().map(|w| w.len()).unwrap_or(0)
    }

    /// Substitui a palavra atual por uma nova
    pub fn replace_current_word(&mut self, new_word: &str) -> Result<(), String> {
        let current = match self.current_word() {
            Some(w) => w,
            None => return Err("No current word to replace".to_string()),
        };

        // Encontrar início da palavra
        let start = self.buffer[..self.cursor_pos]
            .rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = start + current.len();

        // Substituir
        self.buffer.replace_range(start..end, new_word);
        self.cursor_pos = start + new_word.len();
        self.last_update = Instant::now();

        Ok(())
    }
}

impl Default for TypingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_current_word() {
        let mut buffer = TypingBuffer::new();
        buffer.push_char('h');
        buffer.push_char('e');
        buffer.push_char('l');
        buffer.push_char('l');
        buffer.push_char('o');

        assert_eq!(buffer.current_word(), Some("hello".to_string()));
    }

    #[test]
    fn test_backspace() {
        let mut buffer = TypingBuffer::new();
        buffer.push_char('t');
        buffer.push_char('e');
        buffer.push_char('s');
        buffer.push_char('t');

        buffer.pop_char();
        assert_eq!(buffer.current_word(), Some("tes".to_string()));
    }

    #[test]
    fn test_current_word_with_spaces() {
        let mut buffer = TypingBuffer::new();
        for c in "hello world".chars() {
            buffer.push_char(c);
        }

        assert_eq!(buffer.current_word(), Some("world".to_string()));
    }

    #[test]
    fn test_replace_current_word() {
        let mut buffer = TypingBuffer::new();
        for c in "test".chars() {
            buffer.push_char(c);
        }

        buffer.replace_current_word("testing").unwrap();
        assert_eq!(buffer.text(), "testing");
    }
}
