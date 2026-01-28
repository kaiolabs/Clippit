# clippit-core - Padrões de Configuração

## 📍 Localização
`crates/clippit-core/src/config.rs`

## 🎯 Responsabilidade

Gerenciamento centralizado de todas as configurações do Clippit usando arquivo TOML.

### Objetivos
- ✅ **Single Source of Truth**: Todas as configs em um único lugar
- ✅ **Type Safety**: Estruturas fortemente tipadas
- ✅ **Defaults Sensatos**: Funciona out-of-the-box
- ✅ **Validação**: Configs inválidas são rejeitadas
- ✅ **Portabilidade**: Paths XDG-compliant

## 📊 Estrutura Hierárquica

```rust
Config                                    // Struct raiz
├── GeneralConfig
│   ├── max_history_items: usize         // 100
│   ├── poll_interval_ms: u64            // 200ms
│   ├── max_text_size_mb: usize          // 10MB
│   └── max_image_size_mb: usize         // 50MB
│
├── HotkeyConfig
│   ├── modifier: String                 // "super"
│   ├── key: String                      // "v"
│   ├── alt_modifier: Option<String>     // None
│   └── alt_key: Option<String>          // None
│
├── UiConfig
│   ├── theme: String                    // "dark"
│   ├── language: String                 // "en"
│   ├── font_family: String              // "Sans"
│   ├── font_size: u32                   // 11
│   ├── window_width: u32                // 700
│   ├── window_height: u32               // 550
│   └── colors: ThemeColors
│       ├── dark: ColorScheme
│       └── light: ColorScheme
│
├── SearchConfig
│   ├── max_suggestions: usize           // 5
│   └── focus_on_show: bool              // true
│
├── FeaturesConfig
│   ├── capture_text: bool               // true
│   ├── capture_images: bool             // true
│   └── enable_notifications: bool       // true
│
├── PrivacyConfig
│   ├── ignored_apps: Vec<String>        // ["keepassxc", ...]
│   ├── clear_on_exit: bool              // false
│   └── retention_days: Option<u32>      // None
│
├── AdvancedConfig
│   ├── log_level: String                // "info"
│   ├── data_dir: Option<PathBuf>        // None (usa default)
│   └── config_dir: Option<PathBuf>      // None (usa default)
│
└── AutocompleteConfig
    ├── enabled: bool                     // false
    ├── max_suggestions: usize            // 3
    ├── min_chars: usize                  // 2
    ├── delay_ms: u64                     // 300ms
    ├── show_in_passwords: bool           // false
    ├── ignored_apps: Vec<String>         // ["gnome-terminal", ...]
    ├── hotkey_modifier: Option<String>   // Some("ctrl+shift")
    ├── hotkey_key: Option<String>        // Some("a")
    └── ai: AutocompleteAIConfig          // Fase 2
        ├── enabled: bool                  // false
        ├── provider: String               // "local"
        ├── model: String                  // "gpt-4"
        └── api_key: String                // ""
```

## ✅ Padrões Obrigatórios

### 1. Todos os Campos Devem Ter Defaults

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_max_history_items")]
    pub max_history_items: usize,
    
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

fn default_max_history_items() -> usize { 100 }
fn default_poll_interval_ms() -> u64 { 200 }
```

**Por quê?** Configs parciais devem funcionar sem quebrar.

### 2. Serialização via serde + toml

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    
    // ...
}
```

### 3. Path de Configuração Consistente

```rust
use dirs;

pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not determine config directory"))?;
    
    Ok(config_dir.join("clippit").join("config.toml"))
}

pub fn ensure_config_dir() -> Result<PathBuf> {
    let dir = config_path()?.parent().unwrap().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
```

**Paths Padrão**:
- Linux: `~/.config/clippit/config.toml`
- Windows: `%APPDATA%\clippit\config.toml`
- macOS: `~/Library/Application Support/clippit/config.toml`

### 4. Load com Fallback para Defaults

```rust
impl Config {
    /// Carrega config do arquivo, cria com defaults se não existir
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        
        if !path.exists() {
            // Cria config padrão
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        config.validate()
            .context("Config validation failed")?;
        
        Ok(config)
    }
    
    /// Salva config no arquivo
    pub fn save(&self) -> Result<()> {
        ensure_config_dir()?;
        let path = config_path()?;
        
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&path, toml_string)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}
```

### 5. Validação Explícita

```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validar limites
        if self.general.max_history_items == 0 {
            bail!("max_history_items must be > 0");
        }
        
        if self.general.poll_interval_ms < 50 {
            bail!("poll_interval_ms must be >= 50ms");
        }
        
        if self.general.max_text_size_mb > 100 {
            bail!("max_text_size_mb must be <= 100MB");
        }
        
        // Validar hotkey
        if self.hotkey.key.is_empty() {
            bail!("hotkey.key cannot be empty");
        }
        
        // Validar theme
        let valid_themes = ["dark", "light", "nord", "dracula", "gruvbox"];
        if !valid_themes.contains(&self.ui.theme.as_str()) {
            bail!("Invalid theme: {}", self.ui.theme);
        }
        
        Ok(())
    }
}
```

### 6. Defaults via Trait

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            hotkey: HotkeyConfig::default(),
            ui: UiConfig::default(),
            search: SearchConfig::default(),
            features: FeaturesConfig::default(),
            privacy: PrivacyConfig::default(),
            advanced: AdvancedConfig::default(),
            autocomplete: AutocompleteConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_history_items: default_max_history_items(),
            poll_interval_ms: default_poll_interval_ms(),
            max_text_size_mb: default_max_text_size_mb(),
            max_image_size_mb: default_max_image_size_mb(),
        }
    }
}
```

## 📝 Exemplo de TOML

```toml
# ~/.config/clippit/config.toml

