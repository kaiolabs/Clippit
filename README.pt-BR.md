# Clippit

**Gerenciador de área de transferência, lançador e assistente com IA para o GNOME, tudo rodando no seu
computador.** Histórico do que você copia, um lançador no estilo Raycast, corretor de texto em qualquer
campo, assistente de resposta e um chat com IA que mexe de verdade no sistema. Nada vai para a nuvem,
a não ser quando você pede uma pesquisa na web.

🇺🇸 [Read in English](README.md)

## O que ele faz

| | Atalho padrão |
|---|---|
| **Lançador** — apps, arquivos, sites, contas e conversões, controles do sistema, janelas, emoji, snippets | `Alt+Espaço` |
| **Histórico** — textos, links, código e imagens; o texto dos prints é lido (OCR) e dá para procurar por ele; fixar, transformar, colar | `Alt+C` |
| **Chat com a IA** — modelo local com 38 ferramentas: pesquisar na web, abrir apps, mexer em janelas, volume e brilho, ler e organizar arquivos, converter PDF e imagens, git, tarefas no fundo, notas, lembretes… | `Alt+I` |
| **Ações no texto** — resumir, traduzir, explicar ou reescrever o texto selecionado | `Alt+A` |
| **Corretor** — um selo ao lado de qualquer campo de texto; uma tecla corrige tudo | `Alt+O` / `Alt+S` |
| **Assistente de resposta** — lê a conversa da janela em foco (WhatsApp, Slack…) e sugere respostas | `Alt+K` |
| **Modo jogo** — tira a IA da placa de vídeo; jogos são detectados sozinhos | `Ctrl+Shift+E` |
| **Doutor** — confere todas as partes do Clippit a cada 2 minutos e corrige o que quebrar | — |

Todos os atalhos podem ser trocados em **Configurações → Atalhos**, e os modelos da IA em
**Configurações → Corretor**. A tela **Funcionalidades** explica cada recurso.

## Requisitos

- **GNOME Shell 46 no Wayland** (testado no Zorin OS 18, base Ubuntu 24.04). Outras versões podem
  precisar de uma linha no `metadata.json`.
- Pacotes do Debian/Ubuntu (o instalador diz quais faltam).
- [Rust](https://rustup.rs) para compilar.
- Para a IA: uns 4 GB de disco. Placa de vídeo com Vulkan (NVIDIA, AMD ou Intel) deixa rápido; também
  roda no processador com o modelo pequeno.

## Instalar

```bash
git clone https://github.com/kaiolabs/Clippit.git
cd Clippit
./scripts/install.sh --deps
```

O instalador compila, instala só para o seu usuário (`~/.local`), liga os serviços, configura os
atalhos e oferece baixar o motor da IA ([llama.cpp](https://github.com/ggml-org/llama.cpp)) e os
modelos padrão ([Qwen 3.5](https://huggingface.co/unsloth)). Na primeira instalação, saia e entre de
novo na sessão para o GNOME carregar a extensão.

Para remover: `./scripts/uninstall.sh` (seus dados só são apagados se você confirmar).

## Privacidade

- Histórico, IA, corretor, ditado e leitura de imagens rodam localmente.
- A internet só é usada quando você pede (pesquisa, notícias, cotação, clima) e para baixar modelos.
- A IA nunca lê arquivos que pareçam chaves ou senhas, e tudo que muda o seu sistema (mover arquivos,
  fechar um programa, rodar um comando) espera você apertar **Rodar**.

## Documentação

- [Como contribuir](CONTRIBUTING.md) — organização do projeto, como rodar em desenvolvimento, os
  scripts e as regras de código, commits e pull requests (em inglês).
- [Configuração](docs/configuration.md) — todas as opções do `config.toml` e do `corretor.conf`.
- [Histórico de versões](CHANGELOG.md)

## Licença

[MIT](LICENSE)
