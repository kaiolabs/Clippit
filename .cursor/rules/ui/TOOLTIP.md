# clippit-tooltip - Tooltip Flutuante

## 📍 Localização
`crates/clippit-tooltip/src/main.rs`

## 🎯 Responsabilidade

Exibe tooltip flutuante temporário para sugestões de autocomplete.

## 🔧 Implementação

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    let text = args.get(1).unwrap_or(&"".to_string()).clone();
    
    gtk4::init().unwrap();
    
    let window = Window::builder()
        .decorated(false)
        .resizable(false)
        .default_width(280)
        .default_height(130)
        .build();
    
    let label = Label::new(Some(&text));
    window.set_child(Some(&label));
    
    // CSS styling
    let css = r#"
        window {
            background-color: rgba(32, 32, 32, 0.96);
            border-radius: 10px;
        }
        label {
            color: white;
            font-family: monospace;
            font-size: 13px;
        }
    "#;
    
    apply_css(&css);
    
    window.present();
    
    // Auto-close após 3s
    glib::timeout_add_seconds_local(3, move || {
        window.close();
        glib::ControlFlow::Break
    });
    
    gtk4::main();
}
```

## 📝 Uso

```bash
clippit-tooltip "Sugestão 1\nSugestão 2"
```

## 🎨 Estilo

- Background: rgba(32,32,32,0.96)
- Border radius: 10px
- Font: monospace 13px
- Auto-close: 3s

## 🔗 Links
- [UI Overview](./UI-OVERVIEW.md)
- [Autocomplete Feature](../features/AUTOCOMPLETE-GLOBAL.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
