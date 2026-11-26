import QtQuick
import Quickshell
import "../services"
import "../lib/texto.js" as Texto
import "../lib/compat.js" as Compat

// Spell checker of the focused field (same flow as the GNOME extension):
// you type → pause → the chunks are checked by clippit-correct → the badge
// next to the cursor shows how many suggestions → the panel applies them.
Scope {
    id: c

    readonly property int checkDelayMs: 500
    readonly property int minChars: 8
    readonly property int cacheSize: 30
    readonly property int chunkCacheSize: 500
    readonly property int maxPasteChars: 4000
    // Apps where Ctrl+S is "save": there the shortcut only works with the panel open.
    readonly property var saveApps: /^(cursor|code|code-oss|vscodium|codium|gnome-text-editor|org\.gnome\.TextEditor|gedit|org\.gnome\.gedit|sublime_text|jetbrains-.*|libreoffice.*|soffice|gimp.*|inkscape|org\.inkscape\.Inkscape|blender|kdenlive|org\.kde\.kdenlive|obs|com\.obsproject\.Studio|resolve|zed|dev\.zed\.Zed)$/i

    property bool suspended: false
    property bool writing: false
    property bool working: false
    property bool openWhenDone: false
    property bool expanding: false
    property string state_: "idle"
    property var result: null // {text, corrected, sugs}
    property var ignored: ({})
    property bool badgeShown: false
    property bool panelOpen: false
    property string app: ""
    property var panelPos: null
    property var cache: new Map()
    property var chunkCache: new Map()
    property var queue: []
    property string inFlight: ""
    property double lastWarm: 0
    property int warmFocus: -1
    // Called by the assistant to hide the panel when it opens.
    signal dismissed

    function field() {
        return {text: Ime.text, cursor: Ime.cursor};
    }

    function usable(f) {
        const t = (f.text || "").trim();
        return Conf.on("ativo") && Ime.active && !Ime.sensitive && t.length >= minChars && /\s/.test(t);
    }

    function pending() {
        return (result ? result.sugs : []).filter(s => !ignored[s.key]);
    }

    function setSuspended(on) {
        suspended = on;
        reset();
        updateKeys();
    }

    function reset() {
        if (writing)
            return;
        checkTimer.stop();
        closePanel();
        showBadge(false);
        result = null;
    }

    Connections {
        target: Ime
        function onChanged() {
            c.onSurroundingChanged();
        }
        function onFocusIdChanged() {
            c.reset();
            Compositor.focused().then(w => c.app = w ? w.app : "");
        }
        function onActiveChanged() {
            if (!Ime.active && !c.writing)
                c.reset();
        }
        function onBadgeClicked(part) {
            if (part === "fix")
                c.apply(c.pending());
            else
                c.onBadgeClicked();
        }
    }

    Connections {
        target: Conf
        function onValuesChanged() {
            c.updateKeys();
        }
    }

    // ";gatilho" + space in any field becomes the snippet's text.
    function expandSnippet() {
        if (expanding || Ime.sensitive || !Snippets.items.length)
            return false;
        const text = Ime.text, cursor = Ime.cursor;
        if (!text || cursor < 2)
            return false;
        const before = Compat.chars(text).slice(0, cursor).join("");
        const m = before.match(/(?:^|\s)(;[^\s;]+) $/);
        const sn = m && Snippets.items.find(x => x.gatilho === m[1]);
        if (!sn)
            return false;
        expanding = true;
        Compat.co(function* () {
            try {
                const out = yield Snippets.expand(sn.texto);
                const n = Compat.chars(m[1]).length + 1;
                if (out.indexOf("\n") >= 0) {
                    Ime.edit(n, 0, "");
                    yield Paste.viaClipboard(out, true);
                } else {
                    Ime.edit(n, 0, out + " ");
                }
            } catch (e) {
                console.log("clippit: snippet falhou: " + e.message);
            }
            yield Compat.sleep(300);
            c.expanding = false;
        });
        return true;
    }

    function onSurroundingChanged() {
        if (suspended || writing)
            return;
        if (expandSnippet())
            return;
        const f = field();
        if (panelOpen) {
            if (result && Texto.flat(f.text) === Texto.flat(result.text))
                return;
            closePanel();
        }
        // Entered a text field: start loading the model now (it unloads by itself).
        if (Ime.active && Ime.focusId !== warmFocus && Conf.on("ativo") && !Ime.sensitive) {
            warmFocus = Ime.focusId;
            if (Date.now() - lastWarm > 20000) {
                lastWarm = Date.now();
                learn(["--warm"]);
            }
        }
        if (!usable(f)) {
            showBadge(false);
            return;
        }
        const whole = cache.get(Texto.flat(f.text));
        if (whole !== undefined) {
            setResult(f.text, whole, false);
            showBadge(true);
            return;
        }
        refreshFromChunks(f);
        showBadge(true);
        if (!Conf.on("verificar_sozinho"))
            return;
        const sp = split(f);
        for (const line of sp.lines) {
            for (const ch of line.chunks) {
                if (ch !== sp.current)
                    enqueue(ch);
            }
        }
        pendingCurrent = sp.current;
        checkTimer.restart();
    }

    property var pendingCurrent: null
    Timer {
        id: checkTimer
        interval: c.checkDelayMs
        onTriggered: {
            if (c.pendingCurrent)
                c.enqueue(c.pendingCurrent);
        }
    }

    function split(f) {
        const lines = Texto.chunksOf(f.text);
        const cursor = f.cursor || 0;
        let current = null;
        for (const line of lines) {
            for (const ch of line.chunks) {
                if (cursor >= ch.s && cursor <= ch.e + 1)
                    current = ch;
            }
        }
        return {lines, current};
    }

    function refreshFromChunks(f) {
        const lines = split(f).lines;
        const corrected = Texto.assemble(lines, chunkCache);
        const missing = lines.some(l => l.chunks.some(ch => !chunkCache.has(ch.key)));
        setResult(f.text, corrected, missing);
    }

    function enqueue(ch) {
        if (chunkCache.has(ch.key) || queue.some(q => q.key === ch.key) || inFlight === ch.key)
            return;
        queue = queue.concat([{key: ch.key, ctx: ch.ctx, text: ch.text}]);
        work();
    }

    function work() {
        if (working)
            return;
        working = true;
        paintBadge();
        Compat.co(function* () {
            try {
                while (c.queue.length) {
                    const chunk = c.queue[0];
                    c.queue = c.queue.slice(1);
                    const f = c.field();
                    if (!f.text || f.text.indexOf(chunk.text) < 0 || c.chunkCache.has(chunk.key))
                        continue;
                    if (c.state_ !== "issues")
                        c.setState("checking");
                    c.inFlight = chunk.key;
                    let out;
                    try {
                        const raw = yield c.runCorrector(chunk.ctx ? chunk.ctx + " " + chunk.text : chunk.text);
                        out = Texto.takeChunkPart(chunk.ctx, chunk.text, raw);
                    } catch (e) {
                        console.log("clippit: corretor falhou: " + e.message);
                        c.inFlight = "";
                        c.setState("error");
                        c.queue = [];
                        break;
                    }
                    c.inFlight = "";
                    c.chunkCache.set(chunk.key, out);
                    while (c.chunkCache.size > c.chunkCacheSize)
                        c.chunkCache.delete(c.chunkCache.keys().next().value);
                    const now = c.field();
                    if (c.writing || c.panelOpen || !c.usable(now))
                        continue;
                    c.refreshFromChunks(now);
                }
            } finally {
                c.working = false;
                c.paintBadge();
            }
            if (c.openWhenDone && !c.panelOpen) {
                c.openWhenDone = false;
                if (c.pending().length)
                    c.openPanel(null);
            }
        });
    }

    // Click on the badge before checking: checks everything now and opens the panel.
    function check(openAfter) {
        const f = field();
        if (!usable(f))
            return;
        openWhenDone = openAfter;
        for (const line of Texto.chunksOf(f.text)) {
            for (const ch of line.chunks)
                enqueue(ch);
        }
        if (!working && openAfter && pending().length)
            openPanel(null);
    }

    function remember(key, corrected) {
        cache.set(key, corrected);
        while (cache.size > cacheSize)
            cache.delete(cache.keys().next().value);
    }

    function setResult(text, corrected, missing) {
        const sugs = Texto.suggestions(text, corrected);
        result = {text, corrected, sugs: sugs || []};
        setState(pending().length ? "issues" : !missing ? "ok" : (working || queue.length) ? "checking" : "idle");
    }

    // ---- Badge --------------------------------------------------------------

    function setState(s) {
        state_ = s;
        paintBadge();
    }

    function label() {
        const n = pending().length;
        return {
            idle: "✎  Verificar", checking: "Verificando", ok: "✓  Sem erros",
            issues: n + " " + (n === 1 ? "sugestão" : "sugestões"), error: "⚠  Corretor indisponível",
        }[state_];
    }

    function showBadge(on) {
        badgeShown = on && !suspended && !panelOpen;
        paintBadge();
    }

    function paintBadge() {
        Ime.badge({visible: badgeShown, label: label(), state: state_, busy: working, fix: state_ === "issues" && !writing});
        updateKeys();
    }

    function onBadgeClicked() {
        if (panelOpen) {
            dismissPanel();
            return;
        }
        if (state_ === "issues")
            openPanel(null);
        else if (state_ === "idle" || state_ === "error")
            check(true);
    }

    // Keys reserved only while they make sense; the rest of the time they
    // belong to the app: Esc with the panel open; Alt+O with the badge on
    // screen; Alt+S with suggestions; Ctrl+S too, except where it means "save".
    function updateKeys() {
        if (suspended) {
            Atalhos.clear(c);
            return;
        }
        const badgeUp = badgeShown && ["issues", "idle", "error"].indexOf(state_) >= 0;
        const issues = badgeShown && state_ === "issues";
        const applyAll = () => c.apply(c.pending());
        const open = Conf.keyOf("sugestoes"), all = Conf.keyOf("aplicar");
        for (const k in Atalhos.handlers) {
            if ([open, all, "Escape", "<Control>s"].indexOf(k) < 0 && Atalhos.handlers[k].some(h => h.owner === c))
                Atalhos.set(c, k, false);
        }
        Atalhos.set(c, "Escape", panelOpen, () => c.dismissPanel());
        Atalhos.set(c, open, panelOpen || badgeUp, () => c.onBadgeClicked());
        Atalhos.set(c, all, panelOpen || issues, applyAll);
        Atalhos.set(c, "<Control>s", panelOpen || (issues && !saveApps.test(app)), applyAll);
    }

    // ---- Panel --------------------------------------------------------------

    function openPanel(pos) {
        if (!pending().length)
            return;
        Compat.co(function* () {
            // Where the cursor is (clicked on the badge) or the bottom of the window.
            let p = pos || (yield Compositor.cursorPos());
            if (!p) {
                const w = yield Compositor.focused();
                p = w ? {x: w.x + w.width / 2 - 220, y: w.y + w.height - 80} : null;
            }
            c.panelPos = p;
            c.panelOpen = true;
            c.badgeShown = false;
            c.paintBadge();
        });
    }

    function dismissPanel() {
        closePanel();
        const f = field();
        if (usable(f)) {
            setState(pending().length ? "issues" : state_);
            showBadge(true);
        }
    }

    function closePanel() {
        if (panelOpen) {
            panelOpen = false;
            updateKeys();
        }
    }

    function ignore(sug) {
        const m = Object.assign({}, ignored);
        m[sug.key] = true;
        ignored = m;
        // Ignoring a word = it is right for you (a name, slang): goes to the dictionary.
        if (sug.from && sug.from.indexOf(" ") < 0)
            learn(["--learn-word", sug.from]);
        afterChange();
    }

    function apply(sugs) {
        if (!sugs.length || writing)
            return;
        for (const sg of sugs) {
            if (sg.from && sg.to && sg.from.indexOf(" ") < 0 && sg.to.indexOf(" ") < 0)
                learn(["--learn-fix", sg.from, sg.to]);
        }
        const f = field();
        if (!result || Texto.flat(f.text) !== Texto.flat(result.text)) {
            closePanel();
            return;
        }
        const text = f.text;
        const cps = Compat.chars(text);
        const cursor = Math.min(f.cursor, cps.length);
        const newText = Texto.applyEdits(cps, sugs);
        if (sugs.length === pending().length)
            learn(["--learn-case"], JSON.stringify({antes: text, depois: newText}));
        const lo = Math.min.apply(null, [cursor].concat(sugs.map(s => s.s)));
        const hi = Math.max.apply(null, [cursor].concat(sugs.map(s => s.e)));
        const sameLine = cps.slice(lo, hi).indexOf("\n") < 0;
        const corrected = result.corrected;
        writing = true;
        paintBadge();
        Compat.co(function* () {
            try {
                if (sameLine) {
                    // Only the span [lo, hi), which touches the cursor.
                    const span = Texto.applyEdits(cps.slice(lo, hi), sugs.map(s => Object.assign({}, s, {s: s.s - lo, e: s.e - lo})));
                    Ime.edit(cursor - lo, hi - cursor, span);
                } else if (cps.length <= c.maxPasteChars) {
                    yield Paste.replaceAll(newText);
                } else {
                    console.log("clippit: texto longo demais para colar (" + cps.length + ")");
                    return;
                }
            } finally {
                yield Compat.sleep(250);
                c.writing = false;
            }
            c.remember(Texto.flat(newText), corrected);
            c.setResult(Ime.text || newText, corrected, false);
            c.afterChange();
        });
    }

    function afterChange() {
        const n = pending().length;
        setState(n ? "issues" : "ok");
        if (n) {
            if (!panelOpen)
                openPanel(panelPos);
            return;
        }
        closePanel();
        if (usable(field()))
            showBadge(true);
    }

    function runCorrector(text) {
        return Sys.run(Sys.tool("clippit-correct"), text, 25).then(out => out.replace(/\n+$/, ""));
    }

    // Learning runs in the background and never gets in the way if it fails.
    function learn(args, input) {
        Sys.run(Sys.tool("clippit-correct").concat(args), input === undefined ? null : input, 20)
            .catch(e => console.log("clippit: aprendizado falhou: " + e.message));
    }

    Painel {
        corretor: c
    }
}
