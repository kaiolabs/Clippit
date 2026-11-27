pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Everything that talks to the outside: programs, files, clipboard,
// notifications. Each call returns a Promise.
Singleton {
    id: sys

    readonly property string home: Quickshell.env("HOME")
    readonly property string runtimeDir: Quickshell.env("XDG_RUNTIME_DIR") || "/tmp"
    readonly property string dataDir: (Quickshell.env("XDG_DATA_HOME") || home + "/.local/share") + "/clippit"
    readonly property string configDir: (Quickshell.env("XDG_CONFIG_HOME") || home + "/.config") + "/clippit"
    readonly property string cacheDir: (Quickshell.env("XDG_CACHE_HOME") || home + "/.cache") + "/clippit"
    readonly property string workDir: runtimeDir + "/clippit-corretor"

    // ~/.local/bin (install.sh) first, then the PATH (Nix package).
    function tool(name) {
        return ["sh", "-c", "p=\"$HOME/.local/bin/$0\"; [ -x \"$p\" ] && exec \"$p\" \"$@\"; exec \"$0\" \"$@\"", name];
    }

    Component {
        id: jobComponent
        Process {
            id: proc
            property var resolve
            property var reject
            property string input: ""
            property bool hasInput: false
            property int timeoutS: 120
            property bool done: false
            property int code: -1
            property bool outDone: false
            property bool errDone: false
            function finish() {
                if (proc.done || proc.code < 0 || !proc.outDone || !proc.errDone)
                    return;
                proc.done = true;
                timer.stop();
                if (proc.code === 0)
                    proc.resolve(out.text);
                else
                    proc.reject(new Error((err.text || "").trim().slice(0, 200) || "saiu com código " + proc.code));
                proc.destroy();
            }
            stdinEnabled: hasInput
            stdout: StdioCollector {
                id: out
                onStreamFinished: {
                    proc.outDone = true;
                    proc.finish();
                }
            }
            stderr: StdioCollector {
                id: err
                onStreamFinished: {
                    proc.errDone = true;
                    proc.finish();
                }
            }
            onStarted: {
                if (proc.hasInput) {
                    proc.write(proc.input);
                    proc.stdinEnabled = false; // closes stdin
                }
            }
            onExited: (exitCode, _status) => {
                proc.code = exitCode;
                proc.finish();
            }
            property Timer timer: Timer {
                id: timer
                interval: proc.timeoutS * 1000
                running: true
                onTriggered: {
                    proc.signal(9);
                    if (!proc.done) {
                        proc.done = true;
                        proc.reject(new Error("demorou demais"));
                    }
                    proc.destroy();
                }
            }
        }
    }

    // run(argv, input?, timeoutS?) → Promise<stdout>
    function run(argv, input, timeoutS) {
        return new Promise((resolve, reject) => {
            const hasInput = input !== undefined && input !== null;
            const p = jobComponent.createObject(sys, {
                command: argv, resolve, reject, input: hasInput ? String(input) : "", hasInput,
                timeoutS: timeoutS || 120,
            });
            if (!p) {
                reject(new Error("não consegui rodar " + argv[0]));
                return;
            }
            p.running = true;
        });
    }

    function sh(script) {
        return run(["sh", "-c", script]);
    }

    function detach(argv) {
        Quickshell.execDetached(argv);
    }

    // Apps get their own systemd scope, so restarting clippit-shell.service does not kill them.
    // systemd-run moves itself into the scope before it execs the app, so nothing stays in the
    // service cgroup. If the user manager refuses scopes, launch without one rather than not at all.
    property bool scopes: true

    function launch(argv) {
        if (scopes)
            detach(["systemd-run", "--user", "--scope", "--quiet", "--collect", "--slice=app.slice", "--"].concat(argv));
        else
            detach(argv);
    }

    Component.onCompleted: run(["systemd-run", "--user", "--scope", "--quiet", "--collect", "--", "true"], null, 5)
        .catch(e => {
            console.warn("clippit: systemd scopes unavailable, apps start in the shell's cgroup:", e.message);
            sys.scopes = false;
        })

    function launchApp(entry) {
        launch(["gtk-launch", entry.id.replace(/\.desktop$/, "")]);
    }

    function quote(s) {
        return "'" + String(s).replace(/'/g, "'\\''") + "'";
    }

    function notify(title, body, urgency) {
        const argv = ["notify-send", "-a", "Clippit", "-i", urgency === "critical" ? "alarm-symbolic" : "dialog-information"];
        if (urgency)
            argv.push("-u", urgency);
        argv.push(title, body || "");
        detach(argv);
    }

    // Clipboard (wl-clipboard).
    function copy(text) {
        return run(["wl-copy", "--", String(text)], null, 5).catch(() => {});
    }

    function clipboard(primary) {
        return run(primary ? ["wl-paste", "--primary", "--no-newline"] : ["wl-paste", "--no-newline"], null, 3)
            .catch(() => "");
    }

    function openUri(uri, appId) {
        if (appId)
            launch(["gtk-launch", appId.replace(/\.desktop$/, ""), uri]);
        else
            launch(["xdg-open", uri]);
    }

    // Files.
    function read(path) {
        return run(["cat", "--", path], null, 5);
    }

    function readJson(path, fallback) {
        return read(path).then(t => JSON.parse(t)).catch(() => fallback);
    }

    function write(path, text) {
        return run(["sh", "-c", "mkdir -p \"$(dirname \"$0\")\" && cat > \"$0.tmp\" && mv -f \"$0.tmp\" \"$0\"", path], text, 5);
    }

    function writeJson(path, value) {
        return write(path, JSON.stringify(value));
    }

    function remove(path) {
        return run(["rm", "-f", "--", path], null, 5).catch(() => {});
    }

    function exists(path) {
        return run(["test", "-e", path], null, 3).then(() => true, () => false);
    }

    function mkdir(path) {
        return run(["mkdir", "-p", path], null, 3).catch(() => {});
    }

    function has(command) {
        return run(["sh", "-c", "command -v \"$0\" >/dev/null", command], null, 3).then(() => true, () => false);
    }

    function sha256(text) {
        return run(["sh", "-c", "sha256sum | cut -d' ' -f1"], text, 5).then(t => t.trim());
    }
}
