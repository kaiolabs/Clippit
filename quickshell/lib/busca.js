.pragma library
.import "compat.js" as Compat

// Launcher logic shared by every screen: fuzzy match, calculator and unit
// conversions, markdown for the AI, text transforms and the icon tables.
// Same rules as the GNOME extension (core.js).

function fold(s) {
    return (s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

function esc(s) {
    return String(s ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

// Links that come from the web or from the model: only http(s) is followed.
function isWebUrl(u) {
    return /^https?:\/\//i.test(String(u ?? ""));
}

function editDistance(a, b) {
    const d = [];
    for (let i = 0; i <= a.length; i++) {
        const row = new Array(b.length + 1).fill(0);
        row[0] = i;
        d.push(row);
    }
    for (let j = 1; j <= b.length; j++)
        d[0][j] = j;
    for (let i = 1; i <= a.length; i++) {
        for (let j = 1; j <= b.length; j++) {
            d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
            if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1])
                d[i][j] = Math.min(d[i][j], d[i - 2][j - 2] + 1);
        }
    }
    return d[a.length][b.length];
}

// 0 = no match. Start of the name > start of a word > contains > initials >
// one typo ("crom" → chrome) > letters in order.
function matchScore(query, text) {
    const q = fold(query).trim();
    const t = fold(text);
    if (!q || !t)
        return 0;
    if (t.startsWith(q))
        return 100 - Math.min(20, t.length - q.length);
    const words = t.split(/[\s\-_./()]+/);
    if (words.some(w => w.startsWith(q)))
        return 80;
    if (t.includes(q))
        return 60;
    const initials = words.map(w => w[0] ?? "").join("");
    if (q.length >= 2 && initials.startsWith(q))
        return 70;
    if (q.length >= 4 && words.some(w => editDistance(q, w.slice(0, q.length + 1)) <= 1))
        return 45;
    let i = 0;
    for (const ch of t) {
        if (ch === q[i])
            i++;
        if (i === q.length)
            return 30;
    }
    return 0;
}

const UNITS = {
    mm: ["comprimento", 0.001], cm: ["comprimento", 0.01], m: ["comprimento", 1], km: ["comprimento", 1000],
    pol: ["comprimento", 0.0254], "in": ["comprimento", 0.0254], polegada: ["comprimento", 0.0254], polegadas: ["comprimento", 0.0254],
    ft: ["comprimento", 0.3048], pe: ["comprimento", 0.3048], pes: ["comprimento", 0.3048],
    yd: ["comprimento", 0.9144], mi: ["comprimento", 1609.344], milha: ["comprimento", 1609.344], milhas: ["comprimento", 1609.344],
    metro: ["comprimento", 1], metros: ["comprimento", 1], quilometro: ["comprimento", 1000], quilometros: ["comprimento", 1000],
    mg: ["massa", 0.001], g: ["massa", 1], kg: ["massa", 1000], t: ["massa", 1e6], ton: ["massa", 1e6],
    lb: ["massa", 453.59237], lbs: ["massa", 453.59237], libra: ["massa", 453.59237], libras: ["massa", 453.59237],
    oz: ["massa", 28.349523], grama: ["massa", 1], gramas: ["massa", 1], quilo: ["massa", 1000], quilos: ["massa", 1000],
    ms: ["tempo", 0.001], s: ["tempo", 1], seg: ["tempo", 1], segundo: ["tempo", 1], segundos: ["tempo", 1],
    min: ["tempo", 60], minuto: ["tempo", 60], minutos: ["tempo", 60], h: ["tempo", 3600], hora: ["tempo", 3600],
    horas: ["tempo", 3600], dia: ["tempo", 86400], dias: ["tempo", 86400], semana: ["tempo", 604800], semanas: ["tempo", 604800],
    b: ["dados", 1], kb: ["dados", 1024], mb: ["dados", 1024 * 1024], gb: ["dados", 1024 * 1024 * 1024], tb: ["dados", Math.pow(1024, 4)],
    ml: ["volume", 0.001], l: ["volume", 1], litro: ["volume", 1], litros: ["volume", 1],
    "km/h": ["velocidade", 1 / 3.6], kmh: ["velocidade", 1 / 3.6], "m/s": ["velocidade", 1], mph: ["velocidade", 0.44704],
    c: ["temperatura", 0], "°c": ["temperatura", 0], celsius: ["temperatura", 0],
    f: ["temperatura", 0], "°f": ["temperatura", 0], fahrenheit: ["temperatura", 0],
    k: ["temperatura", 0], kelvin: ["temperatura", 0],
};

function toCelsius(v, u) {
    u = u.replace("°", "");
    return u.startsWith("f") ? (v - 32) * 5 / 9 : u.startsWith("k") ? v - 273.15 : v;
}

function fromCelsius(v, u) {
    u = u.replace("°", "");
    return u.startsWith("f") ? v * 9 / 5 + 32 : u.startsWith("k") ? v + 273.15 : v;
}

function parseNum(s) {
    const str = String(s);
    return Number((str.includes(",") ? str.replace(/\./g, "") : str).replace(",", "."));
}

// pt-BR: 1.234,56 (the engine's toLocaleString ignores the locale).
function formatNum(n) {
    if (!isFinite(n))
        return null;
    if (Math.abs(n) >= 1e15 || (Math.abs(n) < 1e-6 && n !== 0))
        return n.toExponential(6).replace(".", ",");
    const r = Number(n.toPrecision(12));
    const neg = r < 0;
    const fixed = Math.abs(r).toFixed(6).replace(/0+$/, "").replace(/\.$/, "");
    const parts = fixed.split(".");
    const int = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ".");
    return (neg ? "-" : "") + int + (parts[1] ? "," + parts[1] : "");
}

function convert(query) {
    let m = fold(query).match(/^(\d+)\s*h\s*(\d+)\s*(?:min)?\s+(?:em|para|in|to)\s+([a-z°/]+)$/);
    if (m) {
        const secs = Number(m[1]) * 3600 + Number(m[2]) * 60;
        const to = UNITS[m[3]];
        if (to && to[0] === "tempo")
            return {value: secs / to[1], unit: m[3]};
    }
    m = fold(query).replace(/\s+/g, " ").match(/^(-?[\d.,]+)\s*([a-z°/]+)\s+(?:em|para|in|to)\s+([a-z°/]+)$/);
    if (!m)
        return null;
    const num = m[1], from = m[2], to = m[3];
    const a = UNITS[from], b = UNITS[to];
    if (!a || !b || a[0] !== b[0])
        return null;
    const v = parseNum(num);
    if (a[0] === "temperatura")
        return {value: fromCelsius(toCelsius(v, from), to), unit: to.replace("°", "").toUpperCase()};
    return {value: v * a[1] / b[1], unit: to};
}

function calculate(query) {
    let s = fold(query).trim();
    if (!/\d/.test(s) || !/[-+*/x×÷^%()]|de |of /.test(s))
        return null;
    const pm = s.match(/^([\d.,]+)\s*%\s*(?:de|of)\s*([\d.,]+)$/);
    if (pm)
        return parseNum(pm[1]) / 100 * parseNum(pm[2]);
    if (s.includes(","))
        s = s.replace(/(\d)\.(?=\d{3}\b)/g, "$1").replace(/,/g, ".");
    s = s.replace(/×|x/g, "*").replace(/÷/g, "/").replace(/\s+/g, "");
    if (!/^[\d.+\-*/^%()]+$/.test(s))
        return null;
    let i = 0;
    const peek = () => s[i];
    const num = () => {
        const st = i;
        while (i < s.length && /[\d.]/.test(s[i]))
            i++;
        if (st === i)
            throw new Error("número");
        return Number(s.slice(st, i));
    };
    let expr;
    const factor = () => {
        if (peek() === "-") {
            i++;
            return -factor();
        }
        if (peek() === "+") {
            i++;
            return factor();
        }
        let v;
        if (peek() === "(") {
            i++;
            v = expr();
            if (s[i++] !== ")")
                throw new Error("parêntese");
        } else {
            v = num();
        }
        if (peek() === "%") {
            i++;
            v /= 100;
        }
        if (peek() === "^") {
            i++;
            v = Math.pow(v, factor());
        }
        return v;
    };
    const term = () => {
        let v = factor();
        while (peek() === "*" || peek() === "/") {
            const op = s[i++];
            v = op === "*" ? v * factor() : v / factor();
        }
        return v;
    };
    expr = () => {
        let v = term();
        while (peek() === "+" || peek() === "-") {
            const op = s[i++];
            const st = i;
            let w = term();
            if (s[i - 1] === "%" && /^[\d.]+%$/.test(s.slice(st, i)))
                w = v * w;
            v = op === "+" ? v + w : v - w;
        }
        return v;
    };
    try {
        const v = expr();
        return i === s.length && isFinite(v) ? v : null;
    } catch (e) {
        return null;
    }
}

// The AI's simple markdown → Qt rich text (bold, code, headings).
function mdToHtml(md) {
    let out = esc(md ?? "");
    out = out.replace(/```[a-z]*\n([\s\S]*?)```/g, (_m, code) =>
        "<pre style=\"font-family: monospace;\">" + code.replace(/\n$/, "") + "</pre>");
    out = out.replace(/`([^`\n]+)`/g, "<code>$1</code>");
    out = out.replace(/\*\*([^*\n]+)\*\*/g, "<b>$1</b>");
    out = out.replace(/^#{1,4}\s*(.+)$/gm, "<b>$1</b>");
    out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<i>$2</i>");
    // Line breaks outside <pre>.
    return out.split(/(<pre[\s\S]*?<\/pre>)/).map(p => p.startsWith("<pre") ? p : p.replace(/\n/g, "<br>")).join("");
}

const WEB = [
    {key: "g", name: "Google", url: "https://www.google.com/search?q={q}", icon: "web-browser-symbolic"},
    {key: "yt", name: "YouTube", url: "https://www.youtube.com/results?search_query={q}", icon: "video-x-generic-symbolic"},
    {key: "gh", name: "GitHub", url: "https://github.com/search?q={q}", icon: "system-software-install-symbolic"},
    {key: "w", name: "Wikipédia", url: "https://pt.wikipedia.org/w/index.php?search={q}", icon: "accessories-dictionary-symbolic"},
    {key: "m", name: "Google Maps", url: "https://www.google.com/maps/search/{q}", icon: "mark-location-symbolic"},
    {key: "tr", name: "Google Tradutor", url: "https://translate.google.com/?sl=auto&tl=pt&text={q}", icon: "preferences-desktop-locale-symbolic"},
];

const KIND = {
    "Apps": "Aplicativo", "Mais usados": "Aplicativo", "Janelas abertas": "Janela", "Comandos": "Comando",
    "Calculadora": "Calculadora", "Web": "Web", "Arquivos": "Arquivo", "Histórico do Clippit": "Histórico",
    "IA local": "IA", "Atalhos": "Atalho", "Sugestões": "Comando",
};

const HEX_COLOR = /^#(?:[0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i;

// Theme icon name (old) → Lucide icon.
const ICON_MAP = {
    "edit-paste-symbolic": "clipboard-list", "starred-symbolic": "sparkles",
    "input-gaming-symbolic": "gamepad-2", "preferences-system-symbolic": "settings",
    "org.gnome.Settings-symbolic": "sliders-horizontal", "system-lock-screen-symbolic": "lock",
    "weather-clear-night-symbolic": "moon", "system-reboot-symbolic": "rotate-cw",
    "system-shutdown-symbolic": "power", "system-log-out-symbolic": "log-out",
    "user-trash-full-symbolic": "trash-2", "tools-check-spelling-symbolic": "spell-check",
    "user-available-symbolic": "messages-square", "insert-text-symbolic": "text-cursor-input",
    "face-smile-symbolic": "smile", "edit-select-all-symbolic": "wand-sparkles",
    "network-wireless-symbolic": "wifi", "bluetooth-active-symbolic": "bluetooth",
    "notifications-disabled-symbolic": "bell-off", "audio-volume-muted-symbolic": "volume-x",
    "audio-volume-high-symbolic": "volume-2", "audio-volume-low-symbolic": "volume-1",
    "display-brightness-symbolic": "sun", "alarm-symbolic": "alarm-clock",
    "view-dual-symbolic": "panel-left", "window-maximize-symbolic": "maximize-2",
    "window-restore-symbolic": "minimize-2", "focus-windows-symbolic": "square-dashed",
    "video-display-symbolic": "monitor", "view-fullscreen-symbolic": "fullscreen",
    "window-minimize-symbolic": "arrow-down-to-line", "window-close-symbolic": "square-x",
    "web-browser-symbolic": "globe", "video-x-generic-symbolic": "youtube",
    "system-software-install-symbolic": "github", "accessories-dictionary-symbolic": "book-open",
    "mark-location-symbolic": "map-pin", "preferences-desktop-locale-symbolic": "languages",
    "accessories-calculator-symbolic": "calculator", "document-edit-symbolic": "pencil",
    "list-add-symbolic": "plus", "text-x-generic-symbolic": "file-text",
    "view-pin-symbolic": "pin", "edit-clear-symbolic": "eraser",
    "format-text-bold-symbolic": "case-upper", "format-text-plaintext-symbolic": "case-lower",
    "format-justify-fill-symbolic": "wrap-text", "format-justify-left-symbolic": "wrap-text",
    "text-x-script-symbolic": "braces", "view-list-bullet-symbolic": "list",
    "help-about-symbolic": "circle-help", "mail-reply-sender-symbolic": "reply",
    "image-x-generic-symbolic": "image", "application-x-executable-symbolic": "app-window",
    "system-run-symbolic": "terminal", "edit-find-symbolic": "search",
    "camera-photo-symbolic": "camera",
};

const ICON_BY_ID = {
    "ctl:dark": "sun-moon", "win:right": "panel-right", "calc": "calculator",
    "cmd:clippit": "settings", "timer": "alarm-clock",
};

function lucideFor(id, themeIcon) {
    return ICON_BY_ID[id] ?? ICON_MAP[themeIcon] ?? "app-window";
}

function titleCase(t) {
    const re = new RegExp("(^|[\\s(\\[\"'])([" + Compat.LETTER + "])", "g");
    return t.toLocaleLowerCase("pt-BR").replace(re, (_m, a, b) => a + b.toLocaleUpperCase("pt-BR"));
}

const TEXT_TRANSFORMS = [
    {id: "maiusculas", title: "MAIÚSCULAS", icon: "format-text-bold-symbolic", fn: t => t.toLocaleUpperCase("pt-BR")},
    {id: "minusculas", title: "minúsculas", icon: "format-text-plaintext-symbolic", fn: t => t.toLocaleLowerCase("pt-BR")},
    {id: "titulo", title: "Iniciais Maiúsculas", icon: "format-text-plaintext-symbolic", fn: titleCase},
    {id: "linha", title: "Uma linha só (sem quebras)", icon: "format-justify-fill-symbolic", fn: t => t.replace(/\s+/g, " ").trim()},
    {id: "limpar", title: "Sem espaços sobrando", icon: "edit-clear-symbolic",
        fn: t => t.split("\n").map(l => l.replace(/[ \t]+/g, " ").trim()).join("\n").replace(/\n{3,}/g, "\n\n").trim()},
    {id: "json", title: "Formatar JSON", icon: "text-x-script-symbolic", fn: t => JSON.stringify(JSON.parse(t), null, 2)},
    {id: "links", title: "Extrair links", icon: "web-browser-symbolic",
        fn: t => {
            const links = [...new Set(t.match(/https?:\/\/[^\s<>"')\]]+/g) ?? [])].join("\n");
            if (!links)
                throw new Error("nenhum link");
            return links;
        }},
    {id: "numeros", title: "Só os números", icon: "accessories-calculator-symbolic",
        fn: t => {
            const n = t.replace(/\D+/g, "");
            if (!n)
                throw new Error("nenhum número");
            return n;
        }},
    {id: "corrigir", title: "Corrigir o texto", icon: "tools-check-spelling-symbolic", ai: "corrigir"},
    {id: "traduzir", title: "Traduzir (pt ↔ en)", icon: "preferences-desktop-locale-symbolic", ai: "traduzir"},
    {id: "resumir", title: "Resumir", icon: "view-list-bullet-symbolic", ai: "resumir"},
];

const TEXT_ACTIONS = [
    {acao: "resumir", title: "Resumir", icon: "view-list-bullet-symbolic", words: ["resumo", "summary"]},
    {acao: "traduzir", title: "Traduzir (pt ↔ en)", icon: "preferences-desktop-locale-symbolic", words: ["tradução", "translate", "ingles"]},
    {acao: "explicar", title: "Explicar", icon: "help-about-symbolic", words: ["o que é", "explain"]},
    {acao: "corrigir", title: "Corrigir", icon: "tools-check-spelling-symbolic", words: ["ortografia", "erros"]},
    {acao: "formal", title: "Deixar mais formal", icon: "format-text-bold-symbolic", words: ["profissional"]},
    {acao: "casual", title: "Deixar mais casual", icon: "face-smile-symbolic", words: ["informal", "leve"]},
    {acao: "curto", title: "Deixar mais curto", icon: "format-justify-left-symbolic", words: ["resumir", "enxugar"]},
    {acao: "responder", title: "Escrever uma resposta", icon: "mail-reply-sender-symbolic", words: ["responder", "reply"]},
    {acao: "perguntar", title: "Perguntar à IA sobre o texto", icon: "starred-symbolic", words: ["chat", "ia"]},
];

const LETTERS = "ABCDEF";

const POPULAR_EMOJI = ["❤️", "👍", "🔥", "✅", "😂", "🙏", "😊", "🎉", "👀", "💯", "😍", "🤔", "😅", "👏", "🚀", "✨", "😭", "🥲", "😎", "👌", "✔️", "⚠️", "❌", "⭐"];

const EMOJI_CATS = ["Fixados", "Sorrisos e emoções", "Pessoas", "Animais e natureza", "Comida e bebida", "Viagens e lugares",
    "Atividades", "Objetos", "Símbolos", "Bandeiras"];

function emojiScore(q, x) {
    const best = Math.max.apply(null, x.words.map(w => matchScore(q, w)).concat([0]));
    return best + matchScore(q, x.desc) * 0.3 - x.desc.length * 0.1 + (POPULAR_EMOJI.includes(x.e) ? 15 : 0);
}

function when(t) {
    const min = Math.round((Date.now() - t) / 60000);
    if (min < 60)
        return "há " + Math.max(1, min) + " min";
    if (min < 1440)
        return "há " + Math.round(min / 60) + " h";
    const d = new Date(t);
    const p = n => String(n).padStart(2, "0");
    return p(d.getDate()) + "/" + p(d.getMonth() + 1) + " " + p(d.getHours()) + ":" + p(d.getMinutes());
}

function today() {
    const d = new Date();
    const p = n => String(n).padStart(2, "0");
    return {data: p(d.getDate()) + "/" + p(d.getMonth() + 1) + "/" + d.getFullYear(), hora: p(d.getHours()) + ":" + p(d.getMinutes())};
}
