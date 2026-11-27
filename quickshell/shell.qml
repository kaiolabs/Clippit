//@ pragma UseQApplication
pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Io
import "services"
import "corretor"
import "assistente"
import "lancador"

// Clippit's interface for Hyprland, Sway, niri and other wlroots-style
// compositors: the same launcher, spell checker and assistant as the GNOME
// extension. Run with `clippit-shell` (the clippit-shell service does it).
ShellRoot {
    id: shell

    Corretor {
        id: corretor
    }

    Assistente {
        id: assistenteRoot
        corretor: corretor
    }

    Lancador {
        id: lancadorRoot
        corretor: corretor
        assistente: assistenteRoot
    }

    // clippit-shell <function> → here. The compositor's binds call `key`.
    IpcHandler {
        target: "clippit"

        function key(accel: string): void {
            Atalhos.fire(accel);
        }
        function lancador(): void {
            Atalhos.fire(Conf.keyOf("lancador"));
        }
        function ia(): void {
            lancadorRoot.toggleView("ia");
        }
        function historico(): void {
            lancadorRoot.toggleView("historico");
        }
        function acoes(): void {
            lancadorRoot.openTextActions(false);
        }
        function assistente(): void {
            assistenteRoot.safeOpen();
        }
        function modo_jogo(): void {
            assistenteRoot.toggleGameMode();
        }
        function estado(): string {
            return JSON.stringify({compositor: Compositor.kind, ime: Ime.available, aberto: lancadorRoot.open, tela: lancadorRoot.view,
                corretor: corretor.state_, selo: corretor.badgeShown, trabalhando: corretor.working, fila: corretor.queue.length,
                atalhos: Object.keys(Atalhos.handlers)});
        }
    }

    Component.onDestruction: Compositor.clearKeys()
}
