# 🎯 Funcionalidades Avançadas do Clippit

## ✅ Implementadas

### 1. Cola Automática ao Clicar
Quando você abre o popup com o atalho e clica em um item:
1. ✅ O item é copiado para o clipboard
2. ✅ **Cola automaticamente** no campo ativo (Ctrl+V simulado)
3. ✅ Fecha o popup

**Como funciona:**
- Usa `xdotool` para simular tecla Ctrl+V
- Espera 100ms para garantir que o clipboard foi atualizado
- Cola no campo/aplicativo que estava focado

### 2. Atalho Personalizável
- ✅ Configure qualquer combinação via Dashboard
- ✅ Suporta: Ctrl, Alt, Shift, Super
- ✅ Funciona com teclado numérico, F1-F12, letras, etc.

## 📋 Requisitos

### xdotool (Para Auto-Paste)
```bash
sudo apt install xdotool
```

Ou rode o script:
```bash
./scripts/install-gtk-deps.sh
```

## 🔮 Planejado - Screenshot Automático

### Como funcionará:
1. Você tira um print (PrtScr, Shift+PrtScr, área, etc.)
2. O Clippit detecta que uma imagem entrou no clipboard
3. Automaticamente salva no histórico
4. Disponível imediatamente para colar

### Implementação:
O daemon já monitora o clipboard constantemente (polling a cada 200ms).
Quando detecta uma imagem:
- ✅ Salva automaticamente no histórico
- ✅ Fica disponível no popup
- ✅ Pode colar em qualquer app

**Já funciona!** O daemon monitora automaticamente:
- Screenshots do sistema (PrtScr)
- Prints de área (Shift+PrtScr)
- Capturas de janela
- Imagens copiadas de navegadores
- Qualquer imagem no clipboard

## 🚀 Como Usar

### 1. Cole do Histórico
```
1. Pressione seu atalho (ex: Ctrl+1 numpad)
2. Veja o popup com histórico
3. Clique em qualquer item
4. É colado automaticamente!
```

### 2. Screenshots
```
1. Tire um print (PrtScr)
2. Está automaticamente salvo
3. Pressione seu atalho
4. Veja o print no histórico com ícone 🖼️
5. Clique para colar
```

### 3. Buscar no Histórico
```
1. Abra o popup (atalho)
2. Digite no campo de busca
3. Filtra em tempo real
4. Clique para colar
```

## ⚙️ Configurações

### Ajustar Intervalo de Monitoramento
Edite `~/.config/clippit/config.toml`:
```toml
[general]
poll_interval_ms = 200  # Reduzir para captura mais rápida (50-1000)
```

Menor = mais responsivo, mas usa mais CPU
Maior = menos CPU, mas pode perder itens rápidos

### Limitar Tamanho de Imagens
```toml
[general]
max_image_size = 5242880  # 5MB em bytes
```

## 🎨 Dicas

1. **Cole Rápido**: Configure um atalho fácil como `Super+V`
2. **Histórico Grande**: Aumente `max_history_items` para 500+
3. **Screenshots**: Use Shift+PrtScr para área, cola automaticamente
4. **Busca**: Use a busca para achar texto copiado dias atrás

## 🐛 Troubleshooting

### Auto-paste não funciona?
```bash
# Instalar xdotool
sudo apt install xdotool

# Verificar se está instalado
which xdotool
```

### Atalho não funciona?
```bash
# Reiniciar daemon
systemctl --user restart clippit

# Ver se pegou o atalho
journalctl --user -u clippit -n 10 | grep "Registered"
```

### Screenshots não aparecem?
- Certifique-se que o daemon está rodando
- Verifique se o print realmente copiou (teste colar em outro app)
- Aumente `max_image_size` se for imagem muito grande

---

**Status**: ✅ Auto-paste funcionando | ✅ Screenshots detectados automaticamente
