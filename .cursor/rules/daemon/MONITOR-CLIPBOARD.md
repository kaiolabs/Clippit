# clippit-daemon - Monitor de Clipboard

## 📍 Localização
`crates/clippit-daemon/src/monitor.rs`

## 🎯 Responsabilidade

Monitora o clipboard do sistema (Wayland/X11) e adiciona novos itens ao histórico automaticamente.

## 🔄 Fluxo

```rust
pub async fn start_monitor(history: Arc<Mutex<HistoryManager>>) {
    let mut clipboard = Clipboard::new().unwrap();
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<String> = None;
    
    loop {
        let config = Config::load().unwrap_or_default();
        
        // Monitor texto
        if config.features.capture_text {
            if let Ok(text) = clipboard.get_text() {
                if Some(&text) != last_text.as_ref() {
                    process_text(&text, &history).await;
                    last_text = Some(text);
                }
            }
        }
        
        // Monitor imagem
        if config.features.capture_images {
            if let Ok(img) = clipboard.get_image() {
                let hash = compute_image_hash(&img);
                if Some(&hash) != last_image_hash.as_ref() {
                    process_image(img, &history).await;
                    last_image_hash = Some(hash);
                }
            }
        }
        
        tokio::time::sleep(Duration::from_millis(
            config.general.poll_interval_ms
        )).await;
    }
}
```

## 📦 Processamento

### Texto
1. Valida com `ContentValidator`
2. Calcula SHA256
3. Verifica duplicatas
4. Adiciona ao `HistoryManager`

### Imagem
1. Converte `ImageData` → PNG bytes
2. Valida tamanho e formato
3. Otimiza (resize se > 2048px)
4. Cria thumbnail 128x128
5. Salva em `~/.local/share/clippit/images/{hash}.png`
6. Adiciona ao histórico com path

## 🔗 Links
- [Daemon Overview](./DAEMON-OVERVIEW.md)
- [Image Handling](../features/IMAGE-HANDLING.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