[general]
max_history_items = 100
poll_interval_ms = 200
max_text_size_mb = 10
max_image_size_mb = 50

[hotkey]
modifier = "super"
key = "v"
# alt_modifier = "ctrl+shift"  # Opcional
# alt_key = "v"                 # Opcional

[ui]
theme = "dark"
language = "en"
font_family = "Sans"
font_size = 11
window_width = 700
window_height = 550

[ui.colors.dark]
background = "#1e1e1e"
foreground = "#d4d4d4"
selection = "#264f78"
border = "#3e3e3e"

[ui.colors.light]
background = "#ffffff"
foreground = "#000000"
selection = "#add6ff"
border = "#cccccc"

[search]
max_suggestions = 5
focus_on_show = true

[features]
capture_text = true
capture_images = true
enable_notifications = true

[privacy]
ignored_apps = ["keepassxc", "bitwarden", "1password"]
clear_on_exit = false
# retention_days = 30  # Opcional

[advanced]
log_level = "info"
# data_dir = "/custom/path"    # Opcional
# config_dir = "/custom/path"  # Opcional

[autocomplete]
enabled = false
max_suggestions = 3
min_chars = 2
delay_ms = 300
show_in_passwords = false
ignored_apps = ["gnome-terminal", "keepassxc"]
# hotkey_modifier = "ctrl+shift"  # Opcional
# hotkey_key = "a"                # Opcional

[autocomplete.ai]
enabled = false
provider = "local"
model = "gpt-4"
api_key = ""
```

## 🔄 Fluxo de Uso

### Inicialização (Daemon)

```rust
use clippit_core::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Carrega config (cria se não existir)
    let config = Config::load()?;
    
    // Usa config
    let poll_interval = Duration::from_millis(
        config.general.poll_interval_ms
    );
    
    let max_items = config.general.max_history_items;
    
    // ...
}
```

### Leitura (Dashboard)

```rust
// Ler config atual
let config = Config::load()?;
println!("Theme: {}", config.ui.theme);
```

### Escrita (Dashboard)

```rust
// Modificar e salvar
let mut config = Config::load()?;
config.ui.theme = "nord".to_string();
config.validate()?;  // Sempre validar antes de salvar
config.save()?;
```

## 🚫 Anti-Patterns

### ❌ Hardcoded Paths

```rust
// NÃO!
let path = "/home/user/.config/clippit/config.toml";

// ✅ Use dirs crate
let path = config_path()?;
```

### ❌ Unwrap no Load

```rust
// NÃO!
let config = Config::load().unwrap();

// ✅ Propague erro ou use default
let config = Config::load()
    .unwrap_or_else(|_| Config::default());
```

### ❌ Sem Defaults

```rust
// NÃO!
pub struct GeneralConfig {
    pub max_items: usize,  // E se não estiver no TOML?
}

// ✅ Sempre tenha default
pub struct GeneralConfig {
    #[serde(default = "default_max_items")]
    pub max_items: usize,
}
```

### ❌ Esquecer de Validar

```rust
// NÃO!
config.general.max_items = 0;  // Inválido!
config.save()?;

// ✅ Sempre valide
config.general.max_items = 0;
config.validate()?;  // Retorna erro
```

## 📝 Checklist ao Adicionar Nova Config

- [ ] Criar struct com `#[derive(Debug, Clone, Serialize, Deserialize)]`
- [ ] Adicionar função `default_*()` para cada campo
- [ ] Implementar `Default` trait para a struct
- [ ] Adicionar ao `Config` principal
- [ ] Atualizar `validate()` com novas regras
- [ ] Atualizar `clippit.example.toml` com exemplo
- [ ] Documentar em `docs/CONFIGURATION.md`
- [ ] Testar load/save completo
- [ ] Adicionar teste de validação

## 🧪 Exemplo de Teste

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert!(parsed.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_max_items() {
        let mut config = Config::default();
        config.general.max_history_items = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_with_missing_fields() {
        let toml = r#"
            [general]
            max_history_items = 50
            # poll_interval_ms ausente, deve usar default
        "#;
        
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.general.max_history_items, 50);
        assert_eq!(config.general.poll_interval_ms, 200);  // Default
    }
}
```

## 🔗 Integração com Outros Módulos

### Daemon
- **Lê**: Todas as configs (polling, hotkeys, features, autocomplete)
- **Escreve**: Nunca (somente leitura)

### Popup
- **Lê**: UI configs (tema, tamanho janela)
- **Escreve**: Nunca

### Dashboard
- **Lê**: Todas as configs (para exibir)
- **Escreve**: Todas as configs (editor de configurações)

### IBus Engine
- **Lê**: `autocomplete` config
- **Escreve**: Nunca

## 🔗 Links Relacionados

- **Core Overview**: [CORE-OVERVIEW.md](./CORE-OVERVIEW.md)
- **Tipos**: [TYPES-DEFINITIONS.md](./TYPES-DEFINITIONS.md)
- **Documentação Config**: [../../docs/CONFIGURATION.md](../../docs/CONFIGURATION.md)
- **Exemplo TOML**: [../../clippit.example.toml](../../clippit.example.toml)

---

**Versão**: 1.0  
**Data**: 2026-01-28
