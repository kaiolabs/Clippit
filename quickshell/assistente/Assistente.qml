import QtQuick
import Quickshell
import "../services"
import "../lib/compat.js" as Compat

// Reply assistant (Alt+K): a screenshot of the focused window, read by OCR
// (local), and the local model suggests your next message. Contexts live in
// ~/.config/clippit/contextos/*.md (first line "apps: whatsapp, slack" picks one
// by itself). Also owns game mode (Ctrl+Shift+E).
Scope {
    id: a

    required property var corretor
    readonly property string contextDir: Sys.configDir + "/contextos"
    readonly property int width_: 580

    property bool open: false
    property var win: null
    property string app: ""
    property string draft: ""
    property string conversation: ""
    property string shot: ""
    property var area: null
    property var contexts: []
    property int ctxIndex: -1
    property var ctxByApp: ({})
    property var history: []
    property var options: []
    property var modelOptions: []
    property int sel: 0
    property string reply: ""
    property string modelReply: ""
    property bool busy: false
    property string status: ""
    property string note: ""
    property int gen: 0
    property bool grabbed: false
    property bool showCtxList: false
    property bool showSources: false
    property var editing: undefined // undefined = closed, null = new, object = editing
    property bool gameBusy: false
    property string visionNote: ""
    property string personNote: ""

    signal focusEntry

    function enable() {
        Atalhos.set(a, Conf.keyOf("assistente"), true, () => a.open ? a.regenerate(0.8) : a.safeOpen());
        Atalhos.set(a, Conf.keyOf("modo_jogo"), true, () => a.toggleGameMode());
    }

    Connections {
        target: Conf
        function onValuesChanged() {
            Atalhos.clear(a);
            a.enable();
            a.syncKeys();
        }
    }

    Component.onCompleted: enable()

    function toggleGameMode() {
        if (gameBusy)
            return;
        gameBusy = true;
        Sys.run(Sys.tool("clippit-correct").concat(["--modo-jogo"]), null, 30).then(raw => {
            const out = JSON.parse(raw);
            const gb = String(Math.round(out.liberado_mb / 102.4) / 10).replace(".", ",");
            if (out.modo_jogo)
                Sys.notify("Clippit — modo jogo ligado", (out.liberado_mb > 100 ? gb + " GB liberados na placa de vídeo. "
                    : "A placa de vídeo está livre. ") + "O corretor continua funcionando no processador. Ctrl+Shift+E para voltar ao normal.");
            else
                Sys.notify("Clippit — modo jogo desligado", "O modelo volta a usar a placa de vídeo na próxima correção, se ela estiver livre.");
        }).catch(e => Sys.notify("Clippit", "Não consegui trocar o modo jogo: " + e.message))
            .then(() => a.gameBusy = false);
    }

    function safeOpen() {
        if (!Conf.aiOn("assistente")) {
            Conf.aiOffNotice("O assistente de resposta");
            return;
        }
        openPanel().catch(e => {
            console.log("clippit: assistente falhou ao abrir: " + e.message);
            close(false);
        });
    }

    function loadContexts() {
        const script = "import json,glob,os,sys\nout={}\nfor f in sorted(glob.glob(os.path.join(sys.argv[1],'*.md'))):\n" +
            "    try: out[f]=open(f,encoding='utf-8').read()\n    except OSError: pass\nprint(json.dumps(out))";
        return Sys.run(["python3", "-c", script, contextDir], null, 5).then(raw => {
            const files = JSON.parse(raw);
            const out = [];
            for (const file in files) {
                let text = files[file];
                let apps = [];
                const first = text.split("\n")[0];
                if (/^apps\s*:/i.test(first)) {
                    apps = first.replace(/^apps\s*:/i, "").split(",").map(x => x.trim().toLowerCase()).filter(Boolean);
                    text = text.split("\n").slice(1).join("\n");
                }
                const raw2 = text.replace(/^\n+/, "");
                text = text.split("\n").filter(l => !/^\s*-\s*[^:]+:\s*$/.test(l) && !/^Preencha abaixo/i.test(l)).join("\n").trim();
                out.push({name: file.replace(/^.*\//, "").replace(/\.md$/, ""), apps, text, raw: raw2, file});
            }
            out.sort((x, y) => x.name.localeCompare(y.name));
            out.push({name: "Sem contexto", apps: [], text: ""});
            return out;
        }).catch(() => [{name: "Sem contexto", apps: [], text: ""}]);
    }

    function pickContext() {
        const remembered = ctxByApp[app];
        if (remembered !== undefined) {
            const i = contexts.findIndex(x => x.name === remembered);
            if (i >= 0)
                return i;
        }
        const i = contexts.findIndex(x => x.apps.some(p => app.indexOf(p) >= 0));
        if (i >= 0)
            return i;
        const geral = contexts.findIndex(x => x.name.toLowerCase() === "geral");
        return geral >= 0 ? geral : contexts.length - 1;
    }

    function openPanel() {
        return Compat.co(function* () {
            const w = yield Compositor.focused();
            if (!w)
                return;
            a.close(false);
            a.win = w;
            a.app = (w.app || "").toLowerCase();
            a.draft = Ime.active ? (Ime.text || "").trim() : "";
            a.history = [];
            a.conversation = "";
            a.options = [];
            a.reply = "";
            a.note = "";
            a.contexts = yield a.loadContexts();
            a.ctxIndex = a.pickContext();
            const g = ++a.gen;
            a.corretor.dismissPanel();
            a.open = true;
            a.setStatus("Lendo a conversa…", true);
            const path = Sys.workDir + "/tela-" + Date.now() + ".png";
            // The whole window (the GNOME version cropped at the text field; here
            // the caret position is not known).
            a.area = {x: w.x, y: w.y, width: w.width, height: w.height, where: "a janela inteira"};
            try {
                yield Compositor.shoot(a.area, path);
                a.shot = path; // kept while the panel is open (the model looks at it), deleted on close
                a.conversation = yield Sys.run(Sys.tool("clippit-correct").concat(["--ocr", path]), null, 60);
            } catch (e) {
                console.log("clippit: não consegui ler a tela: " + e.message);
                if (g === a.gen)
                    a.setStatus("Não consegui ler a tela desta janela.", false);
                return;
            }
            if (g !== a.gen)
                return;
            if (!a.conversation.trim()) {
                a.setStatus("Não achei texto nesta janela.", false);
                return;
            }
            yield a.generate(0.4);
        });
    }

    function generate(temperature) {
        const g = gen;
        setStatus(history.length ? "Ajustando…" : "Escrevendo a resposta…", true);
        const ctx = contexts[ctxIndex];
        const payload = JSON.stringify({
            conversation, draft, context: ctx ? ctx.text : "", context_name: ctx ? ctx.name : "", image: shot,
            history, temperature, variants: history.length ? 0 : 3,
        });
        return Sys.run(Sys.tool("clippit-correct").concat(["--assist"]), payload, 150).then(raw => {
            raw = raw.trim();
            if (g !== a.gen || !a.open)
                return;
            let opts = [raw];
            if (raw.startsWith("{")) {
                try {
                    opts = JSON.parse(raw).opcoes.filter(Boolean);
                } catch (e) {}
            }
            a.options = opts;
            a.modelOptions = opts.slice();
            a.sel = opts.length >= 3 ? 1 : 0;
            a.reply = opts[a.sel];
            a.modelReply = a.reply;
            a.showReply();
        }).catch(e => {
            console.log("clippit: assistente falhou: " + e.message);
            if (g === a.gen)
                a.setStatus("O modelo não respondeu. Tente de novo (Alt+K).", false);
        });
    }

    function regenerate(temperature) {
        if (!conversation || busy)
            return;
        generate(temperature);
    }

    function setStatus(text, b) {
        status = text;
        busy = b;
        syncKeys();
    }

    function showReply() {
        busy = false;
        status = "";
        const lastH = Compat.last(history);
        note = pendingNote || (lastH && lastH.ask ? "Ajuste pedido: " + lastH.ask : "");
        pendingNote = "";
        refreshNotes();
        syncKeys();
    }

    property string pendingNote: ""

    function select(i) {
        if (!options[i])
            return;
        sel = i;
        reply = options[i];
        modelReply = modelOptions[i] ?? reply;
    }

    function setOption(i, text) {
        const o = options.slice();
        o[i] = text;
        options = o;
        if (i === sel)
            reply = text;
    }

    // Enter inserts while the panel has a ready reply (and you are not typing).
    function syncKeys() {
        const want = open && !!reply && !busy && !grabbed;
        Atalhos.set(a, "Escape", open && !grabbed, () => a.close(true));
        const aj = Conf.keyOf("ajuste");
        Atalhos.set(a, aj, open, () => a.toggleKeyboard());
        Atalhos.set(a, "Return", want, () => a.insert());
        Atalhos.set(a, "KP_Enter", want, () => a.insert());
        const many = want && options.length > 1;
        for (let i = 0; i < 3; i++) {
            const n = i;
            Atalhos.set(a, String(i + 1), many && i < options.length, () => a.select(n));
        }
    }

    function mainPerson() {
        const tags = Compat.matchAll(/^\[([^\]\n]{1,40})\]$/gm, conversation || "").map(m => m[1])
            .filter(t => t !== "Você" && t !== "Outro");
        return Compat.last(tags) || "";
    }

    function refreshNotes() {
        Sys.read(Sys.runtimeDir + "/clippit-correct/ready").then(t => {
            a.visionNote = t.indexOf("visão") >= 0 ? "Imagem: a IA também olhou o print, além do texto lido."
                : "Imagem: a IA só recebeu o texto lido (a visão roda só na placa de vídeo livre).";
        }).catch(() => a.visionNote = "");
        const who = mainPerson();
        if (!who) {
            personNote = "";
            return;
        }
        const empty = "Sobre " + who + ": nada anotado ainda (escreva \"nota: …\" no campo de ajuste).";
        Sys.readJson(Sys.dataDir + "/aprendizado/pessoas.json", {}).then(data => {
            const notes = (data[who.toLowerCase()] || {}).notas || [];
            a.personNote = notes.length ? "Sobre " + who + ": " + notes.slice(-3).join("; ")
                + (notes.length > 3 ? " (+" + (notes.length - 3) + ")" : "") + "." : empty;
        });
    }

    function sourcesSummary() {
        const tags = Compat.matchAll(/^\[([^\]\n]{1,40})\]$/gm, conversation || "").map(m => m[1]);
        const people = [...new Set(tags)].map(t => t === "Você" ? "você" : t === "Outro" ? "outra pessoa" : t);
        const ctx = contexts[ctxIndex];
        return [
            "Tela: " + (win ? win.title || app : app) + " — li " + (area ? area.where : "a janela") + ".",
            visionNote || null,
            personNote || null,
            tags.length ? "Conversa: " + tags.length + " " + (tags.length === 1 ? "bloco" : "blocos") + " de mensagens, de " + people.join(", ") + "."
                : "Conversa: não reconheci quem falou o quê; a IA recebeu o texto da tela como está.",
            "Contexto: " + (ctx && ctx.text ? "\"" + ctx.name + "\" (" + ctx.text.split("\n").length + " linhas de instruções)" : "nenhum") + ".",
            draft ? "Seu rascunho no campo: \"" + draft.slice(0, 80) + (draft.length > 80 ? "…" : "") + "\"" : null,
            "Abaixo, o texto exatamente como a IA recebeu (lido da tela por OCR):",
        ].filter(Boolean).join("\n");
    }

    function chooseContext(i) {
        showCtxList = false;
        const changed = i !== ctxIndex;
        ctxIndex = i;
        const m = Object.assign({}, ctxByApp);
        m[app] = contexts[i].name;
        ctxByApp = m;
        if (changed && conversation) {
            history = [];
            generate(0.4);
        }
    }

    function openContextDir() {
        Sys.mkdir(contextDir).then(() => Sys.openUri("file://" + contextDir));
        close(false);
    }

    function saveEditor(nameText, appsText, body) {
        const name = nameText.trim().replace(/[\/\\:*?"<>|]/g, "-");
        if (!name)
            return false;
        const apps = appsText.split(",").map(x => x.trim().toLowerCase()).filter(Boolean);
        const path = contextDir + "/" + name + ".md";
        const old = editing;
        Compat.co(function* () {
            yield Sys.write(path, (apps.length ? "apps: " + apps.join(", ") + "\n" : "") + body.replace(/\s+$/, "") + "\n");
            if (old && old.file && old.file !== path)
                yield Sys.remove(old.file);
            a.contexts = yield a.loadContexts();
            const i = Math.max(0, a.contexts.findIndex(x => x.name === name));
            a.editing = undefined;
            a.releaseKeyboard();
            a.pendingNote = "Contexto \"" + name + "\" salvo.";
            a.ctxIndex = -1;
            a.chooseContext(i);
        });
        return true;
    }

    // "regra: nunca prometer prazo" in the adjust field → goes to the current context.
    function saveRule(rule) {
        const ctx = contexts[ctxIndex];
        const name = !ctx || ctx.name === "Sem contexto" ? "Geral" : ctx.name;
        const path = contextDir + "/" + name + ".md";
        return Compat.co(function* () {
            let text = "";
            try {
                text = yield Sys.read(path);
            } catch (e) {}
            if (!/^Regras:\s*$/m.test(text))
                text = text.replace(/\s*$/, "") + "\n\nRegras:\n";
            text = text.replace(/\s*$/, "") + "\n- " + rule + "\n";
            yield Sys.write(path, text);
            a.contexts = yield a.loadContexts();
            a.ctxIndex = Math.max(0, a.contexts.findIndex(x => x.name === name));
            const m = Object.assign({}, a.ctxByApp);
            m[a.app] = name;
            a.ctxByApp = m;
            return name;
        });
    }

    function toggleKeyboard() {
        if (editing !== undefined)
            return;
        if (grabbed)
            releaseKeyboard();
        else
            takeKeyboard();
    }

    function takeKeyboard() {
        if (!grabbed) {
            grabbed = true;
            syncKeys();
        }
        focusEntry();
    }

    function releaseKeyboard() {
        if (grabbed) {
            grabbed = false;
            syncKeys();
        }
    }

    function sendAsk(text) {
        const ask = text.trim();
        if (!ask) {
            insert();
            return;
        }
        if (busy)
            return;
        const noteM = ask.match(/^(?:nota|anota(?:r)?)\s*[:\-–]\s*(.+)$/i);
        const about = ask.match(/^sobre\s+(.+?)\s*[:\-–]\s*(.+)$/i);
        if (noteM || about) {
            const who = about ? about[1].trim() : mainPerson();
            if (!who) {
                pendingNote = "Não sei com quem é a conversa. Use \"sobre Nome: …\".";
            } else {
                const n = (about ? about[2] : noteM[1]).trim();
                corretor.learn(["--learn-note"], JSON.stringify({conversation, name: who, note: n}));
                pendingNote = "Anotado sobre " + who + ": " + n;
            }
            if (options.length)
                showReply();
            return;
        }
        const rule = ask.match(/^(?:salv[ae]r?\s+(?:isso\s+)?como\s+)?regras?\s*[:\-–]\s*(.+)$/i);
        if (rule) {
            saveRule(rule[1].trim()).then(name => {
                a.history = [];
                a.pendingNote = "Regra salva em \"" + name + "\": " + rule[1].trim();
                if (a.conversation)
                    a.generate(0.4);
            });
            return;
        }
        if (!reply)
            return;
        history = history.concat([{reply, ask}]);
        generate(0.4);
    }

    // What you actually sent, so the next replies come out your way.
    function remember(text) {
        const ctx = contexts[ctxIndex];
        corretor.learn(["--learn"], JSON.stringify({conversation, suggested: modelReply, reply: text, app,
            context_name: ctx ? ctx.name : ""}));
    }

    function copy() {
        if (!reply)
            return;
        remember(reply);
        Sys.copy(reply);
        close(true);
        Sys.notify("Clippit", "Resposta copiada. Cole com Ctrl+V.");
    }

    function insert() {
        const text = reply;
        if (!text || busy)
            return;
        const w = win;
        remember(text);
        close(false);
        Compat.co(function* () {
            if (w)
                yield Compositor.activate(w);
            for (let i = 0; i < 30 && !Ime.active; i++)
                yield Compat.sleep(50);
            if (!Ime.active) {
                yield Sys.copy(text);
                Sys.notify("Clippit", "Não achei o campo de texto. A resposta foi copiada: cole com Ctrl+V.");
                return;
            }
            if (!(Ime.text || "").trim() && text.indexOf("\n") < 0)
                Ime.commit(text);
            else
                yield Paste.replaceAll(text); // replaces the draft, keeps the line breaks
            a.restoreCorrector();
        });
    }

    function close(restoreBadge) {
        const had = open;
        if (shot) {
            Sys.remove(shot); // the screenshot is not kept
            shot = "";
        }
        gen++;
        grabbed = false;
        open = false;
        reply = "";
        busy = false;
        showCtxList = false;
        showSources = false;
        editing = undefined;
        syncKeys();
        if (had && restoreBadge)
            restoreCorrector();
    }

    // The field gets focus back: tell the spell checker so the badge returns.
    function restoreCorrector() {
        Compat.co(function* () {
            for (let i = 0; i < 30 && !Ime.active; i++)
                yield Compat.sleep(50);
            yield Compat.sleep(150);
            if (Ime.active)
                a.corretor.onSurroundingChanged();
        });
    }

    AssistentePainel {
        assistente: a
    }
}
