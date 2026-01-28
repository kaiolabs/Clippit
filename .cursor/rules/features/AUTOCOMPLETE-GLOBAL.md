# Autocomplete Global Feature

## 🎯 Responsabilidade
Sugestões inteligentes enquanto usuário digita em qualquer aplicativo.

## 🔄 Fluxo
1. **IBus captura** keystroke
2. **TypingBuffer** acumula caracteres
3. **Extrai palavra** atual (≥2 chars)
4. **Busca sugestões** no histórico
5. **Score e ordena** por relevância
6. **Exibe popup** flutuante (yad/tooltip)
7. **Tab injeta** texto completo (xdotool)

## 📦 Componentes
- `clippit-ibus/` - Engine IBus
- `daemon/typing_monitor.rs` - Processamento
- `daemon/autocomplete_manager.rs` - Gerenciamento

## ⚙️ Configuração
```toml
[autocomplete]
enabled = false
max_suggestions = 3
min_chars = 2
delay_ms = 300
ignored_apps = ["gnome-terminal", "keepassxc"]
```

## 🔗 Links
- [IBus Engine](../infrastructure/IBUS-ENGINE.md)
- [Typing Monitor](../daemon/TYPING-AUTOCOMPLETE.md)
- [AUTOCOMPLETE_IMPLEMENTATION.md](../../AUTOCOMPLETE_IMPLEMENTATION.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
