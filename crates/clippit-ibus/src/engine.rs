use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use zbus::Connection;

use crate::typing_buffer::TypingBuffer;
use clippit_ipc::IpcClient;

/// Clippit IBus Engine Implementation
/// 
/// Este engine atua como Input Method, capturando keystrokes
/// e enviando-os para o daemon Clippit para processamento
pub struct ClippitEngine {
    /// Buffer de digitação atual
    typing_buffer: Arc<Mutex<TypingBuffer>>,
    /// Conexão DBus
    #[allow(dead_code)]
    connection: Connection,
    /// Cliente IPC para comunicar com o daemon
    #[allow(dead_code)]
    ipc_client: IpcClient,
    /// Estado do engine (habilitado/desabilitado)
    enabled: Arc<Mutex<bool>>,
}

impl ClippitEngine {
    pub async fn new() -> Result<Self> {
        info!("Initializing Clippit IBus Engine...");

        // Conectar ao session bus
        let connection = Connection::session().await?;
        info!("Connected to D-Bus session bus");

        // Criar IPC client
        let ipc_client = IpcClient;

        Ok(Self {
            typing_buffer: Arc::new(Mutex::new(TypingBuffer::new())),
            connection,
            ipc_client,
            enabled: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Clippit IBus Engine running...");

        // Registrar interface IBus no DBus
        // Nota: IBus usa uma interface específica em org.freedesktop.IBus.Engine
        
        // Por enquanto, implementar um loop básico
        // TODO: Implementar interface IBus completa via DBus
        
        loop {
            // Aguardar eventos
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Por enquanto, apenas manter o engine rodando
            // A implementação completa virá nas próximas tarefas
        }
    }

    /// Processa um evento de tecla pressionada
    #[allow(dead_code)] // Será usado quando integrar com IBus
    pub async fn process_key_press(&self, keyval: u32, keycode: u32, state: u32) -> Result<bool> {
        let enabled = *self.enabled.lock().await;
        
        if !enabled {
            // Engine desabilitado, passar tecla direto
            return Ok(false);
        }

        debug!("Key pressed: keyval={}, keycode={}, state={}", keyval, keycode, state);

        // Converter keyval para caractere
        if let Some(c) = self.keyval_to_char(keyval) {
            let mut buffer = self.typing_buffer.lock().await;
            buffer.push_char(c);

            // Se temos palavra com >= 2 caracteres, enviar para daemon
            if buffer.current_word_len() >= 2 {
                if let Some(word) = buffer.current_word() {
                    debug!("Current word: {}", word);
                    
                    // TODO: Enviar via IPC para daemon
                    // TODO: Receber sugestões
                    // TODO: Mostrar popup
                }
            }

            // Passar a tecla adiante (não consumir)
            return Ok(false);
        }

        // Tecla especial
        match keyval {
            0xFF08 => {
                // Backspace
                let mut buffer = self.typing_buffer.lock().await;
                buffer.pop_char();
                Ok(false)
            }
            0xFF0D => {
                // Enter
                let mut buffer = self.typing_buffer.lock().await;
                buffer.clear();
                Ok(false)
            }
            0xFF1B => {
                // Escape
                let mut buffer = self.typing_buffer.lock().await;
                buffer.clear();
                Ok(false)
            }
            _ => Ok(false) // Outras teclas, não processar
        }
    }

    /// Habilita o engine
    #[allow(dead_code)] // Será usado quando integrar com IBus
    pub async fn enable(&self) {
        info!("Enabling Clippit IBus Engine");
        let mut enabled = self.enabled.lock().await;
        *enabled = true;
    }

    /// Desabilita o engine
    #[allow(dead_code)] // Será usado quando integrar com IBus
    pub async fn disable(&self) {
        info!("Disabling Clippit IBus Engine");
        let mut enabled = self.enabled.lock().await;
        *enabled = false;
        
        // Limpar buffer
        let mut buffer = self.typing_buffer.lock().await;
        buffer.clear();
    }

    /// Converte keyval para caractere (simplificado)
    fn keyval_to_char(&self, keyval: u32) -> Option<char> {
        // ASCII simples
        if keyval >= 0x20 && keyval <= 0x7E {
            Some(keyval as u8 as char)
        } else {
            None
        }
    }
}

// TODO: Implementar interface DBus IBus.Engine
// Isso requer expor métodos via zbus macro
