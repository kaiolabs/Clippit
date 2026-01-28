# Arquitetura Detalhada - Clippit

## 📐 Visão Geral Arquitetural

Clippit segue uma **arquitetura de microserviços modular** onde:
- Um **daemon central** gerencia o estado e coordena operações
- Múltiplos **clientes UI** se comunicam via IPC
- **Separação clara** entre lógica de negócio, apresentação e infraestrutura

## 🏗️ Diagrama de Componentes

```mermaid
graph TB
    subgraph OS[Sistema Operacional Linux]
        WC[Wayland Clipboard]
        DP[Desktop Portals]
        IBUS[IBus Framework]
        FS[Filesystem]
    end
    
    subgraph DAEMON[clippit-daemon Background Service]
        MON[Monitor<br/>clipboard polling]
        HK[Hotkey Handler<br/>global shortcuts]
        TM[Typing Monitor<br/>autocomplete]
        IPCS[IPC Server<br/>Unix socket]
    end
    
    subgraph CORE[clippit-core Shared Library]
        CFG[Config<br/>TOML]
        HM[HistoryManager<br/>SQLite]
        VAL[Validator]
        TYPES[Types<br/>Definitions]
    end
    
    subgraph UI[User Interfaces]
        POPUP[clippit-popup<br/>GTK4]
        DASH[clippit-dashboard<br/>Qt/QML]
        TIP[clippit-tooltip<br/>GTK4]
    end
    
    subgraph INFRA[Infrastructure]
        IPC[clippit-ipc<br/>Protocol]
        IBE[clippit-ibus<br/>Engine]
        QTB[clippit-qt-bridge<br/>Models]
    end
    
    WC -->|arboard| MON
    DP -->|global-hotkey| HK
    IBUS -->|zbus| IBE
    
    MON --> HM
    HK --> IPCS
    TM --> IPCS
    
    HM <--> FS
    CFG <--> FS
    
    IPCS <--> IPC
    IPC <--> POPUP
    IPC <--> DASH
    IPC <--> TIP
    IPC <--> IBE
    
    POPUP -.uses.- CORE
    DASH -.uses.- CORE
    DAEMON -.uses.- CORE
    QTB -.uses.- CORE
    IBE -.uses.- CORE
    
    DASH -.uses.- QTB

    style DAEMON fill:#ff6b6b,stroke:#c92a2a,stroke-width:3px
    style CORE fill:#4ecdc4,stroke:#099268,stroke-width:3px
    style UI fill:#a29bfe,stroke:#6c5ce7,stroke-width:2px
    style INFRA fill:#fdcb6e,stroke:#e17055,stroke-width:2px
```

## 📦 Arquitetura em Camadas

### Layer 1: System Integration
```
┌─────────────────────────────────────────┐
│   Sistema Operacional (Linux)          │
│   - Wayland/X11 Clipboard               │
│   - Desktop Portals (hotkeys)          │
│   - IBus Framework (input method)      │
│   - Filesystem (SQLite, images, config)│
└─────────────────────────────────────────┘
                    ▲
                    │ Native APIs
                    ▼
```

### Layer 2: Core Library (Shared Business Logic)
```
┌─────────────────────────────────────────┐
│         clippit-core                    │
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │ Config   │  │ History  │            │
│  │ Manager  │  │ Manager  │            │
│  └──────────┘  └──────────┘            │
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │ Types &  │  │Content   │            │
│  │ Traits   │  │Validator │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
                    ▲
                    │ Library API
                    ▼
```

### Layer 3: Service Layer (Daemon)
```
┌─────────────────────────────────────────┐
│       clippit-daemon (Service)          │
│                                         │
│  ┌──────────────────┐                  │
│  │ Clipboard Monitor│ ← Polling 80ms   │
│  └──────────────────┘                  │
│                                         │
│  ┌──────────────────┐                  │
│  │ Hotkey Handler   │ ← Global hooks   │
│  └──────────────────┘                  │
│                                         │
│  ┌──────────────────┐                  │
│  │ Typing Monitor   │ ← rdev events    │
│  └──────────────────┘                  │
│                                         │
│  ┌──────────────────┐                  │
│  │ IPC Server       │ ← Unix socket    │
│  └──────────────────┘                  │
└─────────────────────────────────────────┘
                    ▲
                    │ IPC Protocol (JSON)
                    ▼
```

