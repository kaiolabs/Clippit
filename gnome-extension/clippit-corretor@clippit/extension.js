// Clippit Corretor — carregador.
//
// A lógica fica em core.js. Este arquivo só o carrega e, sempre que core.js
// muda no disco (checado a cada segundo), recarrega na hora, sem sair da sessão. O GNOME guarda os
// módulos em cache pelo caminho, então cada versão é copiada para um arquivo
// com nome novo em $XDG_RUNTIME_DIR antes do import.

import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const TAG = '[clippit-corretor]';

export default class ClippitCorretorExtension extends Extension {
    enable() {
        this._generation = 0;
        this._instance = null;
        this._pollId = 0;

        this._coreFile = this.dir.get_child('core.js');
        this._runDir = Gio.File.new_for_path(
            GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor']));
        try {
            this._runDir.make_directory_with_parents(null);
        } catch (e) {
            if (!e.matches(Gio.IOErrorEnum, Gio.IOErrorEnum.EXISTS))
                console.error(`${TAG} ${e.message}`);
        }

        // Verifica a data de modificação a cada segundo. Monitor de arquivo
        // (inotify) não é confiável no disco NTFS/FUSE onde fica o projeto.
        this._stamp = this._readStamp();
        this._pollId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 1, () => {
            const stamp = this._readStamp();
            if (stamp && stamp !== this._stamp) {
                this._stamp = stamp;
                this._load();
            }
            return GLib.SOURCE_CONTINUE;
        });

        this._load();
    }

    disable() {
        if (this._pollId) {
            GLib.source_remove(this._pollId);
            this._pollId = 0;
        }
        this._stopInstance();
    }

    _readStamp() {
        try {
            const info = this._coreFile.query_info('time::modified,time::modified-usec,standard::size',
                Gio.FileQueryInfoFlags.NONE, null);
            return `${info.get_attribute_uint64('time::modified')}.${info.get_attribute_uint32('time::modified-usec')}:${info.get_size()}`;
        } catch (_e) {
            return null;
        }
    }

    _stopInstance() {
        try {
            this._instance?.disable();
        } catch (e) {
            console.error(`${TAG} erro ao desligar: ${e.message}`);
        }
        this._instance = null;
    }

    async _load() {
        const gen = ++this._generation;
        const copy = this._runDir.get_child(`core-${GLib.get_real_time()}.js`);
        try {
            this._coreFile.copy(copy, Gio.FileCopyFlags.OVERWRITE, null, null);
            const mod = await import(copy.get_uri());
            if (gen !== this._generation || !this._pollId)
                return;
            this._stopInstance();
            this._instance = new mod.Corretor();
            this._instance.enable();
            console.log(`${TAG} core carregado (versão ${gen})`);
        } catch (e) {
            console.error(`${TAG} falha ao carregar core.js: ${e.message}\n${e.stack}`);
        } finally {
            try {
                copy.delete(null);
            } catch (_e) {}
        }
    }
}
