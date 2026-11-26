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
        id: assistente
        corretor: corretor
    }

    Lancador {
        id: lancador
        corretor: corretor
        assistente: assistente
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
            lancador.toggleView("ia");
        }
        function historico(): void {
            lancador.toggleView("historico");
        }
        function acoes(): void {
            lancador.openTextActions(false);
        }
        function assistente(): void {
            assistente.safeOpen();
        }
        function modo_jogo(): void {
            assistente.toggleGameMode();
        }
        function estado(): string {
            return JSON.stringify({compositor: Compositor.kind, ime: Ime.available, aberto: lancador.open, tela: lancador.view,
                corretor: corretor.state_, selo: corretor.badgeShown, trabalhando: corretor.working, fila: corretor.queue.length,
                atalhos: Object.keys(Atalhos.handlers)});
        }
    }

    Component.onDestruction: Compositor.clearKeys()
}