### Layer 4: Infrastructure
```
┌─────────────────────────────────────────┐
│         Infrastructure Layer            │
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │clippit-  │  │clippit-  │            │
│  │ipc       │  │ibus      │            │
│  └──────────┘  └──────────┘            │
│                                         │
│  ┌──────────┐                          │
│  │clippit-  │                          │
│  │qt-bridge │                          │
│  └──────────┘                          │
└─────────────────────────────────────────┘
                    ▲
                    │
                    ▼
```

### Layer 5: Presentation (UI Clients)
```
┌─────────────────────────────────────────┐
│        Presentation Layer               │
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │clippit-  │  │clippit-  │            │
│  │popup     │  │dashboard │            │
│  │(GTK4)    │  │(Qt/QML)  │            │
│  └──────────┘  └──────────┘            │
│                                         │
│  ┌──────────┐                          │
│  │clippit-  │                          │
│  │tooltip   │                          │
│  └──────────┘                          │
└─────────────────────────────────────────┘
```

## 🔄 Fluxos de Dados Detalhados

### Fluxo 1: Captura de Clipboard (Texto)

```mermaid
sequenceDiagram
    participant U as Usuário
    participant W as Wayland
    participant M as Monitor (daemon)
    participant V as Validator
    participant H as HistoryManager
    participant S as SQLite

    U->>W: Copia texto (Ctrl+C)
    W->>W: Atualiza clipboard
    
    loop Polling 80ms
        M->>W: arboard.get_text()
        W-->>M: "conteúdo novo"
    end
    
    M->>M: Compara com último hash
    alt Conteúdo diferente
        M->>V: validate_text(content)
        V-->>M: Ok ou Error
        
        alt Válido
            M->>M: SHA256(content)
            M->>H: add_entry(texto)
            H->>H: Verifica duplicatas (10 últimos)
            
            alt Não duplicado
                H->>S: INSERT INTO history
                S-->>H: entry_id
                H->>H: Prune se > max_entries
            end
        end
    end
```

### Fluxo 2: Captura de Imagem

```mermaid
sequenceDiagram
    participant U as Usuário
    participant W as Wayland
    participant M as Monitor
    participant V as Validator
    participant H as HistoryManager
    participant FS as Filesystem
    participant S as SQLite

    U->>W: Copia imagem (screenshot)
    W->>W: Atualiza clipboard
    
    M->>W: arboard.get_image()
    W-->>M: ImageData (RGBA)
    
    M->>M: Convert to PNG bytes
    M->>V: validate_image(bytes)
    V-->>M: Ok (size check, format)
    
    M->>M: SHA256(bytes)
    M->>M: Optimize (resize se > 2048px)
    M->>M: Create thumbnail 128x128
    
    M->>FS: Save ~/.local/share/clippit/images/{hash}.png
    FS-->>M: Ok
    
    M->>H: add_entry(image_path, thumbnail)
    H->>S: INSERT (sem BLOB, só path)
    S-->>H: entry_id
```

### Fluxo 3: Abertura do Popup

```mermaid
sequenceDiagram
    participant U as Usuário
    participant HK as Hotkey Handler
    participant IPC as IPC Server
    participant P as Popup Process
    participant H as HistoryManager
    participant S as SQLite

    U->>HK: Pressiona Super+V
    HK->>HK: global_hotkey event
    HK->>HK: Check /tmp/clippit-popup.lock
    
    alt Lock não existe
        HK->>P: spawn clippit-popup
        P->>P: Create lock file (PID)
        P->>IPC: QueryHistoryMetadata {limit: 30}
        IPC->>H: get_recent_metadata(30)
        H->>S: SELECT (sem content_data)
        S-->>H: Vec<Entry> (metadata only)
        H-->>IPC: HistoryMetadataResponse
        IPC-->>P: Entries
        P->>P: Render GTK ListBox
        P->>U: Show window
    else Lock existe
        HK->>P: Kill processo (PID do lock)
        HK->>HK: Remove lock file
    end
```

