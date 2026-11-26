pragma Singleton

import QtQuick
import Quickshell
import "../lib/compat.js" as Compat

// Typing into the app in front: Ctrl+V / Ctrl+A through a virtual keyboard
// (wtype), giving the clipboard back afterwards.
Singleton {
    function keys(combo) {
        // combo: "ctrl+v"
        const parts = combo.split("+");
        const key = parts.pop();
        const argv = ["wtype"];
        parts.forEach(m => argv.push("-M", m));
        argv.push(key);
        parts.slice().reverse().forEach(m => argv.push("-m", m));
        return Sys.run(argv, null, 5).catch(e => console.log("clippit: wtype: " + e.message));
    }

    // Pastes `text` where the cursor is, like Ctrl+V (and restores what was copied).
    function viaClipboard(text, restore) {
        return Compat.co(function* () {
            const old = restore ? yield Sys.clipboard(false) : "";
            yield Sys.copy(text);
            yield Compat.sleep(60);
            yield keys("ctrl+v");
            if (old) {
                yield Compat.sleep(900);
                yield Sys.copy(old);
            }
        });
    }

    // Selects everything in the field and pastes the new text.
    function replaceAll(text) {
        return Compat.co(function* () {
            const old = yield Sys.clipboard(false);
            yield Sys.copy(text);
            yield Compat.sleep(30);
            yield keys("ctrl+a");
            yield Compat.sleep(40);
            yield keys("ctrl+v");
            yield Compat.sleep(500);
            if (old)
                yield Sys.copy(old);
        });
    }

    // Back to the window behind the launcher, then paste.
    function into(win, text) {
        return Compat.co(function* () {
            if (win)
                yield Compositor.activate(win);
            yield Compat.sleep(250);
            yield viaClipboard(text, false);
        });
    }
}
