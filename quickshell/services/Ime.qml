pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// clippit-wl: the focused text field (input-method-v2), the badge next to the
// cursor and the com.clippit.Lancador D-Bus service.
Singleton {
    id: ime

    property bool available: false
    property bool active: false
    property string text: ""
    property int cursor: 0
    property int purpose: 0
    property int hints: 0
    // Bumped on every focus change (a new field, or the same one again).
    property int focusId: 0

    signal changed
    signal badgeClicked(string part)
    signal call(int id, string method, var args)

    // Password, PIN and private fields are never read.
    // text-input-v3: purpose 8 password, 9 PIN; hints 0x40 hidden text, 0x80 sensitive data.
    readonly property bool sensitive: purpose === 8 || purpose === 9 || (hints & 0xc0) !== 0

    function send(obj) {
        if (proc.running)
            proc.write(JSON.stringify(obj) + "\n");
    }

    // Deletes `before`/`after` characters around the cursor and writes `text`.
    function edit(before, after, text) {
        send({cmd: "edit", before, after, text: text || ""});
    }

    function commit(text) {
        edit(0, 0, text);
    }

    function badge(look) {
        send(Object.assign({cmd: "badge"}, look));
    }

    function reply(id, value) {
        send({cmd: "reply", id, value: String(value ?? "")});
    }

    Process {
        id: proc
        command: Sys.tool("clippit-wl")
        running: true
        stdinEnabled: true
        stdout: SplitParser {
            onRead: line => {
                let ev;
                try {
                    ev = JSON.parse(line);
                } catch (e) {
                    return;
                }
                if (ev.ev === "caps") {
                    ime.available = ev.im;
                } else if (ev.ev === "field") {
                    if (ev.active && !ime.active)
                        ime.focusId++;
                    ime.active = ev.active;
                    ime.text = ev.text;
                    ime.cursor = ev.cursor;
                    ime.purpose = ev.purpose;
                    ime.hints = ev.hints;
                    if (Quickshell.env("CLIPPIT_DEBUG"))
                        console.log("clippit: " + (Date.now() % 100000) + " campo ativo=" + ev.active + " " + Array.from(ev.text).length + " letras, cursor " + ev.cursor + ", tipo " + ev.purpose);
                    ime.changed();
                } else if (ev.ev === "badge") {
                    ime.badgeClicked(ev.part);
                } else if (ev.ev === "call") {
                    ime.call(ev.id, ev.method, ev.args);
                }
            }
        }
        onExited: restart.start()
    }

    Timer {
        id: restart
        interval: 2000
        onTriggered: proc.running = true
    }
}