### Fluxo 4: Busca em Tempo Real

```mermaid
sequenceDiagram
    participant U as Usuário
    participant SE as SearchEntry (GTK)
    participant IPC as IPC Client
    participant IPCS as IPC Server
    participant H as HistoryManager
    participant S as SQLite
    participant LB as ListBox (GTK)

    U->>SE: Digite "rust"
    SE->>SE: on_changed signal
    SE->>IPC: SearchHistory {query: "rust"}
    IPC->>IPCS: JSON message
    IPCS->>H: search("rust")
    H->>S: SELECT WHERE content_text LIKE '%rust%'
    S-->>H: Vec<Entry>
    H-->>IPCS: SearchHistoryResponse
    IPCS-->>IPC: JSON response
    IPC-->>SE: Entries
    SE->>LB: Clear + populate
    LB->>U: Exibe resultados
```

### Fluxo 5: Autocomplete Global

```mermaid
sequenceDiagram
    participant U as Usuário
    participant APP as Qualquer App
    participant IBUS as IBus Framework
    participant IE as clippit-ibus Engine
    participant IPC as IPC Server
    participant TM as Typing Monitor
    participant H as HistoryManager
    participant POPUP as Floating Popup

    U->>APP: Digite "cód"
    APP->>IBUS: Input event
    IBUS->>IE: process_key_press('d')
    IE->>IE: Buffer += 'd' → "cód"
    IE->>IE: current_word() → "cód"
    
    alt word.len() >= 2
        IE->>IPC: RequestAutocompleteSuggestions
        IPC->>TM: get_suggestions("cód")
        TM->>H: search("cód%")
        H-->>TM: Vec<Match>
        TM->>TM: Extract words, score, sort
        TM-->>IPC: AutocompleteSuggestions
        IPC-->>IE: Suggestions
        IE->>POPUP: Show yad/tooltip popup
        POPUP->>U: Display ["código", "códigos"]
    end
    
    U->>APP: Pressiona Tab
    APP->>IBUS: Tab event
    IBUS->>IE: process_key_press(Tab)
    IE->>IPC: AcceptSuggestion
    IPC->>TM: inject_text("código")
    TM->>TM: xdotool type "código"
    TM->>POPUP: Hide popup
```

## 🗄️ Modelo de Dados

### SQLite Schema

```sql
-- Tabela principal
CREATE TABLE clipboard_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,           -- "Text" ou "Image"
    content_text TEXT,                    -- Conteúdo se texto
    content_data BLOB,                    -- DEPRECATED (legacy)
    image_path TEXT,                      -- Path para arquivo PNG
    thumbnail_data BLOB,                  -- Thumbnail 128x128
    timestamp TEXT NOT NULL               -- RFC3339 UTC
);

-- Índice para queries rápidas
CREATE INDEX idx_timestamp ON clipboard_history(timestamp DESC);

-- Query comum: últimas 100 entradas
SELECT id, content_type, content_text, image_path, thumbnail_data, timestamp
FROM clipboard_history
ORDER BY timestamp DESC
LIMIT 100;

-- Query de busca
SELECT * FROM clipboard_history
WHERE content_text LIKE '%query%' OR image_path LIKE '%query%'
ORDER BY timestamp DESC;
```

### Estruturas Rust Principais

