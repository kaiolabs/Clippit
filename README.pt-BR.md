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

### Hyprland, Sway, niri (Quickshell)

Fora do GNOME o lançador, o corretor e o assistente rodam numa interface em [Quickshell](https://quickshell.org)
no lugar da extensão do GNOME, com as mesmas telas, teclas e configurações. O `install.sh` escolhe sozinho
quando o GNOME não está rodando; no Nix, use `programs.clippit.quickshell = true;` (e `gnomeExtension = false;`).

- O `clippit-shell` sobe a interface (o serviço de usuário `clippit-shell` faz isso ao entrar). Ele registra
  Alt+Espaço, Alt+C, Alt+I, Alt+A, Alt+K e Ctrl+Shift+E no Hyprland ou no Sway sozinho, de novo a cada recarga da
  configuração. No niri, ligue as teclas na configuração dele a `clippit-shell lancador`, `historico`, `ia`,
  `acoes`, `assistente` e `modo_jogo`.
- O `clippit-wl` lê e edita o campo de texto em foco (input-method-v2) e desenha o selo ao lado do cursor. Só um
  método de entrada roda por vez: não funciona junto com fcitx5 ou ibus.
- Precisa de `quickshell`, `wtype`, `grim`, `wl-clipboard` e `gtk-launch` (do GTK 3) (o pacote do Nix já traz).
- O compositor precisa exportar `WAYLAND_DISPLAY` (e, no Hyprland, `HYPRLAND_INSTANCE_SIGNATURE`) para o
  systemd, por exemplo com `systemctl --user import-environment WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE` ao iniciar.

Diferenças do GNOME, porque o compositor não expõe: o painel de sugestões abre no mouse (Hyprland) ou embaixo da
janela, e não logo acima do cursor; o assistente lê a janela inteira como ela aparece na tela (uma notificação ou outra janela por cima entra no print); rolar fora do lançador não o solta (clicar
fora solta, no Hyprland); sem desfoque atrás dos painéis. No Sway 1.12 não há selo ao lado do cursor (o Sway cai
quando ele fecha); o Alt+O continua abrindo as sugestões. Não perturbe precisa de swaync, mako ou dunst; brilho
precisa do brightnessctl. `clippit-shell estado` mostra o que ele está vendo (compositor, método de entrada,
atalhos, corretor).

### Nix e NixOS

O repositório é um flake. Tem o pacote, um módulo do Home Manager (o recomendado: ele também liga os
serviços, a extensão do GNOME, os atalhos e as suas configurações), um módulo do NixOS e um ambiente de
desenvolvimento.

**Testar sem instalar:**

```bash
nix run github:kaiolabs/Clippit            # abre as Configurações
nix shell github:kaiolabs/Clippit          # todos os comandos clippit-* neste terminal
```

Sem flakes ligados, acrescente `--extra-experimental-features 'nix-command flakes'`.

**Home Manager.** Acrescente a entrada no seu `flake.nix`:

```nix
inputs.clippit = {
  url = "github:kaiolabs/Clippit";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Depois, na configuração do Home Manager (com `inputs` passado em `extraSpecialArgs`):

```nix
{ inputs, pkgs, ... }:
{
  imports = [ inputs.clippit.homeManagerModules.default ];

  programs.clippit = {
    enable = true;

    # ~/.config/clippit/corretor.conf: IA, modelos, corretor, atalhos
    settings = {
      device = "auto";
      nomes = [ "Maria Silva" "Maria" ];
      atalho_ia = "<Super>i";
    };

    # ~/.config/clippit/config.toml: histórico, privacidade, OCR, idioma
    historySettings = {
      general.max_history_items = 500;
      ui.language = "pt";
    };
  };
}
```

Todas as chaves de `settings` e `historySettings` estão em [docs/configuration.md](docs/configuration.md).
Só as chaves que você definiu são escritas: as Configurações continuam funcionando, e essas chaves
voltam ao valor do Nix no próximo `home-manager switch`.

Outras opções:

| Opção | Padrão | O que faz |
|---|---|---|
| `package` | o pacote do flake | `.override { llama-cpp = pkgs.llama-cpp.override { vulkanSupport = true; }; }` para a IA na placa de vídeo, ou `.override { withAI = false; }` para deixar a IA de fora. |
| `gnomeExtension` | `true` | Liga a extensão do GNOME (entra na lista; as suas outras extensões ficam). |
| `historyShortcut` | `"<Alt>c"` | Atalho do GNOME para o histórico; `null` não mexe nos atalhos do GNOME. |
| `freeAltSpace` | `true` | Libera Alt+Espaço (menu da janela do GNOME) para o lançador. |
| `popupAutostart` | `true` | Abre o popup do histórico escondido ao entrar na sessão. |
| `doctor` | `true` | Roda o doutor a cada 2 minutos. |

**Só NixOS** (sem Home Manager): instala o pacote para todos os usuários e liga os serviços. As
configurações ficam nas Configurações do Clippit.

```nix
{ inputs, ... }:
{
  imports = [ inputs.clippit.nixosModules.default ];
  programs.clippit.enable = true;
}
```

Ou só o pacote, pelo overlay: `nixpkgs.overlays = [ inputs.clippit.overlays.default ];` e
`environment.systemPackages = [ pkgs.clippit ];` (aí os serviços você liga:
`systemctl --user enable --now clippit.service`).

**Desenvolvimento:** `nix develop` traz o Rust, as bibliotecas do GTK/Tesseract e o clippy, e o
`cargo build` funciona como no Ubuntu.

Observações no Nix:

- O motor da IA é o `llama-cpp` do nixpkgs (no processador, por padrão). Não use "instalar llama.cpp" nas
  Configurações: ele baixa uma versão do Ubuntu que não roda no NixOS. Os modelos baixam normalmente.
- A extensão do GNOME declara o GNOME Shell 46 no `metadata.json`; em versões mais novas o GNOME pode
  recusar carregá-la.
- O ditado por voz precisa do `faster-whisper`, que não vem no pacote.

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