```rust
// clippit-core/src/types.rs
pub enum ContentType {
    Text,
    Image,
}

pub struct ClipboardEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: Option<String>,
    pub content_data: Option<Vec<u8>>,      // Legacy
    pub image_path: Option<String>,
    pub thumbnail_data: Option<Vec<u8>>,
    pub timestamp: DateTime<Utc>,
}

// clippit-ipc/src/protocol.rs
pub enum IpcMessage {
    QueryHistory { limit: usize },
    QueryHistoryMetadata { limit: usize, offset: usize },
    SearchHistory { query: String },
    GetEntryData { id: i64 },
    SelectItem { id: i64 },
    RequestAutocompleteSuggestions { 
        partial_word: String, 
        context: AppContext,
        max_results: usize 
    },
    AcceptSuggestion { 
        suggestion: String, 
        partial_word: String 
    },
    // ...
}

pub enum IpcResponse {
    Ok,
    Error { message: String },
    HistoryResponse { entries: Vec<HistoryEntry> },
    HistoryMetadataResponse { entries: Vec<HistoryEntry> },
    SearchHistoryResponse { entries: Vec<HistoryEntry> },
    AutocompleteSuggestions { 
        suggestions: Vec<Suggestion>, 
        query: String 
    },
    // ...
}
```

## 🔐 Considerações de Segurança

### 1. Validação de Inputs
- **Texto**: Máximo 10MB, UTF-8 válido
- **Imagem**: Máximo 50MB, formatos PNG/JPEG
- **Paths**: Canonicalizados antes de uso

### 2. SQL Injection Protection
- Uso exclusivo de **prepared statements**
- Parâmetros sempre passados via bindings

### 3. IPC Security
- Socket Unix local (`/tmp/clippit.sock`)
- Permissões: apenas owner pode conectar
- Sem autenticação (local user trust)

### 4. Privacidade
- Apps ignorados configuráveis (password managers)
- Histórico local (não sincronizado por padrão)
- Opção de limpar histórico

### 5. Resource Limits
- Max 100 entradas por padrão
- Pruning automático
- Memory limits em buffers

## ⚡ Otimizações de Performance

### 1. Database
- **Índice em timestamp**: Queries ordenadas rápidas
- **Metadata queries**: Sem carregar BLOBs desnecessários
- **Batch operations**: Pruning em lote

### 2. Images
- **Filesystem storage**: Não infla banco SQLite
- **Hash deduplication**: SHA256 evita duplicatas
- **Thumbnails**: Preview rápido sem carregar imagem completa
- **Lazy loading**: Carrega imagem completa só quando necessário

### 3. IPC
- **JSON streaming**: Uma mensagem por linha
- **Selective queries**: Metadata vs full data
- **Connection pooling**: Reutiliza conexões

### 4. UI
- **Infinite scroll**: Carrega 30 itens iniciais + 20 on-demand
- **Skeleton loaders**: Feedback visual durante carregamento
- **Debounce**: Busca com 300ms delay
- **Virtual scrolling**: Renderiza apenas itens visíveis

### 5. Autocomplete
- **Cache**: Top 1000 palavras em memória
- **Buffer stale**: Limpa após 5s inatividade
- **Throttle**: Mínimo 2 caracteres para sugerir

## 🔄 Padrões de Concorrência

### Tokio Async Runtime
```rust
// daemon/main.rs
#[tokio::main]
async fn main() {
    let history = Arc::new(Mutex::new(HistoryManager::new()));
    
    // Task 1: Clipboard monitor
    let h1 = history.clone();
    tokio::spawn(async move {
        monitor::start_monitor(h1).await;
    });
    
    // Task 2: Hotkey handler
    let h2 = history.clone();
    tokio::spawn(async move {
        hotkey::start_hotkey_handler(h2).await;
    });
    
    // Task 3: IPC server (main thread)
    ipc_server.run(history).await;
}
```

### Thread Safety
- `Arc<Mutex<>>` para shared state
- `tokio::sync::mpsc` para channels
- Lock granularity mínima

## 🔗 Links para Documentação Relacionada

- **Visão Geral**: [00-PROJECT-OVERVIEW.md](./00-PROJECT-OVERVIEW.md)
- **Padrões de Desenvolvimento**: [02-DEVELOPMENT-STANDARDS.md](./02-DEVELOPMENT-STANDARDS.md)
- **IPC Protocol**: [infrastructure/IPC-PROTOCOL.md](./infrastructure/IPC-PROTOCOL.md)
- **History Storage**: [core/HISTORY-STORAGE.md](./core/HISTORY-STORAGE.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
