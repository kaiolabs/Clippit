// Clippit Corretor
//
// Verifica o texto do campo focado (em qualquer app Wayland que fale
// text-input-v3 com o GNOME Shell) e mostra um selo ao lado do cursor com o
// número de sugestões. Clicando no selo abre um painel, no visual do Clippit,
// com uma linha por sugestão: aceitar, ignorar ou corrigir tudo.
//
// Como o texto é alterado:
//  - Erros na linha do cursor: troca só o trecho, pelo método de entrada
//    (delete_surrounding + commit). Nunca escreve quebra de linha, porque
//    editores ricos (Slack, Claude) duplicam linhas vazias quando recebem "\n".
//  - Erros em outras linhas: o protocolo só apaga texto colado ao cursor, então
//    cola o texto inteiro (Ctrl+A, Ctrl+V por um teclado virtual do Shell) e
//    devolve a área de transferência anterior. O app monta os parágrafos como
//    numa colagem normal.
//
// A correção vem do `clippit-correct` (modelo local, stdin → stdout).
// Diagnóstico: journalctl --user -b -o cat | grep clippit-corretor
// O conteúdo do texto NUNCA é registrado no log, só tamanhos e posições.

import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import IBus from 'gi://IBus';
import Meta from 'gi://Meta';
import Pango from 'gi://Pango';
import Shell from 'gi://Shell';
import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as MessageTray from 'resource:///org/gnome/shell/ui/messageTray.js';
import System from 'system';

const TAG = '[clippit-corretor]';
const CHECK_DELAY_MS = 500;     // pausa antes de verificar o trecho que está sendo digitado
const CHUNK_WORDS = 16;         // o texto é verificado em trechos de até N palavras
const CONTEXT_WORDS = 6;        // palavras do trecho anterior mandadas como contexto
// Opções do painel do Clippit (seção Corretor), em ~/.config/clippit/corretor.conf:
//   ativo = true|false              mostra o selo nos campos
//   verificar_sozinho = true|false  false = só verifica ao clicar no selo
const CONF_PATH = GLib.build_filenamev([GLib.get_home_dir(), '.config', 'clippit', 'corretor.conf']);
let _confCache = {stamp: null, values: {}};

function confValue(key, fallback) {
    const file = Gio.File.new_for_path(CONF_PATH);
    let stamp = null;
    try {
        const info = file.query_info('time::modified,time::modified-usec', Gio.FileQueryInfoFlags.NONE, null);
        stamp = `${info.get_attribute_uint64('time::modified')}.${info.get_attribute_uint32('time::modified-usec')}`;
    } catch (_e) {
        return fallback;
    }
    if (stamp !== _confCache.stamp) {
        const values = {};
        try {
            const text = new TextDecoder().decode(GLib.file_get_contents(CONF_PATH)[1]);
            for (const line of text.split('\n')) {
                const bare = line.split('#')[0];
                const i = bare.indexOf('=');
                if (i > 0)
                    values[bare.slice(0, i).trim()] = bare.slice(i + 1).trim();
            }
        } catch (_e) {}
        _confCache = {stamp, values};
    }
    return _confCache.values[key] ?? fallback;
}

// Atalhos globais. O usuário troca no painel (Atalhos); ficam no corretor.conf
// como atalho_<nome> = <Alt>space (formato do GTK). "desligado" tira o atalho.
const SHORTCUTS = {
    lancador: '<Alt>space', ia: '<Alt>i', acoes: '<Alt>a', assistente: '<Alt>k', modo_jogo: '<Control><Shift>e',
    sugestoes: '<Alt>o', aplicar: '<Alt>s', ajuste: '<Alt>f',
};
const keyOf = id => confValue(`atalho_${id}`, SHORTCUTS[id]) || SHORTCUTS[id];

// Registra atalhos que podem mudar com o painel aberto: confere a cada 2 s e
// troca na hora (sem precisar religar nada).
function bindShortcuts(owner, list) {
    const cur = {};
    const apply = () => {
        for (const [id, fn] of list) {
            const want = keyOf(id);
            if (cur[id] === want)
                continue;
            if (cur[id] && cur[id] !== 'desligado')
                owner._setKey(cur[id], false);
            if (want !== 'desligado')
                owner._setKey(want, true, fn);
            if (cur[id])
                console.log(`${TAG} atalho ${id}: ${cur[id]} → ${want}`);
            cur[id] = want;
        }
        return GLib.SOURCE_CONTINUE;
    };
    apply();
    (owner._keyWatchIds ??= []).push(GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 2, apply));
}

function unbindShortcuts(owner) {
    for (const id of owner._keyWatchIds ?? [])
        GLib.source_remove(id);
    owner._keyWatchIds = [];
}

// IA: "ia = false" desliga tudo; "ia_<tarefa> = false" só aquela função.
const aiOn = task => confOn('ia') && confOn(`ia_${task}`);
function aiOffNotice(what) {
    Main.notify('Clippit', `${what} está desligado. Para ligar: Configurações do Clippit → Geral → Inteligência artificial.`);
}

const confOn = (key, fallback = true) => !['false', 'nao', 'não', '0'].includes(String(confValue(key, fallback)).toLowerCase());
const WATCH_INTERVAL_MS = 300;  // checa se o campo perdeu o foco
const CORRECT_TIMEOUT_S = 25;
const MIN_CHARS = 8;
const PANEL_WIDTH = 440;
const CACHE_SIZE = 30;
const CHUNK_CACHE_SIZE = 500;
// Apps onde Ctrl+S é "salvar": o atalho só vale lá com o painel aberto.
const SAVE_APPS = /^(cursor|code|code-oss|vscodium|codium|gnome-text-editor|org\.gnome\.TextEditor|gedit|org\.gnome\.gedit|sublime_text|jetbrains-.*|libreoffice.*|soffice|gimp.*|inkscape|org\.inkscape\.Inkscape|blender|kdenlive|org\.kde\.kdenlive|obs|com\.obsproject\.Studio|resolve|zed|dev\.zed\.Zed)$/i;
const MAX_PASTE_CHARS = 4000;   // acima disso o texto pode vir cortado pelo app
// O PATH do gnome-shell nem sempre inclui ~/.local/bin (onde o install.sh põe
// os programas); sem cópia lá (pacote do Nix, por exemplo), vale a do PATH.
const clippitTool = name => {
    const local = GLib.build_filenamev([GLib.get_home_dir(), '.local', 'bin', name]);
    if (GLib.file_test(local, GLib.FileTest.IS_EXECUTABLE))
        return local;
    return GLib.find_program_in_path(name) ?? local;
};
const CORRECTOR_CMD = clippitTool('clippit-correct');

// ---- Visual (mesmas cores do popup do Clippit) --------------------------
// Fica aqui, e não no stylesheet.css, porque o GNOME só relê o CSS ao entrar
// na sessão; assim o visual também recarrega na hora.
const C = {
    bg: '#1e1e1e', card: '#262626', cardHover: '#2c2c2c', border: '#333333', borderStrong: '#424242',
    text: '#f2f2f2', muted: '#9a9aa0', faint: '#6e6e74', accent: '#4c9dff', accentBg: '#1c2f45',
    danger: '#ff6b6b', dangerBg: '#3a1f22', ok: '#4ade80', accentHover: '#6aafff',
};
const FONT = 'font-family: Inter, Cantarell, sans-serif;';

// ============================================================================
// PADRÃO VISUAL ÚNICO — todo componente (lançador, painel ao lado, corretor,
// assistente) usa só isto. Nada de tamanho, cor ou canto solto no código.
// ============================================================================
const T = {
    // Texto: só 4 tamanhos.
    input: '12pt', body: '10pt', small: '9pt', tiny: '8pt',
    // Cores de texto: principal, secundária, apagada.
    text: C.text, text2: C.muted, text3: C.faint,
    // Superfícies sobre o vidro.
    glass: 'rgba(30,30,34,0.74)', glassBorder: 'rgba(255,255,255,0.10)', shadow: '0 24px 64px rgba(0,0,0,0.6)',
    surface: 'rgba(255,255,255,0.04)', hover: 'rgba(255,255,255,0.08)', line: 'rgba(255,255,255,0.08)',
    accentSoft: 'rgba(76,157,255,0.16)', code: 'rgba(0,0,0,0.30)',
    // Cantos: tecla, botão/ícone, linha/cartão, janela.
    r1: '4px', r2: '6px', r3: '8px', r4: '12px',
    // Margem lateral do conteúdo (busca, lista e chat alinhados).
    pad: 18,
};
// Estilos de texto prontos.
const TX = {
    input: `${FONT} font-size: ${T.input}; color: ${T.text};`,
    body: `${FONT} font-size: ${T.body}; color: ${T.text};`,
    bodyStrong: `${FONT} font-size: ${T.body}; color: ${T.text}; font-weight: 600;`,
    small: `${FONT} font-size: ${T.small}; color: ${T.text2};`,
    smallStrong: `${FONT} font-size: ${T.small}; color: ${T.text}; font-weight: 600;`,
    tiny: `${FONT} font-size: ${T.tiny}; color: ${T.text3};`,
    label: `${FONT} font-size: ${T.tiny}; color: ${T.text3}; font-weight: 600;`,
    mono: `font-family: monospace; font-size: ${T.small}; color: #c8c8cc;`,
};
// Componentes prontos.
const UI = {
    panel: `${FONT} background-color: ${T.glass}; border: 1px solid ${T.glassBorder}; border-radius: ${T.r4};
        padding: 0; box-shadow: ${T.shadow};`,
    row: `border-radius: ${T.r3}; padding: 7px 10px; spacing: 12px; min-height: 26px;`,
    rowOn: `border-radius: ${T.r3}; padding: 7px 10px; spacing: 12px; min-height: 26px; background-color: ${T.hover};`,
    card: `border-radius: ${T.r3}; padding: 8px 10px; spacing: 6px; background-color: ${T.surface};`,
    cardOn: `border-radius: ${T.r3}; padding: 8px 10px; spacing: 6px; background-color: ${T.accentSoft};`,
    code: `border-radius: ${T.r3}; padding: 8px 10px; spacing: 4px; background-color: ${T.code};`,
    key: `${FONT} font-size: ${T.tiny}; color: ${T.text2}; font-weight: 600; background-color: ${T.hover};
        border-radius: ${T.r1}; padding: 1px 6px;`,
    btnPrimary: `${FONT} font-size: ${T.small}; font-weight: 600; color: #ffffff; background-color: ${C.accent};
        border-radius: ${T.r2}; padding: 5px 12px;`,
    btnPrimaryHover: `${FONT} font-size: ${T.small}; font-weight: 600; color: #ffffff; background-color: ${C.accentHover};
        border-radius: ${T.r2}; padding: 5px 12px;`,
    btn: `${FONT} font-size: ${T.small}; color: ${T.text}; background-color: ${T.hover}; border-radius: ${T.r2}; padding: 5px 12px;`,
    btnHover: `${FONT} font-size: ${T.small}; color: ${T.text}; background-color: rgba(255,255,255,0.14);
        border-radius: ${T.r2}; padding: 5px 12px;`,
    iconBtn: `width: 28px; height: 28px; padding: 0; border-radius: ${T.r2}; background-color: ${T.surface};
        border: 1px solid ${T.line};`,
    iconBtnHover: `width: 28px; height: 28px; padding: 0; border-radius: ${T.r2}; background-color: ${T.hover};
        border: 1px solid rgba(255,255,255,0.14);`,
    sep: `background-color: ${T.line}; height: 1px;`,
    vsep: `background-color: ${T.line}; width: 1px;`,
    header: `padding: 14px ${T.pad}px 12px ${T.pad}px; spacing: 10px; min-height: 30px;`,
    footer: `padding: 10px 12px 12px ${T.pad}px; spacing: 8px; min-height: 22px;`,
};

// Foto de um pedaço da tela (só para os autotestes conferirem o visual).
function shootActor(actor, path) {
    return new Promise(res => {
        const [x, y] = actor.get_transformed_position();
        const [w, h] = actor.get_transformed_size();
        const stream = Gio.File.new_for_path(path).replace(null, false, Gio.FileCreateFlags.NONE, null);
        new Shell.Screenshot().screenshot_area(x, y, w, h, stream, (o, r) => {
            try {
                o.screenshot_area_finish(r);
            } catch (_e) {}
            stream.close(null);
            res();
        });
    });
}

// Tela vazia (o mesmo desenho em todo lugar): ícone, título e explicação.
function emptyState(icon, color, title, sub = '') {
    const box = new St.BoxLayout({vertical: true, x_expand: true, y_expand: true, style: 'spacing: 8px; padding: 36px 24px;',
        x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER});
    const tile = iconTile(icon, color, 36);
    tile.x_align = Clutter.ActorAlign.CENTER;
    box.add_child(tile);
    const t = new St.Label({text: title, style: `${TX.bodyStrong} text-align: center; padding-top: 4px;`,
        x_align: Clutter.ActorAlign.CENTER});
    box.add_child(t);
    if (sub) {
        const l = new St.Label({text: sub, style: `${TX.small} text-align: center;`, x_align: Clutter.ActorAlign.CENTER});
        l.clutter_text.line_wrap = true;
        l.clutter_text.line_alignment = Pango.Alignment.CENTER;
        box.add_child(l);
    }
    return box;
}

// Tecla (atalho) com o mesmo desenho em todo lugar.
function keycap(text) {
    return new St.Label({text, style: UI.key, y_align: Clutter.ActorAlign.CENTER});
}

// Rodapé padrão: à esquerda onde você está; à direita as ações com as teclas.
// actions: [{label, key}] — a primeira é a principal (Enter).
function makeFooter() {
    const bar = new St.BoxLayout({style: UI.footer});
    const icon = new St.Icon({icon_size: 14, y_align: Clutter.ActorAlign.CENTER});
    const where = new St.Label({style: TX.small, y_align: Clutter.ActorAlign.CENTER});
    const right = new St.BoxLayout({style: 'spacing: 8px;', x_expand: true, x_align: Clutter.ActorAlign.END});
    bar.add_child(icon);
    bar.add_child(where);
    bar.add_child(right);
    const api = {
        actor: bar, primary: '',
        set(whereText, iconName, actions = []) {
            where.text = whereText;
            where.visible = icon.visible = Boolean(whereText);
            const map = ICON_MAP[iconName];
            const gi = map ? lucide(map[0], 'g') : null;
            if (gi)
                icon.gicon = gi;
            else
                icon.icon_name = iconName;
            right.destroy_all_children();
            const list = actions.filter(a => a && a.label);
            api.primary = list[0]?.label ?? '';
            list.forEach((a, i) => {
                if (i > 0)
                    right.add_child(new St.Widget({style: `${UI.vsep} height: 14px; margin: 0 4px;`, y_align: Clutter.ActorAlign.CENTER}));
                const label = new St.Label({text: a.label, style: i === 0 ? TX.smallStrong : TX.small,
                    y_align: Clutter.ActorAlign.CENTER});
                if (a.run) {
                    // Ação clicável (ex.: Parar): rótulo + tecla num botão só.
                    const box = new St.BoxLayout({style: 'spacing: 8px;'});
                    box.add_child(label);
                    if (a.key)
                        box.add_child(keycap(a.key));
                    const btn = new St.Button({child: box, style: `border-radius: ${T.r2}; padding: 2px 6px;`,
                        reactive: true, track_hover: true, y_align: Clutter.ActorAlign.CENTER});
                    btn.connect('notify::hover', () => (btn.style = `border-radius: ${T.r2}; padding: 2px 6px;` +
                        (btn.hover ? ` background-color: ${T.hover};` : '')));
                    btn.connect('clicked', () => a.run());
                    right.add_child(btn);
                    return;
                }
                right.add_child(label);
                if (a.key)
                    right.add_child(keycap(a.key));
            });
        },
    };
    return api;
}
// As 3 peças do selo têm a mesma altura (24px) e a mesma borda de 1px, para alinhar exato.
const BADGE_H = 24;
const badgeBase = `${FONT} font-size: ${T.small}; font-weight: 600; border-radius: ${T.r2}; padding: 0 9px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.35);`;
const STYLE = {
    badge: {
        idle: `${badgeBase} background-color: rgba(30,30,34,0.85); color: ${C.muted}; border: 1px solid rgba(255,255,255,0.12);`,
        checking: `${badgeBase} background-color: rgba(30,30,34,0.85); color: ${C.muted}; border: 1px solid rgba(255,255,255,0.12);`,
        ok: `${badgeBase} background-color: rgba(30,30,34,0.85); color: ${C.ok}; border: 1px solid rgba(255,255,255,0.12);`,
        issues: `${badgeBase} background-color: rgba(30,30,34,0.85); color: ${C.danger}; border: 1px solid rgba(255,255,255,0.12);`,
        error: `${badgeBase} background-color: rgba(30,30,34,0.85); color: ${C.danger}; border: 1px solid rgba(255,255,255,0.12);`,
    },
    badgeHover: `border-color: ${C.borderStrong}; background-color: ${C.cardHover};`,
    // ✓ no azul do Clippit; tamanho fixo no código (set_size) para alinhar exato.
    fix: `background-color: ${C.accent}; color: #ffffff; border-radius: ${T.r2}; padding: 0;
        box-shadow: 0 2px 8px rgba(0,0,0,0.35);`,
    fixHover: `background-color: #6cb0ff; color: #ffffff; border-radius: ${T.r2}; padding: 0;
        box-shadow: 0 2px 8px rgba(0,0,0,0.35);`,
    ring: `background-color: ${C.bg}; border: 1px solid ${C.border}; border-radius: ${T.r2}; padding: 0;
        box-shadow: 0 2px 8px rgba(0,0,0,0.35);`,
    // Painel de correções: mesmo vidro, textos e componentes do lançador.
    panel: UI.panel,
    header: UI.header,
    title: TX.bodyStrong,
    count: `${UI.key} color: ${C.accent}; background-color: ${T.accentSoft}; margin-left: 4px;`,
    list: `padding: 0 12px 4px 12px; spacing: 6px;`,
    item: `${UI.card} spacing: 10px;`,
    itemHover: `${UI.card} spacing: 10px; background-color: ${T.hover};`,
    kind: `${TX.label} padding-bottom: 3px;`,
    context: `${TX.body} color: ${T.text2};`,
    iconBtn: `background-color: ${T.hover}; border-radius: ${T.r2}; padding: 7px; color: ${T.text2};`,
    iconBtnOk: `background-color: rgba(74,222,128,0.16); border-radius: ${T.r2}; padding: 7px; color: ${C.ok};`,
    iconBtnNo: `background-color: ${C.dangerBg}; border-radius: ${T.r2}; padding: 7px; color: ${C.danger};`,
    fixAll: UI.btnPrimary,
    fixAllHover: UI.btnPrimaryHover,
    close: `border-radius: ${T.r2}; padding: 6px; color: ${T.text2};`,
    closeHover: `border-radius: ${T.r2}; padding: 6px; color: ${T.text}; background-color: ${T.hover};`,
    footer: `${UI.footer} margin-top: 4px;`,
    key: UI.key,
    keyHint: `${TX.small} margin-right: 8px;`,
};

// Desfoca o que está atrás do painel (efeito de vidro do GNOME Shell).
function addGlass(actor) {
    try {
        actor.add_effect(new Shell.BlurEffect({radius: 36, brightness: 0.62, mode: Shell.BlurMode.BACKGROUND}));
    } catch (_e) {}
}

function iconButton(iconName, normal, hover, onClick) {
    const b = new St.Button({
        can_focus: false, track_hover: true, reactive: true,
        child: new St.Icon({icon_name: iconName, icon_size: 16}),
        y_align: Clutter.ActorAlign.CENTER,
    });
    b.style = normal;
    b.connect('notify::hover', () => {
        b.style = b.hover ? hover : normal;
    });
    b.connect('clicked', onClick);
    return b;
}

const esc = s => GLib.markup_escape_text(s, -1);

// ---- Texto --------------------------------------------------------------
// Todas as posições são em code points (o mesmo que o Shell usa no cursor).

function utf8Length(str) {
    return new TextEncoder().encode(str).length;
}

function byteToCharOffset(str, byteOffset) {
    let bytes = 0;
    let chars = 0;
    for (const ch of str) {
        if (bytes >= byteOffset)
            break;
        bytes += utf8Length(ch);
        chars++;
    }
    return chars;
}

// Palavras de uma linha com posição: [{t, s, e}].
function wordsWithPos(cps, from, to) {
    const out = [];
    let i = from;
    while (i < to) {
        while (i < to && /\s/.test(cps[i]))
            i++;
        if (i >= to)
            break;
        const s = i;
        while (i < to && !/\s/.test(cps[i]))
            i++;
        out.push({t: cps.slice(s, i).join(''), s, e: i});
    }
    return out;
}

// Linhas com posição: [{s, e}] (e = índice do "\n" ou fim).
function linesWithPos(cps) {
    const lines = [];
    let s = 0;
    for (let i = 0; i <= cps.length; i++) {
        if (i === cps.length || cps[i] === '\n') {
            lines.push({s, e: i});
            s = i + 1;
        }
    }
    return lines;
}

// LCS entre duas listas de strings → pares de operações.
function lcsOps(a, b) {
    const n = a.length, m = b.length;
    const dp = Array.from({length: n + 1}, () => new Array(m + 1).fill(0));
    for (let i = n - 1; i >= 0; i--) {
        for (let j = m - 1; j >= 0; j--)
            dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
    const ops = [];
    let i = 0, j = 0;
    while (i < n || j < m) {
        if (i < n && j < m && a[i] === b[j]) {
            ops.push({op: 'same', i: i++, j: j++});
        } else if (j < m && (i >= n || dp[i][j + 1] >= dp[i + 1][j])) {
            ops.push({op: 'ins', j: j++, i});
        } else {
            ops.push({op: 'del', i: i++});
        }
    }
    return ops;
}

// Sugestões palavra a palavra, linha a linha. Cada sugestão vira uma edição
// {s, e, rep} sobre o texto original, mais o que mostrar no painel.
function suggestions(original, corrected) {
    const cps = [...original];
    const oLines = linesWithPos(cps);
    let cLines = corrected.split('\n');
    if (cLines.length !== oLines.length) {
        // O modelo mexeu nas quebras: alinha só as linhas com conteúdo.
        const nonEmpty = oLines.filter(l => cps.slice(l.s, l.e).join('').trim());
        const cNon = cLines.filter(l => l.trim());
        if (nonEmpty.length !== cNon.length)
            return null;
        const map = new Map(nonEmpty.map((l, k) => [l, cNon[k]]));
        cLines = oLines.map(l => map.get(l) ?? cps.slice(l.s, l.e).join(''));
    }

    const out = [];
    oLines.forEach((line, li) => {
        const a = wordsWithPos(cps, line.s, line.e);
        const b = cLines[li].split(/\s+/).filter(Boolean);
        const ops = lcsOps(a.map(w => w.t), b);
        let k = 0;
        while (k < ops.length) {
            if (ops[k].op === 'same') {
                k++;
                continue;
            }
            const from = [], to = [];
            const firstI = ops[k].i;
            while (k < ops.length && ops[k].op !== 'same') {
                if (ops[k].op === 'del')
                    from.push(ops[k].i);
                else
                    to.push(b[ops[k].j]);
                k++;
            }
            // N palavras por N palavras viram N sugestões.
            if (from.length === to.length && from.length > 1) {
                from.forEach((wi, x) => out.push(makeSug(cps, a, [wi], [to[x]], wi, line)));
            } else {
                out.push(makeSug(cps, a, from, to, firstI, line));
            }
        }
    });
    return out;
}

function makeSug(cps, words, fromIdx, to, anchorIdx, line) {
    const fromWords = fromIdx.map(i => words[i]);
    const from = fromWords.map(w => w.t).join(' ');
    const rep = to.join(' ');
    let s, e, r;
    if (fromWords.length) {
        s = fromWords[0].s;
        e = fromWords.at(-1).e;
        r = rep;
        if (!rep) {
            // Remoção: leva junto o espaço anterior (ou o seguinte).
            const prev = words[fromIdx[0] - 1];
            const next = words[fromIdx.at(-1) + 1];
            if (prev)
                s = prev.e;
            else if (next)
                e = next.s;
        }
    } else {
        // Inserção depois da palavra anterior (ou no começo da linha).
        const prev = words[anchorIdx - 1];
        s = e = prev ? prev.e : line.s;
        r = prev ? ` ${rep}` : `${rep} `;
    }
    // Contexto: até 3 palavras de cada lado.
    const first = fromIdx.length ? fromIdx[0] : anchorIdx;
    const last = fromIdx.length ? fromIdx.at(-1) : anchorIdx - 1;
    const before = words.slice(Math.max(0, first - 3), first).map(w => w.t).join(' ');
    const after = words.slice(last + 1, last + 4).map(w => w.t).join(' ');
    return {
        s, e, rep: r, from, to: rep,
        key: `${from}→${rep}`,
        before: (first > 3 ? '…' : '') + before,
        after: after + (last + 4 < words.length ? '…' : ''),
    };
}

// Aplica edições (de trás para frente) sobre um array de code points.
function applyEdits(cps, edits) {
    const out = cps.slice();
    for (const ed of [...edits].sort((x, y) => y.s - x.s))
        out.splice(ed.s, ed.e - ed.s, ...ed.rep);
    return out.join('');
}

const flat = text => (text ?? '').split(/\s+/).filter(Boolean).join(' ');

const stripAccents = s => s.normalize('NFD').replace(/[̀-ͯ]/g, '');
const stripPunct = s => s.replace(/[.,;:!?…"'“”()\-]/g, '');

function kindOf(sug) {
    const a = sug.from, b = sug.to;
    if (!a)
        return 'ADIÇÃO';
    if (!b)
        return 'REMOÇÃO';
    const pa = stripPunct(a), pb = stripPunct(b);
    if (pa === pb)
        return 'PONTUAÇÃO';
    if (pa.toLowerCase() === pb.toLowerCase())
        return 'MAIÚSCULA';
    if (stripAccents(pa).toLowerCase() === stripAccents(pb).toLowerCase())
        return 'ACENTUAÇÃO';
    return 'ORTOGRAFIA';
}

// ---- Trechos ------------------------------------------------------------
// O texto é dividido em trechos estáveis (até CHUNK_WORDS palavras, ou até
// um ponto final). Cada trecho é corrigido uma vez e fica em cache: enquanto
// você digita no fim, só o último trecho é verificado de novo, e os trechos
// que já ficaram para trás são verificados sem esperar a pausa.

function chunksOf(text) {
    const cps = [...text];
    return linesWithPos(cps).map(line => {
        const words = wordsWithPos(cps, line.s, line.e);
        const chunks = [];
        let group = [];
        const close = () => {
            if (group.length) {
                const s = group[0].s, e = group.at(-1).e;
                const text = cps.slice(s, e).join('');
                // Contexto: últimas palavras do trecho anterior da mesma linha.
                const prev = chunks.at(-1);
                const ctx = prev ? prev.words.slice(-CONTEXT_WORDS).join(' ') : '';
                chunks.push({s, e, text, ctx, key: ctx ? `${ctx}\u0001${text}` : text, words: group.map(w => w.t)});
            }
            group = [];
        };
        for (const w of words) {
            group.push(w);
            if (group.length >= CHUNK_WORDS || (group.length >= 4 && /[.!?…]$/.test(w.t)))
                close();
        }
        close();
        return {...line, chunks};
    });
}

// O modelo corrige "contexto + trecho"; aqui fica só a parte do trecho,
// alinhando as palavras da saída com as da entrada.
function takeChunkPart(ctx, text, out) {
    if (!ctx)
        return out;
    const inWords = `${ctx} ${text}`.split(/\s+/).filter(Boolean);
    const outWords = out.split(/\s+/).filter(Boolean);
    const b = ctx.split(/\s+/).filter(Boolean).length;
    const kept = [];
    for (const op of lcsOps(inWords, outWords)) {
        if (op.op === 'del')
            continue;
        // Substituições aparecem como 'ins' antes do 'del' da mesma posição.
        if (op.i >= b)
            kept.push(outWords[op.j]);
    }
    return kept.length ? kept.join(' ') : text;
}

// Junta as correções dos trechos (ou o original, se ainda não verificado),
// ajustando as emendas: o modelo trata cada trecho como frase própria.
function assemble(lines, cache) {
    return lines.map(line => {
        const parts = [];
        line.chunks.forEach((ch, i) => {
            let out = cache.get(ch.key);
            const checked = out !== undefined;
            out = checked ? out : ch.text;
            if (checked && i > 0 && !/[.!?…]$/.test(parts[i - 1]) && /^[a-zà-ÿ]/.test(ch.text))
                out = out.charAt(0).toLowerCase() + out.slice(1);
            const next = line.chunks[i + 1];
            // Num corte no meio da frase o modelo costuma fechar com . ? ou ...
            if (checked && next && !/[.!?…,;:]$/.test(ch.text) && /^[a-zà-ÿ]/.test(next.text))
                out = out.replace(/\s*(\.\.\.|…|[.?!])$/, '');
            parts.push(out);
        });
        return parts.join(' ');
    }).join('\n');
}

const sleep = ms => new Promise(r => GLib.timeout_add(GLib.PRIORITY_DEFAULT, ms, () => {
    r();
    return GLib.SOURCE_REMOVE;
}));

// ---- Extensão -----------------------------------------------------------

// ---- Assistente de resposta (Alt+K) -------------------------------------
// Alt+K tira um print só da janela em foco, lê a conversa (OCR, local),
// e o modelo local sugere a sua próxima mensagem. No painel dá para inserir
// no campo, pedir outra, copiar, conversar para ajustar ("mais curta") e
// trocar o contexto (arquivos em ~/.config/clippit/contextos/*.md, com uma
// primeira linha opcional "apps: whatsapp, slack" para escolher sozinho).

const CONTEXT_DIR = GLib.build_filenamev([GLib.get_home_dir(), '.config', 'clippit', 'contextos']);
const ASSIST_WIDTH = 580;

const ASTYLE = {
    title: TX.bodyStrong,
    ctxBtn: `${UI.key} color: ${C.accent}; background-color: ${T.accentSoft}; margin-left: 6px;`,
    ctxBtnHover: `${UI.key} color: #ffffff; background-color: rgba(76,157,255,0.32); margin-left: 6px;`,
    body: `padding: 0 ${T.pad}px; spacing: 8px;`,
    reply: `${TX.body} background-color: ${T.surface}; border-radius: ${T.r3}; padding: 12px 14px; min-height: 60px;`,
    ctxList: `margin: 0 ${T.pad}px 10px ${T.pad}px; padding: 4px; spacing: 2px; background-color: ${T.surface};
        border-radius: ${T.r3};`,
    ctxItem: `${TX.body} border-radius: ${T.r2}; padding: 6px 10px;`,
    ctxItemHover: `${TX.body} border-radius: ${T.r2}; padding: 6px 10px; background-color: ${T.hover};`,
    ctxItemOn: `${TX.bodyStrong} color: ${C.accent}; border-radius: ${T.r2}; padding: 6px 10px; background-color: ${T.accentSoft};`,
    opt: `${UI.card} spacing: 10px;`,
    optOn: `${UI.cardOn} spacing: 10px;`,
    optName: TX.label,
    optText: `${TX.body} background-color: transparent; border: none; padding: 0; caret-color: ${C.accent}; selected-color: #ffffff;`,
    srcWrap: `padding: 10px ${T.pad}px 0 ${T.pad}px; spacing: 6px;`,
    srcToggle: `${TX.small} padding: 2px 0;`,
    srcToggleHover: `${TX.small} color: ${T.text}; padding: 2px 0;`,
    srcBox: `${UI.code} spacing: 8px;`,
    srcSummary: `${TX.small} color: ${T.text};`,
    srcText: TX.mono,
    ctxRow: `border-radius: ${T.r2}; padding: 6px 6px 6px 10px; spacing: 8px;`,
    ctxRowHover: `border-radius: ${T.r2}; padding: 6px 6px 6px 10px; spacing: 8px; background-color: ${T.hover};`,
    ctxRowOn: `border-radius: ${T.r2}; padding: 6px 6px 6px 10px; spacing: 8px; background-color: ${T.accentSoft};`,
    ctxName: TX.bodyStrong,
    ctxNameOn: `${TX.bodyStrong} color: ${C.accent};`,
    ctxApps: TX.tiny,
    ctxFoot: `spacing: 6px; padding: 8px 2px 2px 2px;`,
    edBox: `padding: 0 ${T.pad}px; spacing: 6px;`,
    edLabel: `${TX.label} padding-top: 4px;`,
    edArea: `${TX.body} background-color: ${T.surface}; border-radius: ${T.r3}; padding: 8px 10px; caret-color: ${C.accent};
        min-height: 180px;`,
    edHint: TX.tiny,
    status: TX.small,
    ask: `${TX.small} font-style: italic;`,
    actions: `spacing: 6px; padding: 10px ${T.pad}px 0 ${T.pad}px;`,
    btn: UI.btn,
    btnHover: UI.btnHover,
    entry: `${TX.body} background-color: ${T.surface}; border-radius: ${T.r3}; padding: 7px 10px; caret-color: ${C.accent};
        selected-color: #ffffff;`,
    entryWrap: `padding: 10px ${T.pad}px 0 ${T.pad}px;`,
};

function textButton(label, normal, hover, onClick) {
    const b = new St.Button({label, can_focus: false, track_hover: true, reactive: true});
    b.style = normal;
    b.connect('notify::hover', () => {
        b.style = b.hover ? hover : normal;
    });
    b.connect('clicked', onClick);
    return b;
}

// Anel de carregando (o mesmo desenho do selo).
function makeRing(size = 14) {
    const ring = new St.DrawingArea({width: size, height: size, y_align: Clutter.ActorAlign.CENTER});
    ring.connect('repaint', area => {
        const cr = area.get_context();
        const [w, h] = area.get_surface_size();
        const r = Math.min(w, h) / 2 - 1.5;
        cr.setLineWidth(2);
        cr.setSourceRGBA(1, 1, 1, 0.12);
        cr.arc(w / 2, h / 2, r, 0, 2 * Math.PI);
        cr.stroke();
        cr.setSourceRGBA(0.298, 0.616, 1, 1);
        cr.arc(w / 2, h / 2, r, -Math.PI / 2, Math.PI / 4);
        cr.stroke();
        cr.$dispose();
    });
    ring.set_pivot_point(0.5, 0.5);
    return {
        actor: ring,
        start() {
            // Só gira depois de estar na tela (antes disso o GNOME reclama).
            const go = () => {
                if (ring.get_transition('spin'))
                    return;
                const spin = new Clutter.PropertyTransition({property_name: 'rotation-angle-z', duration: 800, repeat_count: -1});
                spin.set_from(0);
                spin.set_to(360);
                ring.add_transition('spin', spin);
            };
            if (ring.mapped) {
                go();
            } else {
                const id = ring.connect('notify::mapped', () => {
                    if (ring.mapped) {
                        ring.disconnect(id);
                        go();
                    }
                });
            }
        },
        stop() {
            ring.remove_transition('spin');
        },
    };
}

function runProcess(argv, input = null, timeoutS = 120) {
    return new Promise((resolve, reject) => {
        let proc;
        try {
            proc = Gio.Subprocess.new(argv,
                (input !== null ? Gio.SubprocessFlags.STDIN_PIPE : Gio.SubprocessFlags.NONE) |
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE);
        } catch (e) {
            reject(e);
            return;
        }
        const timeoutId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, timeoutS, () => {
            proc.force_exit();
            return GLib.SOURCE_REMOVE;
        });
        proc.communicate_utf8_async(input, null, (p, res) => {
            GLib.source_remove(timeoutId);
            try {
                const [, stdout, stderr] = p.communicate_utf8_finish(res);
                if (!p.get_successful())
                    throw new Error((stderr ?? '').trim().slice(0, 200) || 'falhou');
                resolve(stdout ?? '');
            } catch (e) {
                reject(e);
            }
        });
    });
}

// Aprendizado: roda em segundo plano e nunca atrapalha se falhar.
function learn(args, input = null) {
    runProcess([CORRECTOR_CMD, ...args], input, 20).catch(e =>
        console.log(`${TAG} aprendizado falhou: ${e.message}`));
}

function loadContexts() {
    const out = [];
    const dir = Gio.File.new_for_path(CONTEXT_DIR);
    try {
        const en = dir.enumerate_children('standard::name', Gio.FileQueryInfoFlags.NONE, null);
        let info;
        while ((info = en.next_file(null))) {
            const name = info.get_name();
            if (!name.endsWith('.md'))
                continue;
            const file = GLib.build_filenamev([CONTEXT_DIR, name]);
            const [, bytes] = GLib.file_get_contents(file);
            let text = new TextDecoder().decode(bytes);
            let apps = [];
            const first = text.split('\n')[0];
            if (/^apps\s*:/i.test(first)) {
                apps = first.replace(/^apps\s*:/i, '').split(',').map(a => a.trim().toLowerCase()).filter(Boolean);
                text = text.split('\n').slice(1).join('\n');
            }
            const raw = text.replace(/^\n+/, '');
            // Linhas de modelo ainda não preenchidas não vão para a IA.
            text = text.split('\n')
                .filter(l => !/^\s*-\s*[^:]+:\s*$/.test(l) && !/^Preencha abaixo/i.test(l))
                .join('\n').trim();
            out.push({name: name.replace(/\.md$/, ''), apps, text, raw, file});
        }
    } catch (e) {
        console.log(`${TAG} sem pasta de contextos: ${e.message}`);
    }
    out.sort((a, b) => a.name.localeCompare(b.name));
    out.push({name: 'Sem contexto', apps: [], text: ''});
    return out;
}

class Assistente {
    constructor(corretor) {
        this._c = corretor;
        this._grabs = new Map();
        this._ctxByApp = new Map();
    }

    enable() {
        // Alt+K (assistente) e Ctrl+Shift+E (modo jogo: tira o modelo da placa de vídeo).
        bindShortcuts(this, [['assistente', () => (this._panel ? this._regenerate(0.8) : this._safeOpen())],
            ['modo_jogo', () => this._toggleGameMode()]]);
        // Autoteste (só para depurar): monta o painel uma vez ao carregar.
        const probe = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest']);
        if (GLib.file_test(probe, GLib.FileTest.EXISTS)) {
            const shotDir = new TextDecoder().decode(GLib.file_get_contents(probe)[1]).trim();
            GLib.unlink(probe);
            try {
                this._win = global.display.focus_window;
                this._app = 'selftest';
                this._contexts = loadContexts();
                this._ctxIndex = this._pickContext();
                this._history = [];
                this._buildPanel();
                this._setStatus('teste', true);
                this._reply = 'teste';
                this._options = ['curta', 'teste', 'completa'];
                this._modelOptions = [...this._options];
                this._sel = 1;
                this._conversation = '[Você]\noi';
                this._showReply();
                console.log(`${TAG} autoteste do painel: ok`);
            } catch (e) {
                console.log(`${TAG} autoteste do painel falhou: ${e.message}\n${e.stack}`);
            }
            if (shotDir) {
                GLib.timeout_add(GLib.PRIORITY_DEFAULT, 800, () => {
                    shootActor(this._panel, `${shotDir}/assistente.png`).then(() => this._close(false));
                    return GLib.SOURCE_REMOVE;
                });
            } else {
                this._close(false);
            }
        }
    }

    disable() {
        this._close(false);
        unbindShortcuts(this);
        for (const accel of [...this._grabs.keys()])
            this._setKey(accel, false);
        if (this._accelId) {
            global.display.disconnect(this._accelId);
            this._accelId = 0;
        }
    }

    _setKey(accel, want, fn) {
        const grab = this._grabs.get(accel);
        if (want && !grab) {
            const action = global.display.grab_accelerator(accel, Meta.KeyBindingFlags.NONE);
            if (action === Meta.KeyBindingAction.NONE) {
                console.log(`${TAG} não consegui reservar ${accel}`);
                return;
            }
            const name = Meta.external_binding_name_for_action(action);
            Main.wm.allowKeybinding(name, Shell.ActionMode.ALL);
            this._grabs.set(accel, {action, name, fn});
        } else if (!want && grab) {
            global.display.ungrab_accelerator(grab.action);
            Main.wm.allowKeybinding(grab.name, Shell.ActionMode.NONE);
            this._grabs.delete(accel);
        }
        if (!this._accelId && this._grabs.size) {
            this._accelId = global.display.connect('accelerator-activated', (_d, action) => {
                for (const g of this._grabs.values()) {
                    if (g.action === action)
                        g.fn();
                }
            });
        }
    }

    // ---- Abrir: print → OCR → resposta ----------------------------------

    async _toggleGameMode() {
        if (this._gameBusy)
            return;
        this._gameBusy = true;
        try {
            const out = JSON.parse(await runProcess([CORRECTOR_CMD, '--modo-jogo'], null, 30));
            const gb = (out.liberado_mb / 1024).toLocaleString('pt-BR', {maximumFractionDigits: 1});
            if (out.modo_jogo) {
                Main.notify('Clippit — modo jogo ligado',
                    `${out.liberado_mb > 100 ? `${gb} GB liberados na placa de vídeo. ` : 'A placa de vídeo está livre. '}` +
                    'O corretor continua funcionando no processador. Ctrl+Shift+E para voltar ao normal.');
            } else {
                Main.notify('Clippit — modo jogo desligado',
                    'O modelo volta a usar a placa de vídeo na próxima correção, se ela estiver livre.');
            }
            console.log(`${TAG} modo jogo ${out.modo_jogo ? 'ligado' : 'desligado'} (${out.liberado_mb} MiB liberados)`);
        } catch (e) {
            Main.notify('Clippit', `Não consegui trocar o modo jogo: ${e.message}`);
        } finally {
            this._gameBusy = false;
        }
    }

    async _safeOpen() {
        if (!aiOn('assistente')) {
            aiOffNotice('O assistente de resposta');
            return;
        }
        try {
            await this.open();
        } catch (e) {
            console.log(`${TAG} assistente falhou ao abrir: ${e.message}\n${e.stack}`);
            this._close(false);
        }
    }

    async open() {
        const win = global.display.focus_window;
        if (!win)
            return;
        this._close(false);
        this._win = win;
        this._app = (win.get_wm_class() ?? '').toLowerCase();
        const [draft] = Main.inputMethod.currentFocus ? Main.inputMethod.getSurroundingText() : [''];
        this._draft = (draft ?? '').trim();
        this._history = [];
        this._conversation = '';
        this._contexts = loadContexts();
        this._ctxIndex = this._pickContext();
        this._rect = Main.inputMethod._cursorRect ?? null;
        const gen = this._gen = (this._gen ?? 0) + 1;

        this._c._dismissPanel?.();
        this._buildPanel();
        this._setStatus('Lendo a conversa…', true);

        const dir = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor']);
        GLib.mkdir_with_parents(dir, 0o700);
        const path = GLib.build_filenamev([dir, `tela-${Date.now()}.png`]);
        try {
            await this._shoot(path);
            this._conversation = await runProcess([CORRECTOR_CMD, '--ocr', path], null, 60);
        } catch (e) {
            console.log(`${TAG} não consegui ler a tela: ${e.message}`);
            if (gen === this._gen)
                this._setStatus('Não consegui ler a tela desta janela.', false);
            return;
        } finally {
            // O print fica só enquanto o painel está aberto (o modelo olha a
            // imagem a cada resposta) e é apagado ao fechar.
            this._shot = path;
        }
        if (gen !== this._gen)
            return;
        console.log(`${TAG} assistente: leu ${this._conversation.length} chars app=${this._app}`);
        if (!this._conversation.trim()) {
            this._setStatus('Não achei texto nesta janela.', false);
            return;
        }
        await this._generate(0.4);
    }

    // Lê só a parte da tela que importa: da coluna do campo de texto para a
    // direita e do topo da janela até logo abaixo do campo. Em páginas cheias
    // (sistemas, e-mail, tarefas) isso tira menus e formulários do caminho.
    _pickArea() {
        const f = this._win.get_frame_rect();
        const r = this._rect;
        const full = {x: f.x, y: f.y, width: f.width, height: f.height, where: 'a janela inteira'};
        if (!r || (r.x === 0 && r.y === 0) || r.x < f.x || r.x > f.x + f.width || r.y < f.y || r.y > f.y + f.height)
            return full;
        let x = f.x;
        let where = 'a janela até o campo de texto';
        if (r.x - f.x > 0.2 * f.width) {
            x = Math.max(f.x, Math.round(r.x - 80));
            where = 'a coluna do campo de texto (o que está acima dele)';
        }
        const bottom = Math.min(f.y + f.height, Math.round(r.y + r.height + 30));
        const area = {x, y: f.y, width: f.x + f.width - x, height: bottom - f.y, where};
        return area.height < 150 || area.width < 200 ? full : area;
    }

    _shoot(path) {
        this._area = this._pickArea();
        const a = this._area;
        return new Promise((resolve, reject) => {
            const shooter = new Shell.Screenshot();
            const stream = Gio.File.new_for_path(path).replace(null, false, Gio.FileCreateFlags.NONE, null);
            shooter.screenshot_area(a.x, a.y, a.width, a.height, stream, (o, res) => {
                try {
                    o.screenshot_area_finish(res);
                    stream.close(null);
                    resolve();
                } catch (e) {
                    reject(e);
                }
            });
        });
    }

    _pickContext() {
        const remembered = this._ctxByApp.get(this._app);
        if (remembered !== undefined) {
            const i = this._contexts.findIndex(c => c.name === remembered);
            if (i >= 0)
                return i;
        }
        const i = this._contexts.findIndex(c => c.apps.some(a => this._app.includes(a)));
        if (i >= 0)
            return i;
        // Nenhum contexto é deste app: "Geral" (ou "Sem contexto"), nunca o 1º da lista.
        const geral = this._contexts.findIndex(c => c.name.toLowerCase() === 'geral');
        return geral >= 0 ? geral : this._contexts.length - 1;
    }

    async _generate(temperature) {
        const gen = this._gen;
        this._setStatus(this._history.length ? 'Ajustando…' : 'Escrevendo a resposta…', true);
        const ctx = this._contexts[this._ctxIndex];
        const payload = JSON.stringify({
            conversation: this._conversation,
            draft: this._draft,
            context: ctx?.text ?? '',
            context_name: ctx?.name ?? '',
            image: this._shot ?? '',
            history: this._history,
            temperature,
            variants: this._history.length ? 0 : 3, // ajustes refazem só a escolhida
        });
        try {
            const raw = (await runProcess([CORRECTOR_CMD, '--assist'], payload, 150)).trim();
            if (gen !== this._gen || !this._panel)
                return;
            let options = [raw];
            if (raw.startsWith('{')) {
                try {
                    options = JSON.parse(raw).opcoes.filter(Boolean);
                } catch (_e) {}
            }
            this._options = options;
            this._modelOptions = [...options]; // o que a IA sugeriu, antes de você editar
            // Com três, começa na do meio (a normal); com uma, nela.
            this._sel = options.length >= 3 ? 1 : 0;
            this._reply = options[this._sel];
            this._modelReply = this._reply;
            this._showReply();
        } catch (e) {
            console.log(`${TAG} assistente falhou: ${e.message}`);
            if (gen === this._gen)
                this._setStatus('O modelo não respondeu. Tente de novo (Alt+K).', false);
        }
    }

    _regenerate(temperature) {
        if (!this._conversation || this._busy)
            return;
        this._generate(temperature);
    }

    // ---- Painel -----------------------------------------------------------

    _buildPanel() {
        const panel = new St.BoxLayout({vertical: true, reactive: true, style: STYLE.panel});
        addGlass(panel);
        panel.set_width(ASSIST_WIDTH);

        const header = new St.BoxLayout({style: STYLE.header});
        header.add_child(new St.Label({text: 'Resposta sugerida', style: ASTYLE.title, y_align: Clutter.ActorAlign.CENTER}));
        this._ctxBtn = textButton('', ASTYLE.ctxBtn, ASTYLE.ctxBtnHover, () => this._toggleCtxList());
        this._ctxBtn.y_align = Clutter.ActorAlign.CENTER;
        header.add_child(this._ctxBtn);
        this._updateCtxLabel();
        header.add_child(new St.Widget({x_expand: true}));
        header.add_child(iconButton('folder-open-symbolic', STYLE.close, STYLE.closeHover, () => this._openContextDir()));
        header.add_child(iconButton('window-close-symbolic', STYLE.close, STYLE.closeHover, () => this._close()));
        panel.add_child(header);

        // Lista para escolher o contexto (abre no botão azul).
        this._ctxList = new St.BoxLayout({vertical: true, style: ASTYLE.ctxList, visible: false});
        panel.add_child(this._ctxList);

        const body = new St.BoxLayout({vertical: true, style: ASTYLE.body});
        const statusRow = new St.BoxLayout({style: 'spacing: 8px;'});
        this._ring = makeRing(14);
        statusRow.add_child(this._ring.actor);
        this._status = new St.Label({style: ASTYLE.status, y_align: Clutter.ActorAlign.CENTER});
        statusRow.add_child(this._status);
        this._statusRow = statusRow;
        body.add_child(statusRow);

        // As opções de resposta (curta, normal, completa). Cada uma é editável:
        // clique no texto para escolher e mudar o que quiser.
        this._optBox = new St.BoxLayout({vertical: true, style: 'spacing: 6px;', visible: false});
        body.add_child(this._optBox);

        this._editHint = new St.Label({text: 'Escolha com 1, 2 ou 3. Clique no texto para editar; Esc sai da edição.', style: ASTYLE.ask, visible: false});
        body.add_child(this._editHint);

        this._askLabel = new St.Label({style: ASTYLE.ask, visible: false});
        body.add_child(this._askLabel);
        panel.add_child(body);
        this._body = body;

        const actions = new St.BoxLayout({style: ASTYLE.actions});
        this._insertBtn = textButton('Inserir', STYLE.fixAll, STYLE.fixAllHover, () => this._insert());
        actions.add_child(this._insertBtn);
        actions.add_child(textButton('Outra', ASTYLE.btn, ASTYLE.btnHover, () => this._regenerate(0.8)));
        actions.add_child(textButton('Copiar', ASTYLE.btn, ASTYLE.btnHover, () => this._copy()));
        this._actions = actions;
        panel.add_child(actions);

        // "De onde veio": o que foi lido da tela, quem falou e qual contexto.
        const srcWrap = new St.BoxLayout({vertical: true, style: ASTYLE.srcWrap});
        this._srcToggle = textButton('▸  De onde veio esta resposta', ASTYLE.srcToggle, ASTYLE.srcToggleHover,
            () => this._toggleSources());
        this._srcToggle.x_align = Clutter.ActorAlign.START;
        srcWrap.add_child(this._srcToggle);
        this._srcBox = new St.BoxLayout({vertical: true, style: ASTYLE.srcBox, visible: false});
        this._srcSummary = new St.Label({style: ASTYLE.srcSummary});
        this._srcSummary.clutter_text.line_wrap = true;
        this._srcSummary.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        this._srcBox.add_child(this._srcSummary);
        this._srcText = new St.Label({style: ASTYLE.srcText});
        this._srcText.clutter_text.line_wrap = true;
        this._srcText.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        const srcScroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: false, style: 'max-height: 220px;',
        });
        // A rolagem do GNOME só aceita uma caixa dentro, não um texto direto.
        const srcInner = new St.BoxLayout({vertical: true});
        srcInner.add_child(this._srcText);
        if (srcScroll.set_child)
            srcScroll.set_child(srcInner);
        else
            srcScroll.add_actor(srcInner);
        this._srcBox.add_child(srcScroll);
        srcWrap.add_child(this._srcBox);
        panel.add_child(srcWrap);

        const wrap = new St.BoxLayout({style: ASTYLE.entryWrap});
        this._entry = new St.Entry({
            style: ASTYLE.entry, x_expand: true, can_focus: true,
            hint_text: 'Ajuste ("mais curta"), regra ("regra: …") ou anotação sobre a pessoa ("nota: …")',
        });
        this._entry.clutter_text.connect('activate', () => this._sendAsk());
        this._entry.clutter_text.connect('key-press-event', (_a, ev) => {
            if (ev.get_key_symbol() === Clutter.KEY_Escape) {
                this._close();
                return Clutter.EVENT_STOP;
            }
            // Alt+F de novo: solta o teclado e volta para o campo do app.
            const sym = ev.get_key_symbol();
            if ((sym === Clutter.KEY_f || sym === Clutter.KEY_F) &&
                (ev.get_state() & Clutter.ModifierType.MOD1_MASK)) {
                this._releaseKeyboard();
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        });
        // Para digitar no campo o painel precisa do teclado; só pega ao clicar nele.
        this._entry.connect('button-press-event', () => {
            this._takeKeyboard();
            return Clutter.EVENT_PROPAGATE;
        });
        wrap.add_child(this._entry);
        panel.add_child(wrap);
        this._entryWrap = wrap;

        // Rodapé padrão (o mesmo do lançador).
        const foot = makeFooter();
        foot.set('Assistente', 'starred-symbolic', [{label: 'Inserir', key: '↵'}, {label: 'Ajustar', key: 'Alt F'},
            {label: 'Outra', key: 'Alt K'}, {label: 'Fechar', key: 'Esc'}]);
        const footer = foot.actor;
        panel.add_child(footer);
        this._footer = footer;

        // Editor de contexto (fica escondido até "ver/editar" ou "novo").
        this._editor = new St.BoxLayout({vertical: true, style: ASTYLE.edBox, visible: false});
        panel.add_child(this._editor);

        this._panel = panel;
        Main.layoutManager.addTopChrome(panel);
        this._setKey('Escape', true, () => this._close());
        this._ajusteKey = keyOf('ajuste');
        if (this._ajusteKey !== 'desligado')
            this._setKey(this._ajusteKey, true, () => this._toggleKeyboard());
        this._position();
    }

    _position() {
        if (!this._panel)
            return;
        const frame = this._win?.get_frame_rect();
        const r = this._rect && !(this._rect.x === 0 && this._rect.y === 0)
            ? this._rect
            : frame ? {x: frame.x + frame.width / 2 - ASSIST_WIDTH / 2, y: frame.y + frame.height - 80, width: 0, height: 0}
                : {x: 100, y: 100, width: 0, height: 0};
        const monitor = Main.layoutManager.monitors.find(m =>
            r.x >= m.x && r.x < m.x + m.width && r.y >= m.y && r.y < m.y + m.height) ?? Main.layoutManager.primaryMonitor;
        const [, h] = this._panel.get_preferred_height(ASSIST_WIDTH);
        const x = Math.min(Math.max(r.x - 20, monitor.x + 12), monitor.x + monitor.width - ASSIST_WIDTH - 12);
        let y = r.y - h - 12;
        if (y < monitor.y + 12)
            y = Math.min(r.y + r.height + 12, monitor.y + monitor.height - h - 12);
        this._panel.set_position(Math.round(x), Math.round(y));
    }

    // Enter insere a resposta enquanto o painel está aberto com uma resposta
    // pronta (e você não está digitando um ajuste). Fora disso, Enter é do app.
    _syncEnter() {
        const want = Boolean(this._panel && this._reply && !this._busy && !this._grab);
        this._setKey('Return', want, () => this._insert());
        this._setKey('KP_Enter', want, () => this._insert());
        // 1, 2, 3 escolhem a opção (só com mais de uma e fora da edição).
        const many = want && (this._options?.length ?? 0) > 1;
        for (let i = 0; i < 3; i++)
            this._setKey(String(i + 1), many && i < this._options.length, () => this._select(i));
    }

    _select(i) {
        if (!this._options?.[i])
            return;
        this._sel = i;
        this._reply = this._options[i];
        this._modelReply = this._modelOptions?.[i] ?? this._reply;
        this._paintOptions();
    }

    _paintOptions() {
        (this._optRows ?? []).forEach((row, i) => {
            row.style = i === this._sel ? ASTYLE.optOn : ASTYLE.opt;
        });
    }

    _renderOptions() {
        this._optBox.destroy_all_children();
        this._optRows = [];
        const names = this._options.length >= 3 ? ['Curta', 'Normal', 'Completa'] : [];
        this._options.forEach((text, i) => {
            const row = new St.BoxLayout({style: ASTYLE.opt, reactive: true});
            const chipBox = new St.BoxLayout({vertical: true, style: 'spacing: 2px; padding-top: 2px;'});
            if (this._options.length > 1)
                chipBox.add_child(new St.Label({text: String(i + 1), style: STYLE.key, x_align: Clutter.ActorAlign.CENTER}));
            if (names[i])
                chipBox.add_child(new St.Label({text: names[i], style: ASTYLE.optName}));
            row.add_child(chipBox);
            const entry = new St.Entry({style: ASTYLE.optText, text, can_focus: true, x_expand: true});
            const ct = entry.clutter_text;
            ct.single_line_mode = false;
            ct.activatable = false;
            ct.line_wrap = true;
            ct.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
            ct.connect('text-changed', () => {
                this._options[i] = entry.text;
                if (i === this._sel)
                    this._reply = entry.text;
            });
            ct.connect('key-press-event', (_a, ev) => {
                if (ev.get_key_symbol() === Clutter.KEY_Escape) {
                    this._releaseKeyboard();
                    return Clutter.EVENT_STOP;
                }
                return Clutter.EVENT_PROPAGATE;
            });
            entry.connect('button-press-event', () => {
                this._select(i);
                this._takeKeyboard(entry);
                return Clutter.EVENT_PROPAGATE;
            });
            row.connect('button-press-event', () => {
                this._select(i);
                return Clutter.EVENT_PROPAGATE;
            });
            row.add_child(entry);
            this._optBox.add_child(row);
            this._optRows.push(row);
        });
        this._paintOptions();
    }

    _setStatus(text, busy) {
        if (!this._panel)
            return;
        this._busy = busy;
        this._status.text = text;
        this._statusRow.show();
        this._optBox.visible = Boolean(this._reply);
        this._optBox.opacity = busy ? 110 : 255;
        this._actions.opacity = busy ? 120 : 255;
        this._ring.actor.visible = busy;
        if (busy)
            this._ring.start();
        else
            this._ring.stop();
        this._position();
        this._syncEnter();
    }

    _showReply() {
        this._busy = false;
        this._ring.stop();
        this._statusRow.hide();
        this._renderOptions();
        this._optBox.opacity = 255;
        this._optBox.show();
        this._editHint.show();
        this._updateSources();
        this._actions.opacity = 255;
        const last = this._history.at(-1);
        const note = this._ruleNote ?? (last?.ask ? `Ajuste pedido: ${last.ask}` : '');
        this._ruleNote = null;
        this._askLabel.visible = Boolean(note);
        this._askLabel.text = note;
        this._position();
        this._syncEnter();
    }

    _mainPerson() {
        const tags = [...(this._conversation ?? '').matchAll(/^\[([^\]\n]{1,40})\]$/gm)]
            .map(m => m[1]).filter(t => t !== 'Você' && t !== 'Outro');
        return tags.at(-1) ?? '';
    }

    // O que já foi anotado sobre a pessoa (aprendizado/pessoas.json).
    _personNote() {
        const who = this._mainPerson();
        if (!who)
            return null;
        try {
            const file = GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'aprendizado', 'pessoas.json']);
            const data = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(file)[1]));
            const notes = data[who.toLowerCase()]?.notas ?? [];
            return notes.length
                ? `Sobre ${who}: ${notes.slice(-3).join('; ')}${notes.length > 3 ? ` (+${notes.length - 3})` : ''}.`
                : `Sobre ${who}: nada anotado ainda (escreva "nota: …" no campo de ajuste).`;
        } catch (_e) {
            return `Sobre ${who}: nada anotado ainda (escreva "nota: …" no campo de ajuste).`;
        }
    }

    // A IA também olhou a imagem? (só com a NVIDIA livre e o arquivo de visão.)
    _visionNote() {
        try {
            const ready = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-correct', 'ready']);
            const txt = new TextDecoder().decode(GLib.file_get_contents(ready)[1]);
            return txt.includes('visão')
                ? 'Imagem: a IA também olhou o print, além do texto lido.'
                : 'Imagem: a IA só recebeu o texto lido (a visão roda só na placa de vídeo livre).';
        } catch (_e) {
            return null;
        }
    }

    _toggleSources() {
        const show = !this._srcBox.visible;
        this._srcBox.visible = show;
        this._srcToggle.label = `${show ? '▾' : '▸'}  De onde veio esta resposta`;
        this._updateSources();
        this._position();
    }

    _updateSources() {
        if (!this._srcSummary)
            return;
        const conv = this._conversation ?? '';
        const tags = [...conv.matchAll(/^\[([^\]\n]{1,40})\]$/gm)].map(m => m[1]);
        const people = [...new Set(tags)].map(t => (t === 'Você' ? 'você' : t === 'Outro' ? 'outra pessoa' : t));
        const ctx = this._contexts?.[this._ctxIndex];
        const lines = [
            `Tela: ${this._win?.get_title() ?? this._app} — li ${this._area?.where ?? 'a janela'}.`,
            this._visionNote(),
            this._personNote(),
            tags.length
                ? `Conversa: ${tags.length} ${tags.length === 1 ? 'bloco' : 'blocos'} de mensagens, de ${people.join(', ')}.`
                : 'Conversa: não reconheci quem falou o quê; a IA recebeu o texto da tela como está.',
            `Contexto: ${ctx && ctx.text ? `"${ctx.name}" (${ctx.text.split('\n').length} linhas de instruções)` : 'nenhum'}.`,
            this._draft ? `Seu rascunho no campo: "${this._draft.slice(0, 80)}${this._draft.length > 80 ? '…' : ''}"` : null,
            'Abaixo, o texto exatamente como a IA recebeu (lido da tela por OCR):',
        ].filter(Boolean);
        this._srcSummary.text = lines.join('\n');
        this._srcText.text = conv.trim() || '(nada)';
    }

    _updateCtxLabel() {
        const ctx = this._contexts?.[this._ctxIndex];
        this._ctxBtn.label = `${ctx?.name ?? 'Sem contexto'} ▾`;
    }

    _toggleCtxList() {
        if (this._ctxList.visible) {
            this._ctxList.hide();
            this._position();
            return;
        }
        this._ctxList.destroy_all_children();
        this._contexts.forEach((ctx, i) => {
            const on = i === this._ctxIndex;
            const row = new St.BoxLayout({reactive: true, track_hover: true, style: on ? ASTYLE.ctxRowOn : ASTYLE.ctxRow});
            row.connect('notify::hover', () => {
                row.style = on ? ASTYLE.ctxRowOn : row.hover ? ASTYLE.ctxRowHover : ASTYLE.ctxRow;
            });
            row.connect('button-release-event', () => {
                this._chooseContext(i);
                return Clutter.EVENT_STOP;
            });
            const texts = new St.BoxLayout({vertical: true, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            texts.add_child(new St.Label({text: `${on ? '✓ ' : ''}${ctx.name}`, style: on ? ASTYLE.ctxNameOn : ASTYLE.ctxName}));
            const sub = ctx.file
                ? (ctx.apps.length ? `Escolhido sozinho em: ${ctx.apps.join(', ')}` : 'Escolha manual')
                : 'Responde sem instruções extras';
            texts.add_child(new St.Label({text: sub, style: ASTYLE.ctxApps}));
            row.add_child(texts);
            if (ctx.file) {
                const edit = iconButton('document-edit-symbolic', STYLE.iconBtn, STYLE.closeHover, () => this._openEditor(ctx));
                row.add_child(edit);
            }
            this._ctxList.add_child(row);
        });
        const foot = new St.BoxLayout({style: ASTYLE.ctxFoot});
        foot.add_child(textButton('+ Novo contexto', ASTYLE.btn, ASTYLE.btnHover, () => this._openEditor(null)));
        foot.add_child(new St.Widget({x_expand: true}));
        foot.add_child(textButton('Abrir pasta', ASTYLE.btn, ASTYLE.btnHover, () => this._openContextDir()));
        this._ctxList.add_child(foot);
        this._ctxList.show();
        this._position();
    }

    // ---- Editor de contexto (ver, editar, criar) -----------------------------

    _openEditor(ctx) {
        this._ctxList.hide();
        for (const a of [this._body, this._actions, this._entryWrap, this._footer])
            a.hide();
        const ed = this._editor;
        ed.destroy_all_children();
        this._editing = ctx;

        ed.add_child(new St.Label({text: ctx ? 'NOME DO CONTEXTO' : 'NOME DO NOVO CONTEXTO', style: ASTYLE.edLabel}));
        this._edName = new St.Entry({style: ASTYLE.entry, text: ctx?.name ?? '', hint_text: 'Ex.: Trovana – atendimento', can_focus: true});
        ed.add_child(this._edName);

        ed.add_child(new St.Label({text: 'ESCOLHER SOZINHO NESTES APPS (opcional)', style: ASTYLE.edLabel}));
        this._edApps = new St.Entry({style: ASTYLE.entry, text: ctx?.apps.join(', ') ?? '', hint_text: 'whatsapp, instagram, slack', can_focus: true});
        ed.add_child(this._edApps);

        ed.add_child(new St.Label({text: 'INSTRUÇÕES, INFORMAÇÕES E REGRAS', style: ASTYLE.edLabel}));
        this._edText = new St.Entry({
            style: ASTYLE.edArea, can_focus: true,
            text: ctx?.raw ?? '',
            hint_text: 'Quem você é, o tom, e os fatos que a IA pode usar (frete, prazos, preços)…',
        });
        const ct = this._edText.clutter_text;
        ct.single_line_mode = false;
        ct.activatable = false;
        ct.line_wrap = true;
        ct.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        ed.add_child(this._edText);

        ed.add_child(new St.Label({text: 'Enter quebra linha. Uma regra por linha começando com "- " funciona bem.', style: ASTYLE.edHint}));

        const row = new St.BoxLayout({style: 'spacing: 6px; padding: 8px 0 14px 0;'});
        row.add_child(textButton('Salvar', STYLE.fixAll, STYLE.fixAllHover, () => this._saveEditor()));
        row.add_child(textButton('Voltar', ASTYLE.btn, ASTYLE.btnHover, () => this._closeEditor()));
        ed.add_child(row);

        for (const entry of [this._edName, this._edApps, this._edText]) {
            entry.clutter_text.connect('key-press-event', (_a, ev) => {
                if (ev.get_key_symbol() === Clutter.KEY_Escape) {
                    this._closeEditor();
                    return Clutter.EVENT_STOP;
                }
                return Clutter.EVENT_PROPAGATE;
            });
            entry.connect('button-press-event', () => {
                this._takeKeyboard(entry);
                return Clutter.EVENT_PROPAGATE;
            });
        }
        ed.show();
        this._position();
        this._takeKeyboard(ctx ? this._edText : this._edName);
    }

    _closeEditor() {
        this._editor.hide();
        this._editor.destroy_all_children();
        this._editing = undefined;
        this._releaseKeyboard();
        for (const a of [this._body, this._actions, this._entryWrap, this._footer])
            a.show();
        this._position();
    }

    _saveEditor() {
        const name = this._edName.text.trim().replace(/[\/\\:*?"<>|]/g, '-');
        if (!name) {
            this._edName.hint_text = 'Dê um nome ao contexto';
            return;
        }
        const apps = this._edApps.text.split(',').map(a => a.trim().toLowerCase()).filter(Boolean);
        const body = this._edText.text.replace(/\s+$/, '');
        GLib.mkdir_with_parents(CONTEXT_DIR, 0o700);
        const path = GLib.build_filenamev([CONTEXT_DIR, `${name}.md`]);
        GLib.file_set_contents(path, `${apps.length ? `apps: ${apps.join(', ')}\n` : ''}${body}\n`);
        // Renomeou: apaga o arquivo antigo.
        const old = this._editing;
        if (old?.file && old.file !== path) {
            try {
                Gio.File.new_for_path(old.file).delete(null);
            } catch (_e) {}
        }
        console.log(`${TAG} assistente: contexto ${old ? 'editado' : 'criado'}: ${name}`);
        this._contexts = loadContexts();
        const i = Math.max(0, this._contexts.findIndex(c => c.name === name));
        this._closeEditor();
        this._ruleNote = `Contexto "${name}" salvo.`;
        this._ctxIndex = -1; // força refazer a resposta com o contexto salvo
        this._chooseContext(i);
    }

    _chooseContext(i) {
        this._ctxList.hide();
        const changed = i !== this._ctxIndex;
        this._ctxIndex = i;
        this._ctxByApp.set(this._app, this._contexts[i].name);
        this._updateCtxLabel();
        if (changed && this._conversation) {
            this._history = [];
            this._generate(0.4);
        } else {
            this._position();
        }
    }

    _openContextDir() {
        GLib.mkdir_with_parents(CONTEXT_DIR, 0o700);
        try {
            const ctx = global.create_app_launch_context(global.get_current_time(), -1);
            Gio.AppInfo.launch_default_for_uri(Gio.File.new_for_path(CONTEXT_DIR).get_uri(), ctx);
        } catch (e) {
            console.log(`${TAG} não consegui abrir a pasta de contextos: ${e.message}`);
            Main.notify('Clippit', `Pasta dos contextos: ${CONTEXT_DIR}`);
        }
        this._close(false);
    }

    // "regra: nunca prometer prazo" no campo de ajuste → vai para o contexto atual.
    _saveRule(rule) {
        let ctx = this._contexts[this._ctxIndex];
        const name = !ctx || ctx.name === 'Sem contexto' ? 'Geral' : ctx.name;
        const path = GLib.build_filenamev([CONTEXT_DIR, `${name}.md`]);
        GLib.mkdir_with_parents(CONTEXT_DIR, 0o700);
        let text = '';
        try {
            text = new TextDecoder().decode(GLib.file_get_contents(path)[1]);
        } catch (_e) {}
        if (!/^Regras:\s*$/m.test(text))
            text = `${text.replace(/\s*$/, '')}\n\nRegras:\n`;
        text = `${text.replace(/\s*$/, '')}\n- ${rule}\n`;
        GLib.file_set_contents(path, text);
        this._contexts = loadContexts();
        this._ctxIndex = Math.max(0, this._contexts.findIndex(c => c.name === name));
        this._ctxByApp.set(this._app, name);
        this._updateCtxLabel();
        console.log(`${TAG} assistente: regra salva no contexto ${name}`);
        return name;
    }

    _toggleKeyboard() {
        if (this._editing !== undefined)
            return; // no editor o teclado fica no painel
        if (this._grab)
            this._releaseKeyboard();
        else
            this._takeKeyboard();
    }

    _takeKeyboard(target = this._entry) {
        if (this._grab) {
            target.grab_key_focus();
            return;
        }
        this._grab = Main.pushModal(this._panel, {actionMode: Shell.ActionMode.POPUP});
        this._syncEnter(); // digitando, Enter é do campo
        target.grab_key_focus();
        // Com o teclado preso no painel, clicar fora dele fecha.
        this._outsideId = global.stage.connect('captured-event', (_s, ev) => {
            if (ev.type() !== Clutter.EventType.BUTTON_PRESS)
                return Clutter.EVENT_PROPAGATE;
            const [x, y] = ev.get_coords();
            const [px, py] = this._panel.get_transformed_position();
            const [pw, ph] = this._panel.get_transformed_size();
            if (x < px || x > px + pw || y < py || y > py + ph) {
                this._close();
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        });
    }

    _releaseKeyboard() {
        if (this._outsideId) {
            global.stage.disconnect(this._outsideId);
            this._outsideId = 0;
        }
        if (this._grab) {
            Main.popModal(this._grab);
            this._grab = null;
            this._syncEnter(); // Enter volta a ser "inserir"
        }
    }

    _sendAsk() {
        const ask = this._entry.text.trim();
        if (!ask) {
            this._insert(); // Enter com o campo de ajuste vazio = inserir
            return;
        }
        if (this._busy)
            return;
        // "nota: é cliente desde 2023" / "sobre Marina: usa tamanho 16" → memória da pessoa.
        const note = ask.match(/^(?:nota|anota(?:r)?)\s*[:\-–]\s*(.+)$/i);
        const about = ask.match(/^sobre\s+(.+?)\s*[:\-–]\s*(.+)$/i);
        if (note || about) {
            this._entry.text = '';
            const who = about ? about[1].trim() : this._mainPerson();
            if (!who) {
                this._ruleNote = 'Não sei com quem é a conversa. Use "sobre Nome: …".';
            } else {
                learn(['--learn-note'], JSON.stringify({
                    conversation: this._conversation, name: who, note: (about ? about[2] : note[1]).trim(),
                }));
                this._ruleNote = `Anotado sobre ${who}: ${(about ? about[2] : note[1]).trim()}`;
            }
            if (this._options?.length)
                this._showReply();
            return;
        }
        const rule = ask.match(/^(?:salv[ae]r?\s+(?:isso\s+)?como\s+)?regras?\s*[:\-–]\s*(.+)$/i);
        this._entry.text = '';
        if (rule) {
            const name = this._saveRule(rule[1].trim());
            this._history = [];
            this._ruleNote = `Regra salva em "${name}": ${rule[1].trim()}`;
            if (this._conversation)
                this._generate(0.4);
            return;
        }
        if (!this._reply)
            return;
        this._history.push({reply: this._reply, ask});
        this._generate(0.4);
    }

    // Guarda o que você mandou de fato, para as próximas respostas saírem do seu jeito.
    _remember(text) {
        learn(['--learn'], JSON.stringify({
            conversation: this._conversation,
            suggested: this._modelReply ?? '',
            reply: text,
            app: this._app,
            context_name: this._contexts?.[this._ctxIndex]?.name ?? '',
        }));
    }

    _copy() {
        if (!this._reply)
            return;
        this._remember(this._reply);
        St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, this._reply);
        this._close();
        Main.notify('Clippit', 'Resposta copiada. Cole com Ctrl+V.');
    }

    async _insert() {
        const text = this._reply;
        if (!text || this._busy)
            return;
        const win = this._win;
        this._remember(text);
        this._close();
        // Devolve o foco para a janela da conversa e espera o campo voltar.
        if (win && global.display.focus_window !== win)
            win.activate(global.get_current_time());
        const im = Main.inputMethod;
        for (let i = 0; i < 30 && !im.currentFocus; i++)
            await sleep(50);
        if (!im.currentFocus) {
            St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, text);
            Main.notify('Clippit', 'Não achei o campo de texto. A resposta foi copiada: cole com Ctrl+V.');
            return;
        }
        const [current] = im.getSurroundingText();
        if (!(current ?? '').trim() && !text.includes('\n'))
            im.commit(text);
        else
            await this._c._pasteAll(text); // substitui o rascunho, mantém as quebras de linha
        console.log(`${TAG} assistente: resposta inserida app=${this._app} chars=${[...text].length}`);
    }

    _close(restoreBadge = true) {
        const hadPanel = Boolean(this._panel);
        if (this._shot) {
            try {
                Gio.File.new_for_path(this._shot).delete(null); // o print não fica guardado
            } catch (_e) {}
            this._shot = null;
        }
        this._gen = (this._gen ?? 0) + 1;
        this._releaseKeyboard();
        this._setKey('Escape', false);
        if (this._ajusteKey)
            this._setKey(this._ajusteKey, false);
        this._setKey('Return', false);
        this._setKey('KP_Enter', false);
        for (const d of ['1', '2', '3'])
            this._setKey(d, false);
        if (this._panel) {
            Main.layoutManager.removeChrome(this._panel);
            this._panel.destroy();
            this._panel = null;
        }
        this._reply = null;
        this._busy = false;
        if (hadPanel && restoreBadge)
            this._restoreCorrector();
    }

    // Ao fechar, o campo volta a ter foco: avisa o corretor para o selo voltar
    // sem precisar sair e entrar no campo.
    async _restoreCorrector() {
        const im = Main.inputMethod;
        for (let i = 0; i < 30 && !im.currentFocus; i++)
            await sleep(50);
        await sleep(150);
        if (im.currentFocus && this._c?._im)
            this._c._onSurroundingChanged();
    }
}

// ---- Lançador (Alt+Espaço) -------------------------------------------------
// Uma barra de busca no meio da tela, no estilo do Raycast: abre apps, troca de
// janela, faz contas e conversões, busca no histórico do Clippit e em
// arquivos, roda comandos do sistema, pesquisa na web e pergunta à IA local.

const BUSCA_CMD = clippitTool('clippit-busca');
const LAUNCH_WIDTH = 760;
const LAUNCH_HEIGHT = 470;
const LAUNCH_ROWS = 40;
// Mesmo tamanho em todas as telas (início, histórico e IA): a janela não pula.
const VIEW_SIZE = {root: [1040, 640], historico: [1040, 640], ia: [1040, 640]};
const KEEP_TEXT_SECONDS = 300; // fechou sem querer? reabrindo em até 5 min, o texto volta
const USAGE_FILE = GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'lancador.json']);
const QUICKLINKS_FILE = GLib.build_filenamev([GLib.get_home_dir(), '.config', 'clippit', 'atalhos-web.json']);

// Visual no estilo do Raycast: sem caixas com borda; busca grande no topo,
// lista limpa, prévia ao lado (histórico) e barra de ações embaixo.
const LSTYLE = {
    panel: UI.panel,
    header: UI.header,
    entry: `${TX.input} background-color: transparent; border: none; padding: 2px 0; caret-color: ${C.accent};
        selected-color: #ffffff; selection-background-color: rgba(76,157,255,0.45);`,
    entryHint: `color: ${T.text3};`,
    back: UI.iconBtn,
    backHover: UI.iconBtnHover,
    mode: TX.tiny,
    sep: UI.sep,
    vsep: UI.vsep,
    list: `padding: 2px 8px 6px 8px; spacing: 1px;`,
    section: `${TX.label} padding: 10px 10px 5px 10px;`,
    row: UI.row,
    rowOn: UI.rowOn,
    title: TX.body,
    kind: TX.tiny,
    swatch: `border-radius: ${T.r1}; width: 18px; height: 18px; border: 1px solid rgba(255,255,255,0.15);`,
    // Prévia do histórico
    detail: `padding: 14px ${T.pad}px; spacing: 14px;`,
    preview: TX.body,
    infoTitle: TX.label,
    infoRow: `padding: 6px 0; spacing: 8px; border-top: 1px solid ${T.line};`,
    infoKey: TX.small,
    infoVal: `${TX.small} color: ${T.text};`,
    // Chat da IA
    chat: `padding: 6px ${T.pad}px 18px ${T.pad}px; spacing: 14px;`,
    bubble: `${TX.body} background-color: ${T.accentSoft}; border-radius: ${T.r4}; padding: 8px 12px;`,
    aiBlock: `spacing: 8px;`,
    aiText: TX.body,
    steps: `spacing: 4px;`,
    step: `${UI.row} min-height: 0; padding: 6px 10px; spacing: 10px; background-color: ${T.surface};`,
    stepTitle: TX.smallStrong,
    stepDetail: TX.small,
    stepTime: TX.tiny,
    thinking: `${TX.small} font-style: italic;`,
    // Saída de comando
    outBox: UI.code,
    outCmd: `${TX.mono} color: ${C.accent};`,
    outText: TX.mono,
    outCode: TX.tiny,
    confirmRow: `spacing: 8px; padding-top: 2px;`,
    runBtn: UI.btnPrimary,
    runBtnHover: UI.btnPrimaryHover,
    cancelBtn: UI.btn,
    cancelBtnHover: UI.btnHover,
    // Perguntas com opções
    choiceBox: `spacing: 4px; padding-top: 2px;`,
    choice: `${UI.card} spacing: 10px;`,
    choiceOn: `${UI.cardOn} spacing: 10px;`,
    choiceKey: `${UI.key} color: ${C.accent}; background-color: ${T.accentSoft}; min-width: 10px;`,
    choiceText: TX.body,
    // Linha de correção / ditado e lista do @
    fixRow: `spacing: 10px; padding: 0 ${T.pad}px 10px ${T.pad}px;`,
    fixText: TX.small,
    mention: `padding: 0 8px 8px 8px; spacing: 1px;`,
    // Painel ao lado (Alt+C): mesmo vidro, mesmos textos
    side: UI.panel,
    sideHead: UI.header,
    sideTitle: TX.bodyStrong,
    sideList: `padding: 2px 8px 6px 8px; spacing: 1px;`,
    sideFoot: `${TX.small} padding: 10px ${T.pad}px 12px ${T.pad}px;`,
    sideText: TX.body,
    sideSub: TX.small,
    sideThumb: `border-radius: ${T.r2};`,
    // Seletor de emoji (grade)
    emojiBar: `padding: 0 ${T.pad}px 10px ${T.pad}px; spacing: 6px;`,
    chip: `${TX.small} color: ${T.text2}; background-color: ${T.surface}; border-radius: ${T.r2}; padding: 4px 10px;`,
    chipOn: `${TX.small} color: ${C.accent}; background-color: ${T.accentSoft}; border-radius: ${T.r2}; padding: 4px 10px; font-weight: 600;`,
    emojiGrid: `padding: 0 ${T.pad}px 14px ${T.pad}px; spacing: 8px;`,
    emojiSection: `${TX.label} padding: 6px 0 0 2px;`,
    emojiRow: `spacing: 8px;`,
    tile: `border-radius: ${T.r3}; background-color: ${T.surface}; border: 2px solid transparent;`,
    tileOn: `border-radius: ${T.r3}; background-color: ${T.hover}; border: 2px solid rgba(255,255,255,0.75);`,
    tileGlyph: `font-size: 24pt;`,
    // Rodapé
    footer: UI.footer,
    footApp: TX.small,
    footAction: TX.smallStrong,
    footKey: UI.key,
    footDim: TX.small,
    empty: TX.small,
};

const KIND = {
    'Apps': 'Aplicativo', 'Mais usados': 'Aplicativo', 'Janelas abertas': 'Janela', 'Comandos': 'Comando',
    'Calculadora': 'Calculadora', 'Web': 'Web', 'Arquivos': 'Arquivo', 'Histórico do Clippit': 'Histórico',
    'IA local': 'IA', 'Atalhos': 'Atalho', 'Sugestões': 'Comando',
};
const HEX_COLOR = /^#(?:[0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i;

// ---- Busca aproximada ----------------------------------------------------

const fold = s => (s ?? '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();

function editDistance(a, b) {
    const d = Array.from({length: a.length + 1}, (_, i) => [i, ...new Array(b.length).fill(0)]);
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

// 0 = não combina. Começo do nome > começo de palavra > contém > letras em
// ordem ("gch" → Google Chrome) > um erro de digitação ("crom" → chrome).
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
    const initials = words.map(w => w[0] ?? '').join('');
    if (q.length >= 2 && initials.startsWith(q))
        return 70;
    if (q.length >= 4 && words.some(w => editDistance(q, w.slice(0, q.length + 1)) <= 1))
        return 45; // um erro de digitação ("crom" → chrome)
    let i = 0;
    for (const ch of t) {
        if (ch === q[i])
            i++;
        if (i === q.length)
            return 30; // só letras em ordem: fraco, entra só se não houver nada melhor
    }
    return 0;
}

// ---- Calculadora e conversões ------------------------------------------------

const UNITS = {
    // comprimento (metros)
    mm: ['comprimento', 0.001], cm: ['comprimento', 0.01], m: ['comprimento', 1], km: ['comprimento', 1000],
    pol: ['comprimento', 0.0254], in: ['comprimento', 0.0254], polegada: ['comprimento', 0.0254], polegadas: ['comprimento', 0.0254],
    ft: ['comprimento', 0.3048], pe: ['comprimento', 0.3048], pes: ['comprimento', 0.3048],
    yd: ['comprimento', 0.9144], mi: ['comprimento', 1609.344], milha: ['comprimento', 1609.344], milhas: ['comprimento', 1609.344],
    metro: ['comprimento', 1], metros: ['comprimento', 1], quilometro: ['comprimento', 1000], quilometros: ['comprimento', 1000],
    // massa (gramas)
    mg: ['massa', 0.001], g: ['massa', 1], kg: ['massa', 1000], t: ['massa', 1e6], ton: ['massa', 1e6],
    lb: ['massa', 453.59237], lbs: ['massa', 453.59237], libra: ['massa', 453.59237], libras: ['massa', 453.59237],
    oz: ['massa', 28.349523], grama: ['massa', 1], gramas: ['massa', 1], quilo: ['massa', 1000], quilos: ['massa', 1000],
    // tempo (segundos)
    ms: ['tempo', 0.001], s: ['tempo', 1], seg: ['tempo', 1], segundo: ['tempo', 1], segundos: ['tempo', 1],
    min: ['tempo', 60], minuto: ['tempo', 60], minutos: ['tempo', 60], h: ['tempo', 3600], hora: ['tempo', 3600],
    horas: ['tempo', 3600], dia: ['tempo', 86400], dias: ['tempo', 86400], semana: ['tempo', 604800], semanas: ['tempo', 604800],
    // dados (bytes)
    b: ['dados', 1], kb: ['dados', 1024], mb: ['dados', 1024 ** 2], gb: ['dados', 1024 ** 3], tb: ['dados', 1024 ** 4],
    // volume (litros)
    ml: ['volume', 0.001], l: ['volume', 1], litro: ['volume', 1], litros: ['volume', 1],
    // velocidade (m/s)
    'km/h': ['velocidade', 1 / 3.6], kmh: ['velocidade', 1 / 3.6], 'm/s': ['velocidade', 1], mph: ['velocidade', 0.44704],
    // temperatura (tratada à parte)
    c: ['temperatura', 0], '°c': ['temperatura', 0], celsius: ['temperatura', 0],
    f: ['temperatura', 0], '°f': ['temperatura', 0], fahrenheit: ['temperatura', 0],
    k: ['temperatura', 0], kelvin: ['temperatura', 0],
};

function toCelsius(v, u) {
    u = u.replace('°', '');
    return u.startsWith('f') ? (v - 32) * 5 / 9 : u.startsWith('k') ? v - 273.15 : v;
}

function fromCelsius(v, u) {
    u = u.replace('°', '');
    return u.startsWith('f') ? v * 9 / 5 + 32 : u.startsWith('k') ? v + 273.15 : v;
}

const parseNum = s => Number(String(s).replace(/\./g, (m, i, all) => (all.includes(',') ? '' : m)).replace(',', '.'));

function formatNum(n) {
    if (!Number.isFinite(n))
        return null;
    const r = Math.abs(n) >= 1e15 || (Math.abs(n) < 1e-6 && n !== 0) ? n.toExponential(6) : Number(n.toPrecision(12));
    return typeof r === 'string' ? r.replace('.', ',') : r.toLocaleString('pt-BR', {maximumFractionDigits: 6});
}

function convert(query) {
    // "2h30 em minutos"
    let m = fold(query).match(/^(\d+)\s*h\s*(\d+)\s*(?:min)?\s+(?:em|para|in|to)\s+([a-z°/]+)$/);
    if (m) {
        const secs = Number(m[1]) * 3600 + Number(m[2]) * 60;
        const to = UNITS[m[3]];
        if (to?.[0] === 'tempo')
            return {value: secs / to[1], unit: m[3]};
    }
    m = fold(query).replace(/\s+/g, ' ').match(/^(-?[\d.,]+)\s*([a-z°/]+)\s+(?:em|para|in|to)\s+([a-z°/]+)$/);
    if (!m)
        return null;
    const [, num, from, to] = m;
    const a = UNITS[from], b = UNITS[to];
    if (!a || !b || a[0] !== b[0])
        return null;
    const v = parseNum(num);
    if (a[0] === 'temperatura')
        return {value: fromCelsius(toCelsius(v, from), to), unit: to.replace('°', '').toUpperCase()};
    return {value: v * a[1] / b[1], unit: to};
}

// Avaliador de contas (sem eval): + - * / ^ %, parênteses, "15% de 340".
function calculate(query) {
    let s = fold(query).trim();
    if (!/\d/.test(s) || !/[-+*/x×÷^%()]|de |of /.test(s))
        return null;
    let m = s.match(/^([\d.,]+)\s*%\s*(?:de|of)\s*([\d.,]+)$/);
    if (m)
        return parseNum(m[1]) / 100 * parseNum(m[2]);
    // Número brasileiro: "1.234,5" → 1234.5 (com vírgula, o ponto é de milhar).
    if (s.includes(','))
        s = s.replace(/(\d)\.(?=\d{3}\b)/g, '$1').replace(/,/g, '.');
    s = s.replace(/×|x/g, '*').replace(/÷/g, '/').replace(/\s+/g, '');
    if (!/^[\d.+\-*/^%()]+$/.test(s))
        return null;
    let i = 0;
    const peek = () => s[i];
    const num = () => {
        const st = i;
        while (i < s.length && /[\d.]/.test(s[i]))
            i++;
        if (st === i)
            throw new Error('número');
        return Number(s.slice(st, i));
    };
    const factor = () => {
        if (peek() === '-') {
            i++;
            return -factor();
        }
        if (peek() === '+') {
            i++;
            return factor();
        }
        let v;
        if (peek() === '(') {
            i++;
            v = expr();
            if (s[i++] !== ')')
                throw new Error('parêntese');
        } else {
            v = num();
        }
        if (peek() === '%') {
            i++;
            v /= 100;
        }
        if (peek() === '^') {
            i++;
            v **= factor();
        }
        return v;
    };
    const term = () => {
        let v = factor();
        while (peek() === '*' || peek() === '/') {
            const op = s[i++];
            v = op === '*' ? v * factor() : v / factor();
        }
        return v;
    };
    // "340 + 15%" soma 15% de 340 (como nas calculadoras).
    const expr = () => {
        let v = term();
        while (peek() === '+' || peek() === '-') {
            const op = s[i++];
            const st = i;
            let w = term();
            if (s[i - 1] === '%' && /^[\d.]+%$/.test(s.slice(st, i)))
                w = v * w;
            v = op === '+' ? v + w : v - w;
        }
        return v;
    };
    try {
        const v = expr();
        return i === s.length && Number.isFinite(v) ? v : null;
    } catch (_e) {
        return null;
    }
}

// Markdown simples da IA → marcação do GNOME (negrito, código).
function mdToMarkup(md) {
    let out = esc(md);
    out = out.replace(/```[a-z]*\n([\s\S]*?)```/g, (_m, code) => `<tt>${code.replace(/\n$/, '')}</tt>`);
    out = out.replace(/`([^`\n]+)`/g, '<tt>$1</tt>');
    out = out.replace(/\*\*([^*\n]+)\*\*/g, '<b>$1</b>');
    out = out.replace(/^#{1,4}\s*(.+)$/gm, '<b>$1</b>');
    return out;
}

const WEB = [
    {key: 'g', name: 'Google', url: 'https://www.google.com/search?q={q}', icon: 'web-browser-symbolic'},
    {key: 'yt', name: 'YouTube', url: 'https://www.youtube.com/results?search_query={q}', icon: 'video-x-generic-symbolic'},
    {key: 'gh', name: 'GitHub', url: 'https://github.com/search?q={q}', icon: 'system-software-install-symbolic'},
    {key: 'w', name: 'Wikipédia', url: 'https://pt.wikipedia.org/w/index.php?search={q}', icon: 'accessories-dictionary-symbolic'},
    {key: 'm', name: 'Google Maps', url: 'https://www.google.com/maps/search/{q}', icon: 'mark-location-symbolic'},
    {key: 'tr', name: 'Google Tradutor', url: 'https://translate.google.com/?sl=auto&tl=pt&text={q}', icon: 'preferences-desktop-locale-symbolic'},
];

// ---- Ícones (Lucide, ISC) ----------------------------------------------------
// SVGs já coloridos em icons/ (gerados por scripts/gerar-icones.py). O core.js
// roda de uma cópia em /run, então o caminho é o da extensão instalada.
const ICON_DIR = GLib.build_filenamev([GLib.get_user_data_dir(), 'gnome-shell', 'extensions', 'clippit-corretor@clippit', 'icons']);

function lucide(name, color = 'g') {
    const path = GLib.build_filenamev([ICON_DIR, `${name}-${color}.svg`]);
    return GLib.file_test(path, GLib.FileTest.EXISTS) ? new Gio.FileIcon({file: Gio.File.new_for_path(path)}) : null;
}

// Ícone do tema (nome antigo) → [ícone Lucide, cor do quadradinho].
const ICON_MAP = {
    'edit-paste-symbolic': ['clipboard-list', '#f97316'], 'starred-symbolic': ['sparkles', '#3b82f6'],
    'input-gaming-symbolic': ['gamepad-2', '#22c55e'], 'preferences-system-symbolic': ['settings', '#6b7280'],
    'org.gnome.Settings-symbolic': ['sliders-horizontal', '#6b7280'], 'system-lock-screen-symbolic': ['lock', '#64748b'],
    'weather-clear-night-symbolic': ['moon', '#4f46e5'], 'system-reboot-symbolic': ['rotate-cw', '#f59e0b'],
    'system-shutdown-symbolic': ['power', '#ef4444'], 'system-log-out-symbolic': ['log-out', '#ef4444'],
    'user-trash-full-symbolic': ['trash-2', '#ef4444'], 'tools-check-spelling-symbolic': ['spell-check', '#14b8a6'],
    'user-available-symbolic': ['messages-square', '#3b82f6'], 'insert-text-symbolic': ['text-cursor-input', '#06b6d4'],
    'face-smile-symbolic': ['smile', '#eab308'], 'edit-select-all-symbolic': ['wand-sparkles', '#0ea5e9'],
    'network-wireless-symbolic': ['wifi', '#3b82f6'], 'bluetooth-active-symbolic': ['bluetooth', '#2563eb'],
    'notifications-disabled-symbolic': ['bell-off', '#ef4444'], 'audio-volume-muted-symbolic': ['volume-x', '#64748b'],
    'audio-volume-high-symbolic': ['volume-2', '#10b981'], 'audio-volume-low-symbolic': ['volume-1', '#10b981'],
    'display-brightness-symbolic': ['sun', '#f59e0b'], 'alarm-symbolic': ['alarm-clock', '#f97316'],
    'view-dual-symbolic': ['panel-left', '#0ea5e9'], 'window-maximize-symbolic': ['maximize-2', '#0ea5e9'],
    'window-restore-symbolic': ['minimize-2', '#0ea5e9'], 'focus-windows-symbolic': ['square-dashed', '#0ea5e9'],
    'video-display-symbolic': ['monitor', '#0ea5e9'], 'view-fullscreen-symbolic': ['fullscreen', '#0ea5e9'],
    'window-minimize-symbolic': ['arrow-down-to-line', '#0ea5e9'], 'window-close-symbolic': ['square-x', '#ef4444'],
    'web-browser-symbolic': ['globe', '#3b82f6'], 'video-x-generic-symbolic': ['youtube', '#ef4444'],
    'system-software-install-symbolic': ['github', '#374151'], 'accessories-dictionary-symbolic': ['book-open', '#6b7280'],
    'mark-location-symbolic': ['map-pin', '#22c55e'], 'preferences-desktop-locale-symbolic': ['languages', '#0ea5e9'],
    'accessories-calculator-symbolic': ['calculator', '#f59e0b'], 'document-edit-symbolic': ['pencil', '#6b7280'],
    'list-add-symbolic': ['plus', '#22c55e'], 'text-x-generic-symbolic': ['file-text', '#6b7280'],
    'view-pin-symbolic': ['pin', '#3b82f6'], 'edit-clear-symbolic': ['eraser', '#6b7280'],
    'format-text-bold-symbolic': ['case-upper', '#6b7280'], 'format-text-plaintext-symbolic': ['case-lower', '#6b7280'],
    'format-justify-fill-symbolic': ['wrap-text', '#6b7280'], 'format-justify-left-symbolic': ['wrap-text', '#6b7280'],
    'text-x-script-symbolic': ['braces', '#6b7280'], 'view-list-bullet-symbolic': ['list', '#0ea5e9'],
    'help-about-symbolic': ['circle-help', '#0ea5e9'], 'mail-reply-sender-symbolic': ['reply', '#0ea5e9'],
    'image-x-generic-symbolic': ['image', '#6b7280'], 'application-x-executable-symbolic': ['app-window', '#6b7280'],
    'system-run-symbolic': ['terminal', '#6b7280'], 'edit-find-symbolic': ['search', '#6b7280'],
    'camera-photo-symbolic': ['camera', '#6b7280'],
};
// Alguns itens pedem um ícone próprio (o nome do tema se repete).
const ICON_BY_ID = {
    'ctl:dark': ['sun-moon', '#4f46e5'], 'win:right': ['panel-right', '#0ea5e9'], 'calc': ['calculator', '#f59e0b'],
    'cmd:clippit': ['settings', '#6b7280'], 'timer': ['alarm-clock', '#f97316'],
};

// Ícone de vidro (padrão): fundo translúcido, borda fina e o desenho em branco
// suave. A cor do mapa fica só como referência; o visual é o mesmo em tudo.
function iconTile(name, _color, size = 24) {
    const tile = new St.Bin({
        style: `width: ${size}px; height: ${size}px; border-radius: ${size > 30 ? T.r3 : T.r2};
            background-color: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.10);`,
        y_align: Clutter.ActorAlign.CENTER,
    });
    const gicon = lucide(name, 'w');
    if (gicon)
        tile.set_child(new St.Icon({gicon, icon_size: Math.round(size * 0.58), opacity: 215,
            x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER}));
    return tile;
}

// ---- Caixa rolável que não espreme o conteúdo --------------------------------
// A rolagem do GNOME 46 usa a altura MÍNIMA do conteúdo; textos que quebram
// linha têm mínima menor que a real, e os blocos ficavam espremidos. Esta caixa
// responde a altura real nos dois casos, calculada na hora (nunca um número
// fixo: fixar altura mínima nos filhos derrubou o GNOME em 26/11, porque o
// aviso "natural < minimum" do ClutterBoxLayout aborta a sessão).
// O nome do tipo muda a cada carga porque o core.js é recarregado sem reiniciar.
const FullHeightBox = GObject.registerClass({GTypeName: `ClippitFullHeightBox${Date.now()}`},
    class FullHeightBox extends St.BoxLayout {
        vfunc_get_preferred_height(forWidth) {
            const [, nat] = super.vfunc_get_preferred_height(forWidth);
            return [nat, nat];
        }
    });

// ---- Rolagem suave ----------------------------------------------------------
// A roda do mouse anda aos saltos; aqui cada passo desliza (touchpad continua
// como já era, porque já é contínuo).
function smoothStep(adj, dy, holder) {
    if (!adj)
        return;
    const max = Math.max(adj.lower, adj.upper - adj.page_size);
    const step = Math.max(56, Math.min(160, adj.page_size * 0.2));
    const from = holder._smoothTarget ?? adj.value;
    const target = Math.max(adj.lower, Math.min(max, from + dy * step));
    holder._smoothTarget = target;
    adj.remove_transition?.('value');
    adj.ease(target, {
        duration: 180, mode: Clutter.AnimationMode.EASE_OUT_CUBIC,
        onStopped: () => {
            holder._smoothTarget = null;
        },
    });
}

function smoothScroll(view, getAdj) {
    view.connect('scroll-event', (_a, ev) => {
        const dir = ev.get_scroll_direction();
        let dy = dir === Clutter.ScrollDirection.DOWN ? 1 : dir === Clutter.ScrollDirection.UP ? -1 : 0;
        if (dir === Clutter.ScrollDirection.SMOOTH) {
            if (ev.get_scroll_source?.() === Clutter.ScrollSource.FINGER)
                return Clutter.EVENT_PROPAGATE; // touchpad: já é suave
            dy = ev.get_scroll_delta()[1];
            if (!dy)
                return Clutter.EVENT_PROPAGATE;
        }
        if (!dy)
            return Clutter.EVENT_PROPAGATE;
        smoothStep(getAdj(), dy, view);
        return Clutter.EVENT_STOP;
    });
}

// ---- Snippets: textos prontos chamados por ;gatilho -----------------------
const SNIPPETS_FILE = GLib.build_filenamev([GLib.get_user_config_dir(), 'clippit', 'snippets.json']);
let _snipCache = {stamp: null, items: []};

function loadSnippets() {
    try {
        const info = Gio.File.new_for_path(SNIPPETS_FILE).query_info('time::modified', Gio.FileQueryInfoFlags.NONE, null);
        const stamp = info.get_attribute_uint64('time::modified');
        if (stamp !== _snipCache.stamp) {
            const items = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(SNIPPETS_FILE)[1]));
            _snipCache = {stamp, items: Array.isArray(items) ? items.filter(x => x.gatilho && x.texto) : []};
        }
    } catch (_e) {
        _snipCache = {stamp: null, items: []};
    }
    return _snipCache.items;
}

function saveSnippets(items) {
    GLib.mkdir_with_parents(GLib.path_get_dirname(SNIPPETS_FILE), 0o700);
    GLib.file_set_contents(SNIPPETS_FILE, JSON.stringify(items, null, 2));
}

function clipboardText(type = St.ClipboardType.CLIPBOARD) {
    return new Promise(resolve => St.Clipboard.get_default().get_text(type, (_c, t) => resolve(t ?? '')));
}

// {data} {hora} {area} (o que está copiado)
async function expandSnippet(text) {
    const now = GLib.DateTime.new_now_local();
    let out = text.replaceAll('{data}', now.format('%d/%m/%Y')).replaceAll('{hora}', now.format('%H:%M'));
    if (out.includes('{area}'))
        out = out.replaceAll('{area}', await clipboardText());
    return out;
}

// Cola um texto onde está o cursor, como Ctrl+V (e devolve o que estava copiado).
let _vkb = null;
async function pasteViaClipboard(text, restore = true) {
    const old = restore ? await clipboardText() : null;
    St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, text);
    await sleep(60);
    _vkb ??= Clutter.get_default_backend().get_default_seat().create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
    const key = (kv, down) => _vkb.notify_keyval(GLib.get_monotonic_time(), kv,
        down ? Clutter.KeyState.PRESSED : Clutter.KeyState.RELEASED);
    key(Clutter.KEY_Control_L, true);
    key(Clutter.KEY_v, true);
    key(Clutter.KEY_v, false);
    key(Clutter.KEY_Control_L, false);
    if (old) {
        await sleep(900);
        St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, old);
    }
}

// ---- Transformações de texto (área de transferência e Alt+A) ----------------
const TEXT_TRANSFORMS = [
    {id: 'maiusculas', title: 'MAIÚSCULAS', icon: 'format-text-bold-symbolic', fn: t => t.toLocaleUpperCase('pt-BR')},
    {id: 'minusculas', title: 'minúsculas', icon: 'format-text-plaintext-symbolic', fn: t => t.toLocaleLowerCase('pt-BR')},
    {id: 'titulo', title: 'Iniciais Maiúsculas', icon: 'format-text-plaintext-symbolic',
        fn: t => t.toLocaleLowerCase('pt-BR').replace(/(^|[\s(\["'])(\p{L})/gu, (_m, a, b) => a + b.toLocaleUpperCase('pt-BR'))},
    {id: 'linha', title: 'Uma linha só (sem quebras)', icon: 'format-justify-fill-symbolic', fn: t => t.replace(/\s+/g, ' ').trim()},
    {id: 'limpar', title: 'Sem espaços sobrando', icon: 'edit-clear-symbolic',
        fn: t => t.split('\n').map(l => l.replace(/[ \t]+/g, ' ').trim()).join('\n').replace(/\n{3,}/g, '\n\n').trim()},
    {id: 'json', title: 'Formatar JSON', icon: 'text-x-script-symbolic', fn: t => JSON.stringify(JSON.parse(t), null, 2)},
    {id: 'links', title: 'Extrair links', icon: 'web-browser-symbolic',
        fn: t => [...new Set(t.match(/https?:\/\/[^\s<>"')\]]+/g) ?? [])].join('\n') || (() => {
            throw new Error('nenhum link');
        })()},
    {id: 'numeros', title: 'Só os números', icon: 'accessories-calculator-symbolic',
        fn: t => t.replace(/\D+/g, '') || (() => {
            throw new Error('nenhum número');
        })()},
    {id: 'corrigir', title: 'Corrigir o texto', icon: 'tools-check-spelling-symbolic', ai: 'corrigir'},
    {id: 'traduzir', title: 'Traduzir (pt ↔ en)', icon: 'preferences-desktop-locale-symbolic', ai: 'traduzir'},
    {id: 'resumir', title: 'Resumir', icon: 'view-list-bullet-symbolic', ai: 'resumir'},
];

async function runTransform(tf, text) {
    if (tf.ai) {
        const out = await runProcess([CORRECTOR_CMD, '--transformar'], JSON.stringify({acao: tf.ai, texto: text}), 240);
        return out.trim();
    }
    return tf.fn(text);
}

const LETTERS = 'ABCDEF';

// Os mais usados ganham no desempate (:coração → ❤️, :ok → 👍).
const POPULAR_EMOJI = ['❤️', '👍', '🔥', '✅', '😂', '🙏', '😊', '🎉', '👀', '💯', '😍', '🤔', '😅', '👏', '🚀', '✨', '😭', '🥲', '😎', '👌', '✔️', '⚠️', '❌', '⭐'];

// Ações do Alt+A sobre o texto selecionado.
const TEXT_ACTIONS = [
    {acao: 'resumir', title: 'Resumir', icon: 'view-list-bullet-symbolic', words: ['resumo', 'summary']},
    {acao: 'traduzir', title: 'Traduzir (pt ↔ en)', icon: 'preferences-desktop-locale-symbolic', words: ['tradução', 'translate', 'ingles']},
    {acao: 'explicar', title: 'Explicar', icon: 'help-about-symbolic', words: ['o que é', 'explain']},
    {acao: 'corrigir', title: 'Corrigir', icon: 'tools-check-spelling-symbolic', words: ['ortografia', 'erros']},
    {acao: 'formal', title: 'Deixar mais formal', icon: 'format-text-bold-symbolic', words: ['profissional']},
    {acao: 'casual', title: 'Deixar mais casual', icon: 'face-smile-symbolic', words: ['informal', 'leve']},
    {acao: 'curto', title: 'Deixar mais curto', icon: 'format-justify-left-symbolic', words: ['resumir', 'enxugar']},
    {acao: 'responder', title: 'Escrever uma resposta', icon: 'mail-reply-sender-symbolic', words: ['responder', 'reply']},
    {acao: 'perguntar', title: 'Perguntar à IA sobre o texto', icon: 'starred-symbolic', words: ['chat', 'ia']},
];

// Um só mouse virtual para a extensão toda. Ao carregar, solta qualquer botão
// que tenha ficado preso (versão antiga repetia só o "apertar").
let _vptr = null;
function virtualPointer() {
    _vptr ??= Clutter.get_default_backend().get_default_seat().create_virtual_device(Clutter.InputDeviceType.POINTER_DEVICE);
    return _vptr;
}

class Lancador {
    constructor(corretor) {
        this._c = corretor;
        this._grabs = new Map();
        this._usage = this._loadUsage();
    }

    enable() {
        // Destrava: se algum botão ficou apertado por um clique repetido antigo.
        // Os mouses virtuais de versões antigas são apagados pela coleta de
        // lixo, e o GNOME solta o que estava apertado neles.
        const vp = virtualPointer();
        for (const b of [1, 2, 3])
            vp.notify_button(GLib.get_monotonic_time(), b, Clutter.ButtonState.RELEASED);
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
            System.gc();
            return GLib.SOURCE_REMOVE;
        });
        bindShortcuts(this, [['lancador', () => {
            if (this._panel && !this._grab)
                this._wake(); // estava solto: volta
            else if (this._panel)
                this._close();
            else
                this.open();
        }]]);
        // D-Bus: o atalho do Clippit (clippit-historico) pede para abrir uma tela.
        // A IA (clippit-correct) também usa: listar/mexer em janelas e tirar print.
        const iface = `<node><interface name="com.clippit.Lancador">
            <method name="Abrir"><arg type="s" direction="in" name="tela"/></method>
            <method name="Janelas"><arg type="s" direction="out" name="json"/></method>
            <method name="Janela"><arg type="s" direction="in" name="acao"/><arg type="s" direction="in" name="alvo"/>
                <arg type="s" direction="out" name="json"/></method>
            <method name="Print"><arg type="s" direction="in" name="onde"/><arg type="s" direction="in" name="caminho"/>
                <arg type="s" direction="out" name="resultado"/></method>
            <method name="AreaHash"><arg type="s" direction="out" name="sha256"/></method>
        </interface></node>`;
        this._dbus = Gio.DBusExportedObject.wrapJSObject(iface, {
            Abrir: tela => this.toggleView(['historico', 'ia'].includes(tela) ? tela : 'root'),
            Janelas: () => JSON.stringify(this._listWindows()),
            Janela: (acao, alvo) => JSON.stringify(this._aiWindowOp(acao, alvo)),
            // Doutor do Clippit: só a impressão digital do que está copiado (nunca o texto),
            // para saber se o histórico está gravando — sem roubar o foco de ninguém.
            AreaHashAsync: (_p, inv) => {
                clipboardText().then(t => inv.return_value(new GLib.Variant('(s)',
                    [t ? GLib.compute_checksum_for_string(GLib.ChecksumType.SHA256, t, -1) : ''])))
                    .catch(() => inv.return_value(new GLib.Variant('(s)', [''])));
            },
            PrintAsync: ([onde, caminho], inv) => {
                this._aiPrint(onde, caminho).then(r => inv.return_value(new GLib.Variant('(s)', [r])))
                    .catch(e => inv.return_value(new GLib.Variant('(s)', [`erro: ${e.message}`])));
            },
        });
        try {
            this._dbus.export(Gio.DBus.session, '/com/clippit/Lancador');
            this._nameId = Gio.bus_own_name_on_connection(Gio.DBus.session, 'com.clippit.Lancador',
                Gio.BusNameOwnerFlags.REPLACE | Gio.BusNameOwnerFlags.ALLOW_REPLACEMENT, null, null);
        } catch (e) {
            console.log(`${TAG} lançador: D-Bus indisponível (${e.message})`);
        }
        // Alt+I: abre direto na tela da IA (aperte de novo para fechar).
        bindShortcuts(this, [['ia', () => this.toggleView('ia')]]);
        // Foto do lançador (só para conferir o visual): tela inicial e histórico.
        const probe6 = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-foto']);
        if (GLib.file_test(probe6, GLib.FileTest.EXISTS)) {
            const out = new TextDecoder().decode(GLib.file_get_contents(probe6)[1]).trim();
            GLib.unlink(probe6);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
                (async () => {
                    this._selfTesting = true;
                    const shot = async name => {
                        await sleep(700);
                        const [x, y] = this._panel.get_transformed_position();
                        const [w, h] = this._panel.get_transformed_size();
                        await new Promise(res => {
                            const stream = Gio.File.new_for_path(`${out}/${name}.png`).replace(null, false, Gio.FileCreateFlags.NONE, null);
                            new Shell.Screenshot().screenshot_area(x, y, w, h, stream, (o, r) => {
                                try {
                                    o.screenshot_area_finish(r);
                                } catch (_e) {}
                                stream.close(null);
                                res();
                            });
                        });
                    };
                    const shotSide = async name => {
                        await sleep(700);
                        const [x, y] = this._panel.get_transformed_position();
                        const [sx] = this._side.get_transformed_position();
                        const [sw] = this._side.get_transformed_size();
                        const [, h] = this._panel.get_transformed_size();
                        await new Promise(res => {
                            const stream = Gio.File.new_for_path(`${out}/${name}.png`).replace(null, false, Gio.FileCreateFlags.NONE, null);
                            new Shell.Screenshot().screenshot_area(x, y, sx + sw - x, h, stream, (o, r) => {
                                try {
                                    o.screenshot_area_finish(r);
                                } catch (_e) {}
                                stream.close(null);
                                res();
                            });
                        });
                    };
                    this.open('root');
                    await shot('inicio');
                    this._entry.text = 'modo';
                    await shot('comandos');
                    this._entry.text = 'zzzxq';
                    await shot('vazio');
                    this._enterView('historico');
                    await this._searchAsync('');
                    this._refresh();
                    await shot('historico');
                    this._chat = [];
                    this._enterView('ia');
                    await shot('ia-vazia');
                    this._chat = [
                        {role: 'user', text: 'o que tá ocupando minha memória?'},
                        {role: 'assistant', text: 'Qual memória?', steps: [], choice: {pergunta: 'Qual memória?',
                            opcoes: ['Memória RAM (programas abertos)', 'Armazenamento (espaço no disco)']}, choiceSel: 1, choiceAnswered: 1},
                        {role: 'user', text: 'Armazenamento (espaço no disco)'},
                        {role: 'assistant', text: 'O disco principal está com **92% de uso** (408 GB de 468 GB).',
                            steps: [{titulo: 'Rodando comando', detalhe: 'df -h', estado: 'feito', segundos: 0.1}],
                            outputs: [{comando: 'df -h /', saida: 'Sist. Arq.  Tam. Usado Disp. Uso%\n/dev/nvme0n1p2  468G  408G  37G  92%', codigo: 0, segundos: 0}]},
                        {role: 'user', text: 'apaga os arquivos temporários'},
                        {role: 'assistant', text: 'Esse comando muda algo no seu computador. Confere e aperte **Rodar**.',
                            steps: [{titulo: 'Rodando comando', detalhe: 'rm -rf ~/.cache/thumbnails', estado: 'espera', segundos: 0}],
                            confirm: 'rm -rf ~/.cache/thumbnails'},
                    ];
                    this._renderChat();
                    await shot('ia-chat');
                    await this._openSide();
                    await shotSide('ia-lado');
                    this._closeSide();
                    this._chat = [];
                    this._selText = 'Bom dia! Podemos remarcar a reunião para amanhã às 10h?';
                    this._selFrom = 'selecionado';
                    this._enterView('texto');
                    await shot('texto');
                    this._convs = null;
                    this._enterView('conversas');
                    await shot('conversas');
                    this._enterView('emoji');
                    await shot('emoji');
                    this._entry.text = 'coração';
                    await shot('emoji-busca');
                    this._consumed = true;
                    this._close();
                    this._selfTesting = false;
                    console.log(`${TAG} fotos prontas`);
                })().catch(e => console.log(`${TAG} fotos: ${e.message}`));
                return GLib.SOURCE_REMOVE;
            });
        }
        // Autoteste da seta de voltar em cada tela.
        const probe5 = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-telas']);
        if (GLib.file_test(probe5, GLib.FileTest.EXISTS)) {
            GLib.unlink(probe5);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
                this._selfTesting = true;
                this.open('root');
                const seen = [`root: ${this._back.visible}`];
                for (const v of ['ia', 'historico', 'conversas']) {
                    this._enterView(v);
                    seen.push(`${v}: ${this._back.visible}`);
                }
                this._selText = 'teste';
                this._enterView('texto');
                seen.push(`texto: ${this._back.visible}`);
                this._goBack();
                seen.push(`voltou para: ${this._view} (seta ${this._back.visible})`);
                console.log(`${TAG} telas → ${seen.join(' · ')}`);
                this._consumed = true;
                this._close();
                this._selfTesting = false;
                return GLib.SOURCE_REMOVE;
            });
        }
        // Autoteste de soltar pela rolagem (não clica em nada).
        const probe4 = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-rolar']);
        if (GLib.file_test(probe4, GLib.FileTest.EXISTS)) {
            GLib.unlink(probe4);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
                (async () => {
                    const mods = () => global.get_pointer()[2];
                    const BTN = Clutter.ModifierType.BUTTON1_MASK | Clutter.ModifierType.BUTTON2_MASK | Clutter.ModifierType.BUTTON3_MASK;
                    console.log(`${TAG} rolar → botão preso antes: ${(mods() & BTN) !== 0}`);
                    this._selfTesting = true;
                    this.open('root');
                    await sleep(300);
                    const [ox, oy] = global.get_pointer();
                    const mon = this._mon;
                    const vp = virtualPointer();
                    vp.notify_absolute_motion(GLib.get_monotonic_time(), mon.x + 3, mon.y + Math.round(mon.height / 2));
                    await sleep(80);
                    vp.notify_discrete_scroll(GLib.get_monotonic_time(), Clutter.ScrollDirection.DOWN, Clutter.ScrollSource.WHEEL);
                    await sleep(200);
                    console.log(`${TAG} rolar → aberto: ${Boolean(this._panel)}, solto: ${!this._grab}, botão preso depois: ${(mods() & BTN) !== 0}`);
                    vp.notify_absolute_motion(GLib.get_monotonic_time(), ox, oy);
                    this._consumed = true;
                    this._close();
                    this._selfTesting = false;
                })().catch(e => console.log(`${TAG} rolar: falhou ${e.message}`));
                return GLib.SOURCE_REMOVE;
            });
        }
        // Autoteste das novidades (só para depurar).
        const probe2 = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-novidades']);
        if (GLib.file_test(probe2, GLib.FileTest.EXISTS)) {
            GLib.unlink(probe2);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
                this._selfTestNew().catch(e => {
                    console.log(`${TAG} novidades: falhou ${e.message}\n${e.stack}`);
                    this._close();
                });
                return GLib.SOURCE_REMOVE;
            });
        }
        // Jogos na NVIDIA: quando aparece atalho de jogo novo (Steam, Lutris…),
        // roda o clippit-jogos-nvidia para ele também pedir a placa dedicada.
        this._watchGames();
        // Alt+A: ações no texto selecionado (resumir, traduzir, corrigir…).
        bindShortcuts(this, [['acoes', () => this.openTextActions()]]);
        // Lembretes e timers marcados pela IA (ou "timer 25" no lançador).
        this._remindId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 15, () => {
            this._checkReminders();
            return GLib.SOURCE_CONTINUE;
        });
        this._checkReminders();
        // Autoteste (só para depurar): abre, testa algumas buscas e fecha.
        const probe = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-lancador']);
        if (GLib.file_test(probe, GLib.FileTest.EXISTS)) {
            GLib.unlink(probe);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                try {
                    this._selfTesting = true;
                    this.open();
                    for (const q of ['chr', 'slak', '15% de 340'])  {
                        this._entry.text = q;
                        console.log(`${TAG} lançador [${q}] → ${this._shown.slice(0, 3).map(i => `${i.section}: ${i.title}`).join(' | ')}`);
                    }
                    this._entry.text = '';
                    console.log(`${TAG} lançador [vazio] → ${this._shown.map(i => `${i.section}: ${i.title}`).join(' | ')}`);
                    this._entry.text = 'hist';
                    console.log(`${TAG} lançador [hist] → ${this._shown.slice(0, 2).map(i => `${i.section}: ${i.title}`).join(' | ')}`);
                    this._enterView('historico');
                    this._searchAsync('').then(() => {
                        this._paint();
                        console.log(`${TAG} lançador [cb] → ${this._shown.length} itens, prévia com ${this._detail.get_n_children()} partes, rodapé "${this._footAction.text}"`);
                        const img = this._shown.findIndex(i => i.data?.tipo === 'image');
                        if (img >= 0) {
                            this._sel = img;
                            this._paint();
                            console.log(`${TAG} lançador [imagem] → prévia com ${this._detail.get_n_children()} partes`);
                        }
                        this._toggleZoom();
                        console.log(`${TAG} lançador [ampliado] → lista visível: ${this._scroll.visible}, janela: ${this._panel.width}x${this._panel.height}`);
                        // Fechar sem querer e reabrir: o texto volta.
                        this._enterView('root');
                        this._entry.text = 'texto que eu estava digitando';
                        this._close();
                        this.open();
                        console.log(`${TAG} lançador [reaberto] → "${this._entry.text}" (tela ${this._view}, ${this._panel.width}x${this._panel.height})`);
                        this._entry.text = '';
                        this._consumed = true;
                        this._enterView('ia');
                        this._chat = [];
                        console.log(`${TAG} lançador [pausa] → corretor de trás pausado: ${Boolean(this._c._suspended)}`);
                        this._entry.text = 'abre o @calc';
                        console.log(`${TAG} lançador [@] → ${this._mention?.items.map(i => i.nome).join(' | ')} · lista: ${this._mentionBox.visible}`);
                        this._mentionPick();
                        console.log(`${TAG} lançador [@ escolhido] → "${this._entry.text}", anexos: ${JSON.stringify(this._attach)}`);
                        this._entry.text = '';
                        this._attach = [];
                        this._openSide().then(() => {
                            const [px] = this._panel.get_position();
                            const [sx] = this._side.get_position();
                            const textIdx = this._sideItems.findIndex(h => h.tipo !== 'image');
                            this._sideMove(textIdx - this._sideSel);
                            return this._sideUse(false).then(() => console.log(`${TAG} lançador [copiados] → ` +
                                `${this._sideRows.length || this._sideItems.length} itens, painel em x=${sx} (lançador x=${px}), ` +
                                `colou ${this._entry.text.length} caracteres, painel fechou: ${!this._side}`));
                        }).then(() => {
                        this._entry.text = 'quais processo estao masi consumindo recusos agora';
                        const checking = this._checkFix(this._entry.text);
                        console.log(`${TAG} lançador [verificando] → anel: ${this._fixRing.actor.visible}, "${this._fixLabel.text}", linha: ${this._fixRow.visible}`);
                        checking.then(() => {
                            console.log(`${TAG} lançador [correção] → "${this._fix?.to}" · visível: ${this._fixRow.visible}`);
                            const applied = this._applyFix();
                            console.log(`${TAG} lançador [aplicou] → ${applied}: "${this._entry.text}"`);
                            this._entry.text = '';
                            return this._send('quanto de memória estou usando?');
                        }).then(() => {
                            const last = this._chat.at(-1);
                            console.log(`${TAG} lançador [ia] → campo: "${this._entry.text}", mensagens: ${this._chat.length}, ` +
                                `blocos na tela: ${this._chatBox.get_n_children()}, passos: ${JSON.stringify((last.steps ?? []).map(p => p.titulo))}, ` +
                                `resposta: ${String(last.text).slice(0, 70)}`);
                            // Comando que muda algo: pede "Rodar"; depois mostra a saída.
                            const dir = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'teste-ia']);
                            return this._send(`crie a pasta ${dir}`).then(() => {
                                const ask = this._chat.at(-1);
                                console.log(`${TAG} lançador [confirmar] → ${ask.confirm} · rodapé "${this._footAction.text}"`);
                                return ask.confirm ? this._confirm(ask, true) : null;
                            }).then(() => GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
                                const waitDone = () => this._chat.some(m => m.pending);
                                if (waitDone())
                                    return GLib.SOURCE_CONTINUE;
                                const r = this._chat.at(-1);
                                console.log(`${TAG} lançador [rodou] → saídas: ${JSON.stringify(r.outputs)}, pasta existe: ` +
                                    `${GLib.file_test(dir, GLib.FileTest.IS_DIR)}, resposta: ${String(r.text).slice(0, 60)}`);
                                this._chat = [];
                                this._close();
                                this._selfTesting = false;
                                return GLib.SOURCE_REMOVE;
                            }));
                        });
                        });
                    }).catch(e => {
                        console.log(`${TAG} autoteste do lançador falhou: ${e.message}\n${e.stack}`);
                        this._close();
                    });
                } catch (e) {
                    console.log(`${TAG} autoteste do lançador falhou: ${e.message}\n${e.stack}`);
                    this._close();
                }
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    // Abre o lançador já numa tela; se ela já está aberta, fecha.
    toggleView(view) {
        if (view === 'ia' && !aiOn('chat')) {
            aiOffNotice('O chat com a IA');
            return;
        }
        if (this._panel && !this._grab) {
            // Estava solto: reativa (e troca de tela se for outra).
            this._wake();
            if (view === 'historico' && this._view !== 'historico')
                this._toggleSide();
            else if (view !== this._view && view !== 'root')
                this._enterView(view);
            return;
        }
        // Atalho do Clippit com o lançador aberto (fora do histórico): abre a
        // área de transferência ao lado, para colar algo na conversa.
        if (view === 'historico' && this._panel && this._view !== 'historico') {
            this._toggleSide();
            return;
        }
        if (this._panel && this._view === view) {
            this._close();
            return;
        }
        if (!this._panel)
            this.open(view);
        else if (view !== 'root')
            this._enterView(view);
    }

    disable() {
        this._close();
        this._gamesMon?.cancel();
        this._gamesMon = null;
        if (this._gamesId) {
            GLib.source_remove(this._gamesId);
            this._gamesId = 0;
        }
        if (this._remindId) {
            GLib.source_remove(this._remindId);
            this._remindId = 0;
        }
        try {
            this._dbus?.unexport();
        } catch (_e) {}
        if (this._nameId) {
            Gio.bus_unown_name(this._nameId);
            this._nameId = 0;
        }
        unbindShortcuts(this);
        for (const accel of [...this._grabs.keys()])
            this._setKey(accel, false);
        if (this._accelId) {
            global.display.disconnect(this._accelId);
            this._accelId = 0;
        }
    }

    _setKey(accel, want, fn) {
        const grab = this._grabs.get(accel);
        if (want && !grab) {
            const action = global.display.grab_accelerator(accel, Meta.KeyBindingFlags.NONE);
            if (action === Meta.KeyBindingAction.NONE) {
                console.log(`${TAG} lançador: não consegui reservar ${accel}`);
                return;
            }
            const name = Meta.external_binding_name_for_action(action);
            Main.wm.allowKeybinding(name, Shell.ActionMode.ALL);
            this._grabs.set(accel, {action, name, fn});
        } else if (!want && grab) {
            global.display.ungrab_accelerator(grab.action);
            Main.wm.allowKeybinding(grab.name, Shell.ActionMode.NONE);
            this._grabs.delete(accel);
        }
        if (!this._accelId && this._grabs.size) {
            this._accelId = global.display.connect('accelerator-activated', (_d, action) => {
                for (const g of this._grabs.values()) {
                    if (g.action === action)
                        g.fn();
                }
            });
        }
    }

    // ---- Uso (os mais usados aparecem primeiro) --------------------------

    _loadUsage() {
        try {
            return JSON.parse(new TextDecoder().decode(GLib.file_get_contents(USAGE_FILE)[1]));
        } catch (_e) {
            return {};
        }
    }

    _bump(key) {
        this._usage[key] = (this._usage[key] ?? 0) + 1;
        try {
            GLib.mkdir_with_parents(GLib.path_get_dirname(USAGE_FILE), 0o700);
            GLib.file_set_contents(USAGE_FILE, JSON.stringify(this._usage));
        } catch (_e) {}
    }

    _usageBoost(key) {
        return Math.min(30, (this._usage[key] ?? 0) * 3);
    }

    // ---- Janela ------------------------------------------------------------

    open(view = 'root') {
        if (this._panel)
            return;
        const saved = this._saved;
        this._saved = null;
        const fresh = saved && (GLib.get_monotonic_time() - saved.time) / 1e6 < KEEP_TEXT_SECONDS;
        this._prevWin = global.display.focus_window;
        this._prevApp = this._prevWin ? Shell.WindowTracker.get_default().get_window_app(this._prevWin) : null;
        this._sel = 0;
        this._gen = 0;
        this._view = 'root'; // root | historico | ia
        this._zoom = false;
        this._consumed = false;
        this._wasHistory = undefined;
        this._async = {files: [], history: []};
        this._btState = undefined;
        this._answer = null;
        this._apps = Shell.AppSystem.get_default().get_installed().filter(a => a.should_show());

        const panel = new St.BoxLayout({vertical: true, reactive: true, style: LSTYLE.panel});
        panel.set_size(LAUNCH_WIDTH, LAUNCH_HEIGHT);
        // Vidro: desfoca o que está atrás da janela (o fundo é semitransparente).
        try {
            panel.add_effect(new Shell.BlurEffect({radius: 24, brightness: 0.6, mode: Shell.BlurMode.BACKGROUND}));
        } catch (e) {
            console.log(`${TAG} lançador: sem desfoque (${e.message})`);
        }

        // Busca, sem caixa: o texto grande no topo.
        const header = new St.BoxLayout({style: LSTYLE.header});
        this._back = new St.Button({
            child: new St.Icon({gicon: lucide('chevron-left', 'g'), icon_size: 16,
                x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER}),
            style: LSTYLE.back, y_align: Clutter.ActorAlign.CENTER, visible: false, can_focus: false,
            reactive: true, track_hover: true,
        });
        this._back.connect('notify::hover', () => (this._back.style = this._back.hover ? LSTYLE.backHover : LSTYLE.back));
        this._back.connect('clicked', () => this._goBack());
        header.add_child(this._back);
        this._entry = new St.Entry({
            style: LSTYLE.entry, can_focus: true, x_expand: true,
            hint_text: 'Buscar apps, comandos e mais…',
        });
        header.add_child(this._entry);
        this._entry.hint_actor?.set_style?.(LSTYLE.entryHint);
        this._modeLabel = new St.Label({style: LSTYLE.mode, y_align: Clutter.ActorAlign.CENTER, visible: false});
        header.add_child(this._modeLabel);
        panel.add_child(header);
        // Correção do que você digitou (na tela da IA): sugestão + Alt+S/Tab.
        this._fixRow = new St.BoxLayout({style: LSTYLE.fixRow, visible: false, reactive: true, track_hover: true});
        this._fixIcon = new St.Icon({icon_name: 'tools-check-spelling-symbolic', icon_size: 14,
            style: `color: ${C.accent};`, y_align: Clutter.ActorAlign.CENTER});
        this._fixRow.add_child(this._fixIcon);
        this._fixRing = makeRing(14);
        this._fixRing.actor.y_align = Clutter.ActorAlign.CENTER;
        this._fixRow.add_child(this._fixRing.actor);
        this._fixLabel = new St.Label({style: LSTYLE.fixText, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
        this._fixLabel.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        this._fixRow.add_child(this._fixLabel);
        this._fixBtn = textButton('Corrigir  Alt+S', LSTYLE.runBtn, LSTYLE.runBtnHover, () => this._applyFix());
        this._fixRow.add_child(this._fixBtn);
        panel.add_child(this._fixRow);
        // "@": anexar um app ou arquivo à pergunta (lista sob o campo).
        this._mentionBox = new St.BoxLayout({vertical: true, style: LSTYLE.mention, visible: false});
        panel.add_child(this._mentionBox);
        // Categorias do seletor de emoji (só aparece nessa tela).
        this._emojiBar = new St.BoxLayout({style: LSTYLE.emojiBar, visible: false});
        panel.add_child(this._emojiBar);

        // Corpo: lista (e, no histórico, a prévia ao lado).
        const body = new St.BoxLayout({x_expand: true, y_expand: true});
        this._scroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: true, x_expand: true, y_expand: true,
        });
        this._list = new St.BoxLayout({vertical: true, style: LSTYLE.list});
        smoothScroll(this._scroll, () => this._list?.vadjustment);
        if (this._scroll.set_child)
            this._scroll.set_child(this._list);
        else
            this._scroll.add_actor(this._list);
        body.add_child(this._scroll);
        this._vsep = new St.Widget({style: LSTYLE.vsep, y_expand: true, visible: false});
        body.add_child(this._vsep);
        // Prévia do histórico, com rolagem (texto longo, imagem + informações).
        this._detailScroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: true, y_expand: true, visible: false,
        });
        this._detail = new St.BoxLayout({vertical: true, style: LSTYLE.detail});
        smoothScroll(this._detailScroll, () => this._detail?.vadjustment);
        if (this._detailScroll.set_child)
            this._detailScroll.set_child(this._detail);
        else
            this._detailScroll.add_actor(this._detail);
        body.add_child(this._detailScroll);

        // Resposta da IA ocupa o corpo inteiro.
        this._answerScroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: true, x_expand: true, y_expand: true, visible: false,
        });
        // Conversa com a IA (como um chat): suas perguntas e as respostas.
        // Caixa rolável por fora (_chatScrollBox) e o conteúdo por dentro: a de
        // fora pergunta a altura à de dentro, que responde a altura real.
        this._chatBox = new FullHeightBox({vertical: true, style: LSTYLE.chat});
        this._chatScrollBox = new St.BoxLayout({vertical: true});
        this._chatScrollBox.add_child(this._chatBox);
        smoothScroll(this._answerScroll, () => this._chatScrollBox?.vadjustment);
        if (this._answerScroll.set_child)
            this._answerScroll.set_child(this._chatScrollBox);
        else
            this._answerScroll.add_actor(this._chatScrollBox);
        body.add_child(this._answerScroll);
        // Chat vazio: fica fora da rolagem para centralizar na altura toda
        // (sem fixar altura nenhuma, que derruba o GNOME).
        this._chatEmpty = new St.Bin({x_expand: true, y_expand: true, visible: false,
            x_align: Clutter.ActorAlign.FILL, y_align: Clutter.ActorAlign.FILL});
        body.add_child(this._chatEmpty);
        panel.add_child(body);

        // Barra de ações (padrão): onde você está e o que cada tecla faz.
        this._foot = makeFooter();
        this._footAction = {get text() {
            return this._f.primary;
        }, _f: this._foot};
        panel.add_child(this._foot.actor);

        this._panel = panel;
        Main.layoutManager.addTopChrome(panel);
        // No monitor da janela em foco (sem janela: onde está o mouse), não no principal.
        const lm = Main.layoutManager;
        const idx = this._prevWin?.get_monitor() ?? -1;
        this._mon = (idx >= 0 ? lm.monitors[idx] : null) ?? lm.currentMonitor ?? lm.primaryMonitor;
        this._applySize();

        this._grab = Main.pushModal(panel, {actionMode: Shell.ActionMode.POPUP});
        this._c.setSuspended?.(true);
        this._entry.grab_key_focus();
        this._entry.clutter_text.connect('text-changed', () => this._onQuery());
        this._entry.clutter_text.connect('key-press-event', (_a, ev) => this._onKey(ev));
        // Enquanto o lançador segura a tela, o GNOME entrega TODOS os cliques à
        // janela dele (não ao palco): é aqui que se vê o clique fora.
        this._onCaptured = ev => {
            const type = ev.type();
            const inside = actor => {
                if (!actor)
                    return false;
                const [x, y] = ev.get_coords();
                const [ax, ay] = actor.get_transformed_position();
                const [aw, ah] = actor.get_transformed_size();
                return x >= ax && x <= ax + aw && y >= ay && y <= ay + ah;
            };
            if (!this._grab) {
                // Solto: o resto da tela funciona normal. Clicar no lançador
                // (ou no painel ao lado) volta a ativá-lo.
                if (type === Clutter.EventType.BUTTON_PRESS && (inside(panel) || inside(this._side))) {
                    this._wake();
                    if (inside(this._side) && this._sideEvent(ev))
                        return Clutter.EVENT_STOP;
                }
                return Clutter.EVENT_PROPAGATE;
            }
            if (this._side && this._sideEvent(ev))
                return Clutter.EVENT_STOP;
            if (type === Clutter.EventType.SCROLL && !inside(panel) && !inside(this._side)) {
                this._release(0); // rolar fora: solta (a próxima rolagem já vai para a janela)
                return Clutter.EVENT_STOP;
            }
            if (type !== Clutter.EventType.BUTTON_PRESS)
                return Clutter.EVENT_PROPAGATE;
            if (!inside(panel) && !inside(this._side)) {
                // Clique fora: solta o lançador (fica na tela) e o clique vai
                // para onde você clicou (aba, janela, botão).
                this._release(ev.get_button());
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        };
        panel.connect('captured-event', (_a, ev) => this._onCaptured(ev));
        if (fresh) {
            // Volta como estava: mesma tela, mesmo texto, cursor no fim.
            if (saved.view !== 'root')
                this._enterView(saved.view);
            this._entry.text = saved.text;
            this._entry.clutter_text.set_cursor_position(-1);
            this._entry.clutter_text.set_selection(-1, -1);
        } else {
            this._refresh();
        }
        if (view !== 'root' && this._view !== view)
            this._enterView(view);
    }

    // Solta o teclado e o mouse sem fechar: o lançador fica na tela, meio
    // transparente, e o clique que soltou é repetido onde você clicou.
    _release(button = 0) {
        if (!this._grab)
            return;
        Main.popModal(this._grab);
        this._grab = null;
        if (this._panel)
            this._panel.opacity = 200;
        if (this._side)
            this._side.opacity = 200;
        this._c.setSuspended?.(false); // o corretor volta para o app
        if (button) {
            // Repete o clique INTEIRO (aperta e solta) no mesmo dispositivo:
            // só apertar deixava o botão preso e travava a tela toda.
            GLib.idle_add(GLib.PRIORITY_HIGH_IDLE, () => {
                const vp = virtualPointer();
                vp.notify_button(GLib.get_monotonic_time(), button, Clutter.ButtonState.PRESSED);
                vp.notify_button(GLib.get_monotonic_time() + 1, button, Clutter.ButtonState.RELEASED);
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    // Volta a ativar o lançador solto (clique nele, ou o atalho de novo).
    _wake() {
        if (!this._panel || this._grab)
            return;
        this._prevWin = global.display.focus_window ?? this._prevWin; // a janela de trás pode ter mudado
        this._grab = Main.pushModal(this._panel, {actionMode: Shell.ActionMode.POPUP});
        this._panel.opacity = 255;
        if (this._side)
            this._side.opacity = 255;
        this._c.setSuspended?.(true);
        this._entry.grab_key_focus();
    }

    _applySize() {
        if (!this._panel)
            return;
        const [w, h] = VIEW_SIZE[this._isHistory() ? 'historico' : this._view] ?? VIEW_SIZE.root;
        const mon = this._mon ?? Main.layoutManager.primaryMonitor;
        const W = Math.min(w, mon.width - 40), H = Math.min(h, mon.height - 60);
        this._panel.set_size(W, H);
        // Textos que quebram linha dentro da rolagem precisam saber a largura.
        this._chatBox?.set_width(W - 2);
        this._panel.set_position(Math.round(mon.x + (mon.width - W) / 2), Math.round(mon.y + (mon.height - H) * 0.38));
        this._W = W;
        this._placeSide();
    }

    // Telas internas (como no Raycast): histórico e IA têm a sua própria busca.
    _isHistory(q = this._entry?.text?.trim() ?? '') {
        return this._view === 'historico' || /^cb(\s|$)/i.test(q);
    }

    // Seta de voltar (e Backspace no campo vazio): um passo para trás.
    _goBack() {
        if (this._view === 'texto' && this._tx) {
            this._tx = null; // resultado → lista de ações
            this._render();
        } else if (this._cbTx) {
            this._cbTx = null; // transformações → histórico
            this._refresh();
        } else {
            this._enterView('root');
        }
        this._entry.grab_key_focus();
    }

    _enterView(view) {
        this._view = view;
        if (this._back)
            this._back.visible = view !== 'root';
        if (this._emojiBar)
            this._emojiBar.visible = view === 'emoji';
        if (view === 'emoji')
            this._emojiSel = 0;
        this._zoom = false;
        this._answer = null;
        this._applySize();
        this._entry.hint_text = {
            root: 'Buscar apps, comandos e mais…',
            historico: 'Filtrar o que você copiou…',
            ia: 'Pergunte algo… (@ anexa app, arquivo ou print · Alt+V fala)',
            conversas: 'Procurar nas conversas…',
            texto: 'O que fazer com o texto? Escolha abaixo ou escreva…',
            emoji: 'Buscar emoji e símbolos…',
        }[view];
        this._sel = 0;
        if (this._entry.text)
            this._entry.text = ''; // dispara a busca de novo
        else
            this._onQuery();
    }

    async _selfTestNew() {
        const log = (k, v) => console.log(`${TAG} novidades [${k}] → ${v}`);
        this._selfTesting = true;
        this.open('ia');
        // 0. clique fora solta (sem fechar); atalho/clique reativa
        this._release(0);
        log('solto', `aberto: ${Boolean(this._panel)}, segura teclado: ${Boolean(this._grab)}, corretor do app ligado: ${!this._c._suspended}, opacidade: ${this._panel.opacity}`);
        this._wake();
        log('reativado', `segura teclado: ${Boolean(this._grab)}, corretor pausado: ${Boolean(this._c._suspended)}, opacidade: ${this._panel.opacity}`);
        this._chat = [];
        this._chatId = null;
        // 1. resposta ao vivo
        let maxLive = 0;
        const watch = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
            maxLive = Math.max(maxLive, this._liveShown?.length ?? 0);
            return GLib.SOURCE_CONTINUE;
        });
        await this._send('explique em duas frases o que é um buraco negro');
        GLib.source_remove(watch);
        log('ao vivo', `texto parcial visto na tela: ${maxLive} letras; final: ${this._chat.at(-1).text.length} letras`);
        // dúvida com opções
        this._chat = [];
        await this._send('o que tá ocupando tanto minha memória?');
        const ask = this._pendingChoice();
        log('dúvida', ask ? `"${ask.text}" · opções na tela: ${ask._rows?.length} (${ask.choice.opcoes.join(' | ')})` : `não perguntou: ${this._chat.at(-1).text.slice(0, 80)}`);
        if (ask) {
            this._moveChoice(ask, 1);
            const p = this._answerChoice(ask, ask.choiceSel);
            await sleep(300);
            while (this._chat.some(m => m.pending))
                await sleep(300);
            const r = this._chat.at(-1);
            log('escolheu B', `"${this._chat.at(-2).text}" → passos: ${JSON.stringify((r.steps ?? []).map(x => x.detalhe || x.titulo))} · ${r.text.slice(0, 90)}`);
        }
        // 8. conversa salva
        this._convs = null;
        log('conversas', `salvas: ${this._loadConvs().length}; última: "${this._convItems('')[0].title}"`);
        // 3. @print
        this._entry.text = '@pri';
        log('@print', `opções: ${this._mention?.items.map(i => i.nome).join(' | ')}`);
        await this._attachPrint(this._mention);
        const att = this._attach?.[0];
        log('@print anexado', `"${this._entry.text}" · ${att?.tipo} · arquivo existe: ${GLib.file_test(att?.caminho ?? '', GLib.FileTest.EXISTS)}`);
        this._entry.text = '';
        this._attach = [];
        // 2. Alt+A (texto selecionado)
        this._selText = 'Bom dia! Podemos remarcar a reunião para amanhã às 10h?';
        this._selFrom = 'selecionado';
        this._enterView('texto');
        log('alt+a', this._shown.map(i => i.title).join(' | '));
        await this._runTx('traduzir', 'Traduzir');
        log('alt+a traduzir', `"${this._tx.text}" · rodapé "${this._footAction.text}"`);
        this._tx = null;
        // 4. snippets
        const before = loadSnippets();
        saveSnippets([...before, {gatilho: ';testeclippit', nome: 'teste', texto: 'Hoje é {data}'}]);
        this._enterView('root');
        this._entry.text = ';testecl';
        log('snippets', `${this._shown.map(i => i.title).join(' | ')} · expandido: "${await expandSnippet('Hoje é {data}')}"`);
        saveSnippets(before);
        // 5–6. controles e janelas (só lista, sem executar)
        this._entry.text = 'modo escuro';
        log('controles', this._shown.slice(0, 2).map(i => `${i.title} (${i.subtitle})`).join(' | '));
        this._entry.text = 'volume 40';
        log('volume', this._shown[0]?.title);
        this._entry.text = 'metade esquerda';
        log('janelas', `${this._shown[0]?.title} · ${this._shown[0]?.subtitle}`);
        this._entry.text = 'timer 25 chá';
        log('timer', `${this._shown[0]?.title} · ${this._shown[0]?.subtitle}`);
        // 7. emoji
        this._entry.text = ':coração';
        log('emoji', this._shown.slice(0, 4).map(i => `${i.emoji} ${i.title}`).join(' | '));
        // 12–13. histórico: transformar e fixar
        this._enterView('historico');
        await this._searchAsync('');
        const h = this._async.history.find(x => x.tipo !== 'image' && x.texto);
        this._cbTx = h;
        this._refresh();
        log('transformar', `${this._shown.length} opções: ${this._shown.slice(0, 4).map(i => i.title).join(' | ')} · MAIÚSCULAS: "${TEXT_TRANSFORMS[0].fn(h.texto).slice(0, 30)}"`);
        this._cbTx = null;
        await this._togglePin(h);
        const pinned = this._async.history.find(x => x.fixado);
        log('fixar', `fixado no topo: ${this._async.history[0]?.fixado === true} (${pinned?.texto?.slice(0, 30)})`);
        if (pinned)
            await this._togglePin(pinned);
        log('desafixar', `fixados agora: ${this._async.history.filter(x => x.fixado).length}`);
        // painel ao lado com transformações
        this._enterView('ia');
        await this._openSide();
        this._sideTx = this._sideItems.find(x => x.tipo !== 'image');
        this._paintSide();
        log('alt+c transformar', `${this._sideRows.length} opções · rodapé "${this._sideFoot.text}"`);
        this._closeSide();
        // 11. ditado (sem whisper instalado: deve avisar)
        await this._toggleDictation();
        await sleep(1500);
        await this._stopDictation(true);
        log('ditado', `"${this._fixLabel.text}"`);
        this._chat = [];
        this._chatId = null;
        this._consumed = true;
        this._close();
        this._selfTesting = false;
    }

    // ---- Ditado (Alt+V) ------------------------------------------------------

    async _toggleDictation() {
        if (this._rec) {
            await this._stopDictation(true);
            return;
        }
        const dir = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'anexos']);
        GLib.mkdir_with_parents(dir, 0o700);
        const path = GLib.build_filenamev([dir, `voz-${Date.now()}.wav`]);
        let proc;
        try {
            proc = Gio.Subprocess.new(['pw-record', '--rate', '16000', '--channels', '1', '--format', 's16', path],
                Gio.SubprocessFlags.STDERR_SILENCE);
        } catch (e) {
            this._setFixState('error', `Não consegui usar o microfone: ${e.message}`);
            return;
        }
        const rec = this._rec = {proc, path, t0: GLib.get_monotonic_time()};
        this._setFixState('rec', 'Ouvindo… fale e aperte Alt+V para terminar');
        rec.tick = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
            if (this._rec !== rec)
                return GLib.SOURCE_REMOVE;
            const secs = Math.floor((GLib.get_monotonic_time() - rec.t0) / 1e6);
            if (this._fixLabel)
                this._fixLabel.text = `Ouvindo… ${secs} s · Alt+V para terminar`;
            if (secs >= 90) { // limite de segurança
                this._stopDictation(true);
                return GLib.SOURCE_REMOVE;
            }
            return GLib.SOURCE_CONTINUE;
        });
    }

    async _stopDictation(transcribe) {
        const rec = this._rec;
        if (!rec)
            return;
        this._rec = null;
        if (rec.tick)
            GLib.source_remove(rec.tick);
        try {
            rec.proc.send_signal(2); // SIGINT: o pw-record fecha o arquivo direito
            await new Promise(r => rec.proc.wait_async(null, () => r()));
        } catch (_e) {}
        if (!transcribe || !this._panel) {
            GLib.unlink(rec.path);
            return;
        }
        this._setFixState('checking', 'Transcrevendo…');
        try {
            const text = (await runProcess([CORRECTOR_CMD, '--transcrever', rec.path], null, 180)).trim();
            if (!this._panel)
                return;
            if (text) {
                const ct = this._entry.clutter_text;
                const cur = this._entry.text;
                const pos = ct.get_cursor_position() < 0 ? cur.length : ct.get_cursor_position();
                const piece = (pos > 0 && !/\s$/.test(cur.slice(0, pos)) ? ' ' : '') + text;
                ct.insert_text(piece, pos);
                this._setFixState(null);
            } else {
                this._setFixState('error', 'Não entendi nada. Tente de novo (Alt+V).');
            }
        } catch (e) {
            if (this._panel)
                this._setFixState('error', `Ditado: ${e.message.replace(/^.*?:\s*/, '')}`);
        } finally {
            GLib.unlink(rec.path);
        }
    }

    // ---- Transformar e fixar o que foi copiado ----------------------------

    _cbTxItems(q) {
        const h = this._cbTx;
        const text = h.tipo === 'image' ? (h.ocr || '').trim() : h.texto ?? '';
        const items = TEXT_TRANSFORMS
            .map(tf => ({tf, s: q ? matchScore(q, tf.title) : 100}))
            .filter(x => x.s >= 40)
            .map(({tf, s}) => ({
                id: `cbtx:${tf.id}`, data: h, icon: tf.icon, score: s, title: tf.title,
                subtitle: tf.ai ? 'IA local' : '', hint: 'Colar', keepOpen: true,
                run: () => this._applyCbTransform(tf, text),
                alt: () => this._applyCbTransform(tf, text, true),
            }));
        items.push({
            id: 'cbtx:pin', data: h, icon: 'view-pin-symbolic', score: 0,
            title: h.fixado ? 'Desafixar do topo' : 'Fixar no topo (não some com o tempo)', subtitle: 'Ctrl+P',
            hint: 'Aplicar', keepOpen: true, run: () => this._togglePin(h),
        });
        return items;
    }

    async _applyCbTransform(tf, text, copyOnly = false) {
        if (!text) {
            Main.notify('Clippit', 'Esse item não tem texto para transformar.');
            return;
        }
        this._cbTx = null;
        if (tf.ai) {
            // Corrigir/traduzir/resumir: mostra o resultado antes de colar.
            this._selText = text;
            this._selFrom = 'copiado';
            this._enterView('texto');
            this._runTx(tf.ai, tf.title);
            return;
        }
        let out;
        try {
            out = tf.fn(text);
        } catch (e) {
            Main.notify('Clippit', `Não deu para ${tf.title.toLowerCase()}: ${e.message}`);
            this._refresh();
            return;
        }
        if (copyOnly) {
            St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, out);
            this._consumed = true;
            this._close();
            Main.notify('Clippit', 'Copiado.');
        } else {
            this._pasteText(out);
        }
    }

    async _togglePin(h) {
        try {
            await runProcess([BUSCA_CMD, h.fixado ? 'desafixar' : 'fixar', String(h.id)], null, 5);
        } catch (e) {
            Main.notify('Clippit', `Não consegui fixar: ${e.message}`);
            return;
        }
        this._cbTx = null;
        if (this._side) {
            this._closeSide();
            this._openSide();
        }
        if (this._panel && this._isHistory()) {
            const q = this._entry.text.trim();
            try {
                this._async.history = JSON.parse(await runProcess([BUSCA_CMD, 'historico',
                    this._view === 'historico' ? q : q.replace(/^cb\s*/i, '')], null, 5));
            } catch (_e) {}
            this._refresh();
        }
    }

    // ---- Colar no app de trás ----------------------------------------------

    async _pasteText(text) {
        this._consumed = true;
        if (this._panel)
            this._close();
        const win = this._prevWin;
        if (win && global.display.focus_window !== win)
            win.activate(global.get_current_time());
        await sleep(250);
        await pasteViaClipboard(text, false);
    }

    // ---- Alt+A: ações no texto selecionado --------------------------------

    async openTextActions(fromLauncher = false) {
        if (!aiOn('acoes')) {
            aiOffNotice('As ações no texto');
            return;
        }
        // O texto selecionado (seleção primária); sem seleção, o que está copiado.
        let text = (await clipboardText(St.ClipboardType.PRIMARY)).trim();
        let from = 'selecionado';
        if (!text || fromLauncher) {
            const copied = (await clipboardText()).trim();
            if (!text || (fromLauncher && copied)) {
                text = copied;
                from = 'copiado';
            }
        }
        if (!text) {
            Main.notify('Clippit', 'Selecione (ou copie) um texto e aperte Alt+A.');
            return;
        }
        if (this._panel && !fromLauncher)
            this._close();
        this._selText = text;
        this._selFrom = from;
        this._tx = null;
        if (!this._panel)
            this.open('texto');
        else
            this._enterView('texto');
    }

    // Comandos de IA somem do lançador quando a função está desligada.
    _aiFilter(items) {
        const need = {'cmd:ia': 'chat', 'cmd:conversas': 'chat', 'cmd:acoes': 'acoes'};
        return items.filter(it => !need[it.id] || aiOn(need[it.id]));
    }

    _textActionItems(q) {
        if (!this._selText)
            return [{id: 'tx-vazio', title: 'Selecione um texto em qualquer app e aperte Alt+A', subtitle: '',
                icon: 'edit-select-all-symbolic', score: 0, keepOpen: true, run: () => {}}];
        const preview = this._selText.replace(/\s+/g, ' ').slice(0, 70);
        const section = `Texto ${this._selFrom}: “${preview}${this._selText.length > 70 ? '…' : ''}”`;
        const items = TEXT_ACTIONS
            .map(a => ({a, s: q ? Math.max(matchScore(q, a.title), ...a.words.map(w => matchScore(q, w) * 0.8)) : 100}))
            .filter(x => x.s >= 40)
            .map(({a, s}) => ({
                id: `tx:${a.acao}`, section, icon: a.icon, score: s, title: a.title, subtitle: '', hint: 'Fazer',
                keepOpen: true, run: () => this._runTx(a.acao, a.title),
            }));
        if (q)
            items.unshift({
                id: 'tx:livre', section, icon: 'document-edit-symbolic', score: 1000, title: `Fazer: ${q}`,
                subtitle: 'Pedido livre sobre o texto', hint: 'Fazer', keepOpen: true,
                run: () => this._runTx('', q, q),
            });
        return items;
    }

    async _runTx(acao, title, instrucao = '') {
        if (acao === 'perguntar') {
            // Vai para o chat com o texto anexado.
            const text = this._selText;
            this._enterView('ia');
            this._attachToken(null, {tipo: 'texto', nome: 'seleção', texto: text});
            return;
        }
        const tx = this._tx = {title, pending: true, text: '', live: ''};
        this._entry.text = '';
        this._render();
        const statusFile = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-correct', 'agente-status']);
        try {
            GLib.file_set_contents(statusFile, '');
        } catch (_e) {}
        const pollId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
            if (this._tx !== tx || !tx.pending)
                return GLib.SOURCE_REMOVE;
            try {
                const st = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(statusFile)[1]).trim() || '{}');
                if ((st.texto ?? '') !== tx.live && this._txLabel) {
                    tx.live = st.texto ?? '';
                    this._txLabel.clutter_text.set_markup(mdToMarkup(tx.live));
                }
            } catch (_e) {}
            return GLib.SOURCE_CONTINUE;
        });
        try {
            const out = await runProcess([CORRECTOR_CMD, '--transformar'],
                JSON.stringify({acao, texto: this._selText, instrucao}), 240);
            tx.text = out.trim();
        } catch (e) {
            tx.error = e.message;
        } finally {
            tx.pending = false;
            GLib.source_remove(pollId);
        }
        if (this._panel && this._tx === tx)
            this._render();
    }

    _renderTx() {
        const tx = this._tx;
        this._scroll.hide();
        this._detailScroll.hide();
        this._vsep.hide();
        this._answerScroll.show();
        this._chatEmpty.hide();
        this._back.visible = true;
        this._modeLabel.text = 'Texto selecionado';
        this._chatBox.destroy_all_children();
        this._stepsBox = null;
        this._liveText = null;
        const head = new St.BoxLayout({style: 'spacing: 8px;'});
        if (tx.pending) {
            const ring = makeRing(14);
            ring.start();
            ring.actor.y_align = Clutter.ActorAlign.CENTER;
            head.add_child(ring.actor);
        }
        head.add_child(new St.Label({text: tx.pending ? `${tx.title}…` : tx.title, style: LSTYLE.stepTitle,
            y_align: Clutter.ActorAlign.CENTER}));
        this._chatBox.add_child(head);
        this._txLabel = new St.Label({style: LSTYLE.aiText});
        this._txLabel.clutter_text.line_wrap = true;
        this._txLabel.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        const shown = tx.error ? `Não consegui: ${tx.error}` : tx.pending ? tx.live : tx.text;
        if (shown)
            this._txLabel.clutter_text.set_markup(tx.error ? esc(shown) : mdToMarkup(shown));
        this._chatBox.add_child(this._txLabel);
        this._setFooter('Texto selecionado', 'edit-select-all-symbolic', tx.pending ? [{label: 'Escrevendo…'}]
            : [{label: 'Colar no lugar', key: '↵'}, {label: 'Copiar', key: 'Ctrl ↵'}, {label: 'Voltar', key: '⌫'}]);
    }

    // ---- Snippets (;gatilho) ------------------------------------------------

    _snippetItems(q) {
        const items = [];
        const make = q.match(/^;([^\s=]+)\s*=\s*([\s\S]+)$/);
        if (make) {
            const gat = `;${make[1]}`;
            items.push({
                id: 'snip:novo', section: 'Snippets', icon: 'list-add-symbolic', score: 1000,
                title: `Criar snippet ${gat}`, subtitle: make[2].slice(0, 80), hint: 'Criar', keepOpen: true,
                run: () => {
                    const list = loadSnippets().filter(x => x.gatilho !== gat);
                    list.push({gatilho: gat, nome: make[1], texto: make[2].replaceAll('\\n', '\n')});
                    saveSnippets(list);
                    Main.notify('Clippit', `Snippet ${gat} criado. Digite ${gat} e espaço em qualquer campo.`);
                    this._entry.text = ';';
                },
            });
            return items;
        }
        const term = q.slice(1);
        for (const sn of loadSnippets()) {
            const sc = term ? Math.max(matchScore(term, sn.gatilho.slice(1)), matchScore(term, sn.nome ?? '') * 0.9,
                matchScore(term, sn.texto) * 0.5) : 100;
            if (sc < 40)
                continue;
            items.push({
                id: `snip:${sn.gatilho}`, section: 'Snippets', icon: 'insert-text-symbolic', score: sc,
                title: `${sn.gatilho}  ${sn.nome ?? ''}`, subtitle: sn.texto.replace(/\s+/g, ' ').slice(0, 90), hint: 'Colar',
                run: async () => this._pasteText(await expandSnippet(sn.texto)),
                alt: async () => {
                    St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, await expandSnippet(sn.texto));
                    Main.notify('Clippit', 'Snippet copiado.');
                },
            });
        }
        items.sort((a, b) => b.score - a.score);
        items.push({
            id: 'snip:editar', section: 'Snippets', icon: 'document-edit-symbolic', score: 0,
            title: 'Editar snippets', subtitle: 'Abre o arquivo · ou digite ;gatilho = texto', hint: 'Abrir',
            run: () => {
                if (!GLib.file_test(SNIPPETS_FILE, GLib.FileTest.EXISTS))
                    saveSnippets([{gatilho: ';data', nome: 'Data de hoje', texto: '{data}'},
                        {gatilho: ';obg', nome: 'Agradecimento', texto: 'Muito obrigado! Qualquer coisa, estou à disposição.'}]);
                Gio.AppInfo.launch_default_for_uri(`file://${SNIPPETS_FILE}`, global.create_app_launch_context(0, -1));
            },
        });
        return items;
    }

    // ---- Seletor de emoji em grade (":" ou "Emoji e símbolos") ----------------

    _emojiData() {
        if (this._emoji2)
            return this._emoji2;
        const cache = GLib.build_filenamev([GLib.get_user_cache_dir(), 'clippit', 'emoji-v2.json']);
        try {
            const list = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(cache)[1]));
            this._emoji2 = list.map(x => ({e: x.e, desc: x.d, cat: x.c, words: x.w, all: fold(x.w.join(' | '))}));
        } catch (_e) {
            if (!this._emojiLoading) {
                this._emojiLoading = true;
                runProcess([BUSCA_CMD, 'emoji'], null, 30).then(() => {
                    this._emojiLoading = false;
                    this._render();
                }).catch(e => console.log(`${TAG} emoji: ${e.message}`));
            }
            return null;
        }
        return this._emoji2;
    }

    _emojiPrefsFile() {
        return GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'emoji.json']);
    }

    _emojiPrefs() {
        if (!this._ePrefs) {
            try {
                this._ePrefs = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(this._emojiPrefsFile())[1]));
            } catch (_e) {
                this._ePrefs = {fixados: ['❤️', '👋', '👍', '🚀', '🎉', '🙏'], recentes: []};
            }
        }
        return this._ePrefs;
    }

    _saveEmojiPrefs() {
        try {
            GLib.file_set_contents(this._emojiPrefsFile(), JSON.stringify(this._ePrefs));
        } catch (_e) {}
    }

    // Seções da grade: [{title, items}] conforme busca e categoria.
    _emojiSections(q) {
        const data = this._emojiData();
        if (!data)
            return null;
        const byE = new Map(data.map(x => [x.e, x]));
        const prefs = this._emojiPrefs();
        const term = fold(q.trim());
        if (term) {
            const first = term.split(/\s+/)[0];
            const found = data.filter(x => x.all.includes(first))
                .map(x => ({x, s: Math.max(...x.words.map(w => matchScore(q.trim(), w))) + matchScore(q.trim(), x.desc) * 0.3 -
                    x.desc.length * 0.1 + (POPULAR_EMOJI.includes(x.e) ? 15 : 0)}))
                .filter(y => y.s >= 45).sort((a, b) => b.s - a.s).slice(0, 96).map(y => y.x);
            return [{title: found.length ? 'Resultados' : 'Nenhum emoji com esse nome', items: found}];
        }
        const pick = list => list.map(e => byE.get(e) ?? {e, desc: e, words: []}).filter(Boolean);
        const sections = [];
        const cat = this._emojiCat ?? 'Sorrisos e emoções';
        if (cat === 'Fixados' || !this._emojiCat) {
            sections.push({title: 'Fixados', items: pick(prefs.fixados)});
            if (prefs.recentes.length)
                sections.push({title: 'Recentes', items: pick(prefs.recentes.slice(0, 16))});
        }
        if (cat !== 'Fixados')
            sections.push({title: cat, items: data.filter(x => x.cat === cat)});
        return sections;
    }

    _emojiCats() {
        return ['Fixados', 'Sorrisos e emoções', 'Pessoas', 'Animais e natureza', 'Comida e bebida', 'Viagens e lugares',
            'Atividades', 'Objetos', 'Símbolos', 'Bandeiras'];
    }

    _paintEmojiBar() {
        const bar = this._emojiBar;
        bar.destroy_all_children();
        const cur = this._emojiCat ?? 'Sorrisos e emoções';
        const searching = Boolean(this._entry.text.trim());
        for (const c of this._emojiCats()) {
            const on = !searching && c === cur;
            const b = new St.Button({label: c, style: on ? LSTYLE.chipOn : LSTYLE.chip, can_focus: false, reactive: true});
            b.connect('clicked', () => {
                this._emojiCat = c;
                this._emojiSel = 0;
                this._entry.text = '';
                this._render();
            });
            bar.add_child(b);
        }
    }

    _renderEmoji() {
        this._scroll.hide();
        this._detailScroll.hide();
        this._vsep.hide();
        this._answerScroll.show();
        this._chatEmpty.hide();
        this._emojiBar.show();
        this._paintEmojiBar();
        const box = this._chatBox;
        box.destroy_all_children();
        this._stepsBox = null;
        this._liveText = null;
        const gadj = this._chatScrollBox?.vadjustment;
        gadj?.remove_transition?.('value');
        if (gadj)
            gadj.value = 0; // a grade sempre começa no topo
        const sections = this._emojiSections(this._entry.text);
        if (!sections) {
            box.add_child(new St.Label({text: 'Carregando os emoji…', style: `${TX.small} padding: 12px 0;`}));
            return;
        }
        // Colunas pela largura da janela; quadradinhos ocupam a linha toda.
        const avail = (this._W ?? LAUNCH_WIDTH) - 2 - 2 * T.pad;
        const gap = 8;
        const cols = Math.max(6, Math.floor((avail + gap) / (72 + gap)));
        const size = Math.floor((avail - gap * (cols - 1)) / cols);
        this._emojiFlat = [];
        this._emojiRows = [];
        const grid = new St.BoxLayout({vertical: true, style: `spacing: 8px;`});
        for (const sec of sections) {
            grid.add_child(new St.Label({text: sec.title, style: LSTYLE.emojiSection}));
            for (let i = 0; i < sec.items.length; i += cols) {
                const row = new St.BoxLayout({style: LSTYLE.emojiRow});
                const rowIdx = [];
                for (const x of sec.items.slice(i, i + cols)) {
                    const idx = this._emojiFlat.length;
                    const tile = new St.Button({style: LSTYLE.tile, reactive: true, can_focus: false, track_hover: true,
                        width: size, height: Math.min(size, 76)});
                    tile.set_child(new St.Label({text: x.e, style: LSTYLE.tileGlyph,
                        x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER}));
                    tile.connect('notify::hover', () => tile.hover && this._emojiSelect(idx, false));
                    tile.connect('clicked', () => {
                        this._emojiSelect(idx, false);
                        this._emojiUse(false);
                    });
                    row.add_child(tile);
                    this._emojiFlat.push({x, tile, row: this._emojiRows.length, col: rowIdx.length});
                    rowIdx.push(idx);
                }
                this._emojiRows.push(rowIdx);
                grid.add_child(row);
            }
        }
        box.add_child(grid);
        this._emojiSel = Math.min(this._emojiSel ?? 0, Math.max(0, this._emojiFlat.length - 1));
        this._emojiSelect(this._emojiSel, true);
    }

    _emojiSelect(i, scroll) {
        const flat = this._emojiFlat ?? [];
        if (!flat.length) {
            this._setFooter('Emoji', 'face-smile-symbolic', []);
            return;
        }
        flat[this._emojiSel]?.tile.set_style(LSTYLE.tile);
        this._emojiSel = Math.max(0, Math.min(flat.length - 1, i));
        const cur = flat[this._emojiSel];
        cur.tile.set_style(LSTYLE.tileOn);
        const pinned = this._emojiPrefs().fixados.includes(cur.x.e);
        this._setFooter(`${cur.x.e}  ${cur.x.desc}`, 'face-smile-symbolic', [{label: 'Colar', key: '↵'},
            {label: 'Copiar', key: 'Ctrl ↵'}, {label: pinned ? 'Desafixar' : 'Fixar', key: 'Ctrl P'}]);
        if (scroll) {
            GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
                const adj = this._chatScrollBox?.vadjustment;
                const t = flat[this._emojiSel]?.tile;
                if (adj && t?.get_parent()) {
                    const [, ty] = t.get_transformed_position();
                    const [, by] = this._chatScrollBox.get_transformed_position();
                    const y = ty - by + adj.value;
                    if (y < adj.value + 30)
                        adj.ease(Math.max(0, y - 40), {duration: 140, mode: Clutter.AnimationMode.EASE_OUT_CUBIC});
                    else if (y + t.height > adj.value + adj.page_size - 10)
                        adj.ease(y + t.height - adj.page_size + 20, {duration: 140, mode: Clutter.AnimationMode.EASE_OUT_CUBIC});
                }
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    _emojiKey(sym, ev) {
        const ctrl = (ev.get_state() & Clutter.ModifierType.CONTROL_MASK) !== 0;
        const shift = (ev.get_state() & Clutter.ModifierType.SHIFT_MASK) !== 0;
        const flat = this._emojiFlat ?? [];
        const cur = flat[this._emojiSel];
        const moveRow = d => {
            if (!cur)
                return;
            const r = Math.max(0, Math.min(this._emojiRows.length - 1, cur.row + d));
            const row = this._emojiRows[r];
            this._emojiSelect(row[Math.min(cur.col, row.length - 1)], true);
        };
        switch (sym) {
        case Clutter.KEY_Left:
            this._emojiSelect(this._emojiSel - 1, true);
            return true;
        case Clutter.KEY_Right:
            this._emojiSelect(this._emojiSel + 1, true);
            return true;
        case Clutter.KEY_Up:
            moveRow(-1);
            return true;
        case Clutter.KEY_Down:
            moveRow(1);
            return true;
        case Clutter.KEY_Return:
        case Clutter.KEY_KP_Enter:
        case Clutter.KEY_ISO_Enter:
            this._emojiUse(ctrl);
            return true;
        case Clutter.KEY_Tab:
        case Clutter.KEY_ISO_Left_Tab: {
            const cats = this._emojiCats();
            const i = cats.indexOf(this._emojiCat ?? 'Sorrisos e emoções');
            this._emojiCat = cats[(i + (shift || sym === Clutter.KEY_ISO_Left_Tab ? cats.length - 1 : 1)) % cats.length];
            this._emojiSel = 0;
            this._entry.text = '';
            this._render();
            return true;
        }
        case Clutter.KEY_p:
        case Clutter.KEY_P:
            if (!ctrl || !cur)
                return false;
            {
                const p = this._emojiPrefs();
                p.fixados = p.fixados.includes(cur.x.e) ? p.fixados.filter(e => e !== cur.x.e) : [...p.fixados, cur.x.e];
                this._saveEmojiPrefs();
                this._render();
            }
            return true;
        }
        return false;
    }

    _emojiUse(copyOnly) {
        const cur = this._emojiFlat?.[this._emojiSel];
        if (!cur)
            return;
        const p = this._emojiPrefs();
        p.recentes = [cur.x.e, ...p.recentes.filter(e => e !== cur.x.e)].slice(0, 24);
        this._saveEmojiPrefs();
        if (copyOnly) {
            St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, cur.x.e);
            this._consumed = true;
            this._close();
            Main.notify('Clippit', `${cur.x.e} copiado.`);
        } else {
            this._pasteText(cur.x.e);
        }
    }

    // ---- Emoji (:nome) ------------------------------------------------------

    _emojiItems(q) {
        if (!this._emoji) {
            // Lista gerada pelo clippit-busca (IBus, nomes em português) e guardada em cache.
            const cache = GLib.build_filenamev([GLib.get_user_cache_dir(), 'clippit', 'emoji.json']);
            try {
                const list = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(cache)[1]));
                this._emoji = list.map(x => ({e: x.e, desc: x.d, words: x.w, all: fold(x.w.join(' | '))}));
            } catch (_e) {
                if (!this._emojiLoading) {
                    this._emojiLoading = true;
                    runProcess([BUSCA_CMD, 'emoji'], null, 30).then(() => {
                        this._emojiLoading = false;
                        this._refresh();
                    }).catch(e => console.log(`${TAG} emoji: ${e.message}`));
                }
                return [{id: 'emoji-carregando', section: 'Emoji', icon: 'face-smile-symbolic', score: 0,
                    title: 'Carregando os emoji…', subtitle: '', keepOpen: true, run: () => {}}];
            }
        }
        const term = q.trim();
        const ft = fold(term);
        const scored = this._emoji
            .filter(x => ft && x.all.includes(ft.split(/\s+/)[0]))
            // Desempate: o nome principal parecido e curto ganha (:coração → ❤️).
            .map(x => ({x, s: Math.max(...x.words.map(w => matchScore(term, w))) + matchScore(term, x.desc) * 0.3 -
                x.desc.length * 0.1 + (POPULAR_EMOJI.includes(x.e) ? 15 : 0)}))
            .filter(y => y.s >= 45)
            .sort((a, b) => b.s - a.s)
            .slice(0, 40);
        if (!scored.length)
            return [{id: 'emoji-vazio', section: 'Emoji', icon: 'face-smile-symbolic', score: 0,
                title: term ? 'Nenhum emoji com esse nome' : 'Digite o nome: :coração, :fogo, :ok', subtitle: '', keepOpen: true,
                run: () => {}}];
        return scored.map(({x, s}) => ({
            id: `emoji:${x.e}`, section: 'Emoji', emoji: x.e, score: s, title: x.desc,
            subtitle: [...new Set(x.words)].slice(1, 5).join(', '), hint: 'Colar',
            run: () => this._pasteText(x.e),
            alt: () => {
                St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, x.e);
                Main.notify('Clippit', `${x.e} copiado.`);
            },
        }));
    }

    // ---- Atalhos com número ------------------------------------------------

    _quickNumberItems(q) {
        const items = [];
        let m = q.match(/^timer\s+(\d+(?:[.,]\d+)?)\s*(min|m|minutos?)?\s*(.*)$/i);
        if (m) {
            const min = parseFloat(m[1].replace(',', '.'));
            const label = m[3]?.trim() || `Timer de ${m[1]} min`;
            items.push({id: 'timer', section: 'Comandos', icon: 'alarm-symbolic', score: 990,
                title: `Timer de ${m[1]} min`, subtitle: m[3]?.trim() || 'Avisa com notificação e som', hint: 'Marcar',
                run: () => {
                    this.addReminder(min, label);
                    Main.notify('Clippit', `Timer de ${m[1]} min marcado.`);
                }});
        }
        m = q.match(/^vol(?:ume)?\s+(\d{1,3})\s*%?$/i);
        if (m) {
            const v = Math.min(150, +m[1]);
            items.push({id: 'volume', section: 'Comandos', icon: 'audio-volume-high-symbolic', score: 990,
                title: `Volume em ${v}%`, subtitle: 'Saída de som padrão', hint: 'Aplicar',
                run: () => Gio.Subprocess.new(['wpctl', 'set-volume', '-l', '1.5', '@DEFAULT_AUDIO_SINK@', `${v}%`], Gio.SubprocessFlags.NONE)});
        }
        m = q.match(/^brilho\s+(\d{1,3})\s*%?$/i);
        if (m) {
            const v = Math.max(1, Math.min(100, +m[1]));
            items.push({id: 'brilho', section: 'Comandos', icon: 'display-brightness-symbolic', score: 990,
                title: `Brilho em ${v}%`, subtitle: 'Tela do notebook', hint: 'Aplicar',
                run: () => this._setBrightness(v)});
        }
        return items;
    }

    // ---- Controles do sistema ---------------------------------------------

    _dbusGet(name, path, iface, prop) {
        try {
            const r = Gio.DBus.session.call_sync(name, path, 'org.freedesktop.DBus.Properties', 'Get',
                new GLib.Variant('(ss)', [iface, prop]), null, Gio.DBusCallFlags.NONE, 500, null);
            return r.deepUnpack()[0].deepUnpack();
        } catch (_e) {
            return null;
        }
    }

    _dbusSet(name, path, iface, prop, variant) {
        Gio.DBus.session.call(name, path, 'org.freedesktop.DBus.Properties', 'Set',
            new GLib.Variant('(ssv)', [iface, prop, variant]), null, Gio.DBusCallFlags.NONE, 2000, null, null);
    }

    _setBrightness(v) {
        this._dbusSet('org.gnome.SettingsDaemon.Power', '/org/gnome/SettingsDaemon/Power',
            'org.gnome.SettingsDaemon.Power.Screen', 'Brightness', new GLib.Variant('i', v));
    }

    _controlCommands() {
        const iface = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
        const notif = new Gio.Settings({schema_id: 'org.gnome.desktop.notifications'});
        const dark = iface.get_string('color-scheme') === 'prefer-dark';
        const dnd = !notif.get_boolean('show-banners');
        // Lido uma vez por abertura (a busca roda a cada letra).
        if (this._btState === undefined)
            this._btState = this._dbusGet('org.gnome.SettingsDaemon.Rfkill', '/org/gnome/SettingsDaemon/Rfkill',
                'org.gnome.SettingsDaemon.Rfkill', 'BluetoothAirplaneMode');
        const btOff = this._btState;
        const sh = cmd => () => Gio.Subprocess.new(['sh', '-c', cmd], Gio.SubprocessFlags.NONE);
        const step = (method) => () => Gio.DBus.session.call('org.gnome.SettingsDaemon.Power', '/org/gnome/SettingsDaemon/Power',
            'org.gnome.SettingsDaemon.Power.Screen', method, null, null, Gio.DBusCallFlags.NONE, 2000, null, null);
        return [
            {id: 'ctl:wifi', key: 'ctl:wifi', icon: 'network-wireless-symbolic', title: 'Wi-Fi (liga/desliga)', subtitle: 'Sistema',
                words: ['wifi', 'wi-fi', 'internet', 'rede sem fio'], hint: 'Alternar',
                run: sh('if [ "$(nmcli radio wifi)" = enabled ]; then nmcli radio wifi off; else nmcli radio wifi on; fi')},
            {id: 'ctl:bt', key: 'ctl:bt', icon: 'bluetooth-active-symbolic',
                title: btOff === null ? 'Bluetooth (liga/desliga)' : btOff ? 'Ligar Bluetooth' : 'Desligar Bluetooth',
                subtitle: btOff === null ? 'Sistema' : btOff ? 'Está desligado' : 'Está ligado', words: ['bluetooth', 'bt', 'fone'],
                hint: 'Alternar', run: () => this._dbusSet('org.gnome.SettingsDaemon.Rfkill', '/org/gnome/SettingsDaemon/Rfkill',
                    'org.gnome.SettingsDaemon.Rfkill', 'BluetoothAirplaneMode', new GLib.Variant('b', !btOff))},
            {id: 'ctl:dark', key: 'ctl:dark', icon: 'weather-clear-night-symbolic', title: dark ? 'Usar tema claro' : 'Usar modo escuro',
                subtitle: dark ? 'Modo escuro está ligado' : 'Modo escuro está desligado', words: ['modo escuro', 'tema', 'dark', 'claro'],
                hint: 'Alternar', run: () => iface.set_string('color-scheme', dark ? 'default' : 'prefer-dark')},
            {id: 'ctl:dnd', key: 'ctl:dnd', icon: 'notifications-disabled-symbolic', title: dnd ? 'Desligar o Não perturbe' : 'Ligar o Não perturbe',
                subtitle: dnd ? 'Notificações escondidas' : 'Notificações aparecendo', words: ['nao perturbe', 'notificacoes', 'silencio', 'foco'],
                hint: 'Alternar', run: () => notif.set_boolean('show-banners', dnd)},
            {id: 'ctl:mute', key: 'ctl:mute', icon: 'audio-volume-muted-symbolic', title: 'Som mudo (liga/desliga)', subtitle: 'Saída de som',
                words: ['mudo', 'mute', 'silenciar', 'som'], hint: 'Alternar', run: sh('wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle')},
            {id: 'ctl:volup', key: 'ctl:volup', icon: 'audio-volume-high-symbolic', title: 'Aumentar o volume', subtitle: '+10% · ou "volume 50"',
                words: ['volume', 'mais alto', 'aumentar som'], hint: 'Aplicar', run: sh('wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 10%+')},
            {id: 'ctl:voldown', key: 'ctl:voldown', icon: 'audio-volume-low-symbolic', title: 'Diminuir o volume', subtitle: '−10%',
                words: ['volume', 'mais baixo', 'abaixar som'], hint: 'Aplicar', run: sh('wpctl set-volume @DEFAULT_AUDIO_SINK@ 10%-')},
            {id: 'ctl:brup', key: 'ctl:brup', icon: 'display-brightness-symbolic', title: 'Aumentar o brilho', subtitle: 'ou "brilho 70"',
                words: ['brilho', 'tela mais clara'], hint: 'Aplicar', run: step('StepUp')},
            {id: 'ctl:brdown', key: 'ctl:brdown', icon: 'display-brightness-symbolic', title: 'Diminuir o brilho', subtitle: 'Tela',
                words: ['brilho', 'tela mais escura'], hint: 'Aplicar', run: step('StepDown')},
        ];
    }

    // ---- Janelas (a que estava em foco antes do lançador) -----------------

    _windowCommands() {
        const w = this._prevWin;
        if (!w)
            return [];
        const name = w.get_title() ?? 'janela';
        const mk = (op, title, icon, words) => ({
            id: `win:${op}`, key: `win:${op}`, icon, title, subtitle: name, words: ['janela', 'window', ...words],
            hint: 'Aplicar', run: () => this._winOp(w, op),
        });
        return [
            mk('left', 'Janela na metade esquerda', 'view-dual-symbolic', ['esquerda', 'left', 'lado']),
            mk('right', 'Janela na metade direita', 'view-dual-symbolic', ['direita', 'right', 'lado']),
            mk('max', 'Maximizar janela', 'window-maximize-symbolic', ['maximizar', 'tela toda']),
            mk('restore', 'Restaurar tamanho da janela', 'window-restore-symbolic', ['restaurar', 'desmaximizar']),
            mk('center', 'Centralizar janela', 'focus-windows-symbolic', ['centralizar', 'meio']),
            mk('monitor', 'Mandar janela para o outro monitor', 'video-display-symbolic', ['monitor', 'tela', 'outra tela']),
            mk('full', 'Tela cheia (liga/desliga)', 'view-fullscreen-symbolic', ['tela cheia', 'fullscreen']),
            mk('min', 'Minimizar janela', 'window-minimize-symbolic', ['minimizar', 'esconder']),
            mk('close', 'Fechar janela', 'window-close-symbolic', ['fechar', 'close']),
        ];
    }

    // ---- Janelas e print para a IA (via D-Bus) -----------------------------

    _normalWindows() {
        return global.display.list_all_windows().filter(w => w.get_window_type() === Meta.WindowType.NORMAL &&
            !w.is_skip_taskbar());
    }

    _listWindows() {
        const tracker = Shell.WindowTracker.get_default();
        return this._normalWindows().map(w => ({
            id: String(w.get_id()), titulo: w.get_title() ?? '', app: tracker.get_window_app(w)?.get_name() ?? '',
            area: (w.get_workspace()?.index() ?? 0) + 1, monitor: w.get_monitor() + 1,
            atual: w === this._prevWin, maximizada: !!w.get_maximized(), minimizada: w.minimized, pid: w.get_pid(),
        }));
    }

    // alvo: "atual" (a janela de trás do lançador), um id, ou parte do nome do app/título.
    _findWindows(alvo) {
        const a = (alvo ?? '').trim().toLowerCase();
        const all = this._normalWindows();
        if (!a || ['atual', 'ativa', 'essa', 'esta', 'foco'].includes(a))
            return this._prevWin ? [this._prevWin] : [];
        const byId = all.filter(w => String(w.get_id()) === a);
        if (byId.length)
            return byId;
        // Nome do app primeiro (exato, começo, pedaço); o título só se nenhum app bater.
        const tracker = Shell.WindowTracker.get_default();
        const app = w => (tracker.get_window_app(w)?.get_name() ?? '').toLowerCase();
        for (const test of [n => n === a, n => n.startsWith(a), n => n.includes(a)]) {
            const hit = all.filter(w => test(app(w)));
            if (hit.length)
                return hit;
        }
        return all.filter(w => (w.get_title() ?? '').toLowerCase().includes(a));
    }

    _aiWindowOp(acao, alvo) {
        const ops = {focar: 'focus', fechar: 'close', maximizar: 'max', restaurar: 'restore', minimizar: 'min',
            esquerda: 'left', direita: 'right', centralizar: 'center', outro_monitor: 'monitor', tela_cheia: 'full'};
        const area = /^area_(\d+)$/.exec(acao ?? '');
        if (!ops[acao] && !area)
            return {erro: `ação desconhecida: ${acao}`};
        const wins = this._findWindows(alvo);
        if (!wins.length)
            return {erro: `nenhuma janela encontrada para "${alvo}"`};
        const tracker = Shell.WindowTracker.get_default();
        const names = wins.map(w => `${tracker.get_window_app(w)?.get_name() ?? ''} — ${w.get_title() ?? ''}`);
        if (acao === 'focar') {
            // Só marca: quem dá o foco é o _runActions, depois que o lançador sai da frente.
            return {ok: true, focar: String(wins[0].get_id()), janelas: names.slice(0, 1)};
        }
        for (const w of wins) {
            if (area) {
                const n = Math.max(1, parseInt(area[1]));
                const wsm = global.workspace_manager;
                while (wsm.get_n_workspaces() < n)
                    wsm.append_new_workspace(false, global.get_current_time());
                w.change_workspace_by_index(n - 1, false);
            } else {
                this._winOp(w, ops[acao]);
            }
        }
        return {ok: true, janelas: names};
    }

    async _aiPrint(onde, caminho) {
        const win = this._prevWin;
        const r = onde === 'janela' && win && !win.minimized ? win.get_frame_rect()
            : global.display.get_monitor_geometry(win?.get_monitor() ?? global.display.get_primary_monitor());
        // O lançador some da foto: fica transparente só durante o print.
        const hide = [this._panel, this._side].filter(Boolean);
        hide.forEach(a => (a.opacity = 0));
        try {
            await sleep(180);
            await new Promise((res, rej) => {
                const stream = Gio.File.new_for_path(caminho).replace(null, false, Gio.FileCreateFlags.NONE, null);
                new Shell.Screenshot().screenshot_area(r.x, r.y, r.width, r.height, stream, (o, res2) => {
                    try {
                        o.screenshot_area_finish(res2);
                        stream.close(null);
                        res();
                    } catch (e) {
                        stream.close(null);
                        rej(e);
                    }
                });
            });
        } finally {
            hide.forEach(a => (a.opacity = 255));
        }
        const tracker = Shell.WindowTracker.get_default();
        return onde === 'janela' && win ? `${tracker.get_window_app(win)?.get_name() ?? ''} — ${win.get_title() ?? ''}` : 'tela';
    }

    _winOp(w, op) {
        const t = global.get_current_time();
        const wa = w.get_work_area_current_monitor();
        const unmax = () => {
            if (w.get_maximized())
                w.unmaximize(Meta.MaximizeFlags.BOTH);
        };
        switch (op) {
        case 'left':
        case 'right': {
            unmax();
            const half = Math.floor(wa.width / 2);
            w.move_resize_frame(true, op === 'left' ? wa.x : wa.x + half, wa.y, half, wa.height);
            break;
        }
        case 'max':
            w.maximize(Meta.MaximizeFlags.BOTH);
            break;
        case 'restore':
            unmax();
            break;
        case 'center': {
            unmax();
            const r = w.get_frame_rect();
            w.move_frame(true, wa.x + Math.round((wa.width - r.width) / 2), wa.y + Math.round((wa.height - r.height) / 2));
            break;
        }
        case 'monitor': {
            const n = global.display.get_n_monitors();
            if (n > 1)
                w.move_to_monitor((w.get_monitor() + 1) % n);
            break;
        }
        case 'full':
            if (w.is_fullscreen())
                w.unmake_fullscreen();
            else
                w.make_fullscreen();
            break;
        case 'min':
            w.minimize();
            return;
        case 'close':
            w.delete(t);
            return;
        }
        w.activate(t);
    }

    // ---- Jogos sempre na placa dedicada ------------------------------------

    _watchGames() {
        const cmd = clippitTool('clippit-jogos-nvidia');
        if (!GLib.file_test(cmd, GLib.FileTest.IS_EXECUTABLE))
            return;
        const run = () => runProcess([cmd], null, 60)
            .then(out => {
                if (out.startsWith('Na NVIDIA agora'))
                    console.log(`${TAG} ${out.trim()}`);
            })
            .catch(e => console.log(`${TAG} jogos na NVIDIA: ${e.message}`));
        run();
        try {
            const dir = Gio.File.new_for_path(GLib.build_filenamev([GLib.get_user_data_dir(), 'applications']));
            this._gamesMon = dir.monitor_directory(Gio.FileMonitorFlags.NONE, null);
            this._gamesMon.connect('changed', (_m, file) => {
                if (!file.get_basename()?.endsWith('.desktop'))
                    return;
                // Espera a Steam terminar de escrever (e junta vários de uma vez).
                if (this._gamesId)
                    GLib.source_remove(this._gamesId);
                this._gamesId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, () => {
                    this._gamesId = 0;
                    run();
                    return GLib.SOURCE_REMOVE;
                });
            });
        } catch (e) {
            console.log(`${TAG} jogos na NVIDIA: sem monitorar a pasta (${e.message})`);
        }
    }

    // ---- Lembretes e timers -------------------------------------------------

    _remindFile() {
        return GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'lembretes.json']);
    }

    _readReminders() {
        try {
            return JSON.parse(new TextDecoder().decode(GLib.file_get_contents(this._remindFile())[1]));
        } catch (_e) {
            return [];
        }
    }

    _writeReminders(items) {
        try {
            GLib.mkdir_with_parents(GLib.path_get_dirname(this._remindFile()), 0o700);
            GLib.file_set_contents(this._remindFile(), JSON.stringify(items));
        } catch (e) {
            console.log(`${TAG} lembretes: ${e.message}`);
        }
    }

    addReminder(minutes, texto) {
        const items = this._readReminders().filter(x => !x.feito);
        items.push({id: Date.now(), quando: Date.now() / 1000 + minutes * 60, texto, timer: true});
        this._writeReminders(items);
    }

    _checkReminders() {
        const items = this._readReminders();
        const now = Date.now() / 1000;
        const due = items.filter(x => !x.feito && x.quando <= now);
        if (!due.length)
            return;
        for (const x of due) {
            x.feito = true;
            this._alarm(x.timer ? 'Timer acabou' : 'Lembrete', x.texto);
        }
        // Guarda só o que ainda vai tocar (e o que tocou há menos de 1 dia).
        this._writeReminders(items.filter(x => !x.feito || now - x.quando < 86400));
    }

    // Notificação que fica até você fechar, com som.
    _alarm(title, body) {
        try {
            const source = new MessageTray.Source({title: 'Clippit', iconName: 'alarm-symbolic'});
            Main.messageTray.add(source);
            const n = new MessageTray.Notification({source, title, body, urgency: MessageTray.Urgency.CRITICAL});
            source.addNotification(n);
        } catch (e) {
            console.log(`${TAG} lembrete (notificação simples): ${e.message}`);
            Main.notify(title, body);
        }
        try {
            global.display.get_sound_player().play_from_theme('alarm-clock-elapsed', title, null);
        } catch (_e) {}
    }

    // ---- Conversas salvas (Ctrl+H) -----------------------------------------

    _convDir() {
        return GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'conversas']);
    }

    // Diagnóstico que o doutor do Clippit deixou para você ver (e rodar a
    // correção proposta com o Rodar de sempre).
    _doctorInbox() {
        const path = GLib.build_filenamev([GLib.get_user_data_dir(), 'clippit', 'doutor', 'sugestao.json']);
        if (!GLib.file_test(path, GLib.FileTest.EXISTS) || this._chat.some(m => m.pending))
            return;
        try {
            const sg = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(path)[1]));
            GLib.unlink(path);
            this._chat.push({role: 'user', text: 'Diagnóstico do Clippit'});
            this._chat.push({role: 'assistant', text: sg.texto ?? 'Achei um problema.', confirm: sg.comando || null,
                steps: [], sources: []});
            this._saveChat();
        } catch (e) {
            console.log(`${TAG} doutor: sugestão ilegível (${e.message})`);
            try {
                GLib.unlink(path);
            } catch (_e) {}
        }
    }

    _saveChat() {
        if (this._selfTesting)
            return; // autoteste não vai para o seu histórico
        const msgs = (this._chat ?? []).filter(m => !m.pending && (m.text || m.role === 'user'));
        if (!msgs.some(m => m.role === 'user'))
            return;
        this._chatId ??= String(Date.now());
        const data = {
            id: this._chatId, titulo: msgs.find(m => m.role === 'user').text.slice(0, 90), atualizado: Date.now(),
            mensagens: msgs.map(m => ({role: m.role, text: m.text, steps: m.steps, sources: m.sources,
                outputs: m.outputs, attachments: m.attachments, confirm: m.confirm, choice: m.choice,
                choiceAnswered: m.choice ? (m.choiceAnswered ?? 'texto') : undefined, confirmState: m.confirmState ?? (m.confirm ? 'no' : undefined)})),
        };
        try {
            GLib.mkdir_with_parents(this._convDir(), 0o700);
            GLib.file_set_contents(GLib.build_filenamev([this._convDir(), `${this._chatId}.json`]), JSON.stringify(data));
        } catch (e) {
            console.log(`${TAG} lançador: não salvei a conversa (${e.message})`);
        }
    }

    _loadConvs() {
        const out = [];
        try {
            const dir = Gio.File.new_for_path(this._convDir());
            const en = dir.enumerate_children('standard::name', Gio.FileQueryInfoFlags.NONE, null);
            let info;
            while ((info = en.next_file(null))) {
                if (!info.get_name().endsWith('.json'))
                    continue;
                try {
                    const [, raw] = GLib.file_get_contents(GLib.build_filenamev([this._convDir(), info.get_name()]));
                    out.push(JSON.parse(new TextDecoder().decode(raw)));
                } catch (_e) {}
            }
        } catch (_e) {}
        return out.sort((a, b) => b.atualizado - a.atualizado);
    }

    _convItems(q) {
        this._convs ??= this._loadConvs();
        const when = t => {
            const min = Math.round((Date.now() - t) / 60000);
            if (min < 60)
                return `há ${Math.max(1, min)} min`;
            if (min < 1440)
                return `há ${Math.round(min / 60)} h`;
            return GLib.DateTime.new_from_unix_local(Math.floor(t / 1000)).format('%d/%m %H:%M');
        };
        const items = this._convs
            .map(c => ({c, s: q ? Math.max(matchScore(q, c.titulo), ...c.mensagens.map(m => matchScore(q, m.text ?? '') * 0.8)) : 1}))
            .filter(x => x.s > 0)
            .map(({c}) => ({
                id: `conv:${c.id}`, section: 'Conversas', icon: 'user-available-symbolic', score: c.atualizado,
                title: c.titulo, subtitle: `${when(c.atualizado)} · ${(n => `${n} ${n === 1 ? 'pergunta' : 'perguntas'}`)(c.mensagens.filter(m => m.role === 'user').length)}`,
                hint: 'Abrir', keepOpen: true, data: {conv: c},
                run: () => {
                    this._chat = c.mensagens.map(m => ({...m}));
                    this._chatId = c.id;
                    this._chatTime = GLib.get_monotonic_time();
                    this._enterView('ia');
                },
            }));
        if (!items.length)
            items.push({id: 'conv-vazio', title: q ? 'Nenhuma conversa com esse texto' : 'Nenhuma conversa salva ainda',
                subtitle: '', icon: 'user-available-symbolic', score: 0, run: () => {}, keepOpen: true});
        return items;
    }

    _deleteConv(it) {
        const c = it?.data?.conv;
        if (!c)
            return;
        try {
            Gio.File.new_for_path(GLib.build_filenamev([this._convDir(), `${c.id}.json`])).delete(null);
        } catch (_e) {}
        this._convs = this._convs.filter(x => x.id !== c.id);
        if (this._chatId === c.id) {
            this._chat = [];
            this._chatId = null;
        }
        this._refresh();
    }

    // ---- @: anexar app ou arquivo à pergunta -------------------------------

    _updateMention() {
        if (this._view !== 'ia' || !this._entry) {
            this._hideMention();
            return;
        }
        const text = this._entry.text;
        let pos = this._entry.clutter_text.get_cursor_position();
        if (pos < 0)
            pos = text.length;
        const m = text.slice(0, pos).match(/(^|\s)@([^\s@]*)$/);
        if (!m) {
            this._hideMention();
            return;
        }
        const q = m[2];
        const start = pos - q.length - 1;
        const gen = this._mentionGen = (this._mentionGen ?? 0) + 1;
        const apps = q
            ? this._apps.map(a => ({a, s: matchScore(q, a.get_name())})).filter(x => x.s > 0)
                .sort((x, y) => y.s - x.s).slice(0, 5).map(x => x.a)
            : this._apps.map(a => ({a, n: this._usage[`app:${a.get_id()}`] ?? 0})).sort((x, y) => y.n - x.n)
                .slice(0, 5).map(x => x.a);
        const items = apps.map(a => ({tipo: 'app', nome: a.get_name(), id: a.get_id(), gicon: a.get_icon(), sub: 'App'}));
        // @print: captura a janela de trás e anexa como imagem.
        if (!q || ['print', 'tela', 'captura', 'screenshot', 'imagem'].some(w => w.startsWith(fold(q))))
            items.unshift({tipo: 'print', nome: 'print', gicon: lucide('camera', 'g'),
                sub: 'Captura a janela de trás e a IA olha a imagem'});
        this._mention = {start, end: pos, q, items, sel: 0};
        this._paintMention();
        if (q.length >= 2) {
            runProcess([BUSCA_CMD, 'arquivos', q], null, 6).then(out => {
                if (gen !== this._mentionGen || !this._mention)
                    return;
                for (const f of JSON.parse(out).slice(0, 5)) {
                    this._mention.items.push({
                        tipo: 'arquivo', nome: f.nome, uri: f.uri, caminho: decodeURIComponent(f.uri.replace(/^file:\/\//, '')),
                        gicon: Gio.content_type_get_icon(Gio.content_type_guess(f.nome, null)[0]), sub: f.pasta,
                    });
                }
                this._paintMention();
            }).catch(() => {});
        }
    }

    _paintMention() {
        const box = this._mentionBox;
        box.destroy_all_children();
        const mt = this._mention;
        if (!mt || !mt.items.length) {
            box.visible = Boolean(mt) && mt.q.length >= 2;
            if (box.visible)
                box.add_child(new St.Label({text: 'Procurando apps e arquivos…', style: LSTYLE.sideSub}));
            return;
        }
        mt.items.forEach((it, i) => {
            const row = new St.BoxLayout({style: i === mt.sel ? LSTYLE.rowOn : LSTYLE.row, reactive: true});
            row.add_child(new St.Icon({gicon: it.gicon ?? null, icon_name: 'text-x-generic-symbolic', icon_size: 20,
                y_align: Clutter.ActorAlign.CENTER}));
            row.add_child(new St.Label({text: it.nome, style: LSTYLE.sideText, y_align: Clutter.ActorAlign.CENTER}));
            const sub = new St.Label({text: it.sub ?? '', style: LSTYLE.sideSub, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            sub.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            row.add_child(sub);
            row.connect('button-press-event', () => {
                mt.sel = i;
                this._mentionPick();
                return Clutter.EVENT_STOP;
            });
            box.add_child(row);
        });
        box.show();
    }

    _mentionMove(d) {
        const mt = this._mention;
        if (!mt?.items.length)
            return;
        mt.sel = Math.max(0, Math.min(mt.items.length - 1, mt.sel + d));
        this._paintMention();
    }

    // Troca "@texto" por "@Nome " e guarda o anexo para a pergunta.
    _mentionPick() {
        const mt = this._mention;
        const it = mt?.items[mt.sel];
        if (!it)
            return;
        if (it.tipo === 'print') {
            this._attachPrint(mt);
            return;
        }
        const text = this._entry.text;
        const before = `${text.slice(0, mt.start)}@${it.nome} `;
        this._attach = [...(this._attach ?? []).filter(a => a.nome !== it.nome), {
            tipo: it.tipo, nome: it.nome, id: it.id, caminho: it.caminho, uri: it.uri,
        }];
        this._hideMention();
        this._entry.text = before + text.slice(mt.end).replace(/^\s+/, '');
        this._entry.clutter_text.set_cursor_position(before.length);
        this._hideMention();
    }

    _attachToken(mt, att) {
        const text = this._entry.text;
        const before = `${text.slice(0, mt ? mt.start : text.length)}${mt || !text || /\s$/.test(text) ? '' : ' '}@${att.nome} `;
        this._attach = [...(this._attach ?? []).filter(a => a.nome !== att.nome), att];
        this._hideMention();
        this._entry.text = before + (mt ? text.slice(mt.end).replace(/^\s+/, '') : '');
        this._entry.clutter_text.set_cursor_position(before.length);
        this._hideMention();
    }

    _nextName(base) {
        const used = new Set((this._attach ?? []).map(a => a.nome));
        let n = 1;
        while (used.has(n === 1 ? base : `${base}${n}`))
            n++;
        return n === 1 ? base : `${base}${n}`;
    }

    // Esconde o lançador por um instante, fotografa a janela de trás e anexa.
    async _attachPrint(mt) {
        const dir = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'anexos']);
        GLib.mkdir_with_parents(dir, 0o700);
        const path = GLib.build_filenamev([dir, `print-${Date.now()}.png`]);
        const win = this._prevWin;
        const mon = this._mon ?? Main.layoutManager.primaryMonitor;
        const r = win && !win.minimized ? win.get_frame_rect() : {x: mon.x, y: mon.y, width: mon.width, height: mon.height};
        const hidden = [this._panel, this._side].filter(Boolean);
        hidden.forEach(a => (a.opacity = 0));
        await sleep(180);
        try {
            await new Promise((resolve, reject) => {
                const stream = Gio.File.new_for_path(path).replace(null, false, Gio.FileCreateFlags.NONE, null);
                new Shell.Screenshot().screenshot_area(r.x, r.y, r.width, r.height, stream, (o, res) => {
                    try {
                        o.screenshot_area_finish(res);
                        stream.close(null);
                        resolve();
                    } catch (e) {
                        reject(e);
                    }
                });
            });
        } catch (e) {
            console.log(`${TAG} lançador: print falhou (${e.message})`);
        } finally {
            hidden.forEach(a => (a.opacity = 255));
        }
        if (!this._panel || !GLib.file_test(path, GLib.FileTest.EXISTS))
            return;
        this._attachToken(mt, {tipo: 'imagem', nome: this._nextName('print'), caminho: path});
    }

    _hideMention() {
        this._mention = null;
        this._mentionGen = (this._mentionGen ?? 0) + 1;
        if (this._mentionBox) {
            this._mentionBox.destroy_all_children();
            this._mentionBox.hide();
        }
    }

    // ---- Área de transferência ao lado (Alt+C) ------------------------------

    _toggleSide() {
        // O atalho pode chegar duas vezes (tecla + D-Bus do Clippit).
        const now = GLib.get_monotonic_time();
        if (now - (this._sideToggled ?? 0) < 400000)
            return;
        this._sideToggled = now;
        if (this._side)
            this._closeSide();
        else
            this._openSide();
    }

    _closeSide() {
        this._sideTx = null;
        if (!this._side)
            return;
        Main.layoutManager.removeChrome(this._side);
        this._side.destroy();
        this._side = null;
        this._sideRows = [];
    }

    _placeSide() {
        if (!this._side || !this._panel)
            return;
        const mon = this._mon ?? Main.layoutManager.primaryMonitor;
        const [px, py] = this._panel.get_position();
        const [W, H] = this._panel.get_size();
        const sw = 360;
        let x = px + W + 12; // à direita; sem espaço, à esquerda
        if (x + sw > mon.x + mon.width - 8)
            x = px - 12 - sw;
        if (x < mon.x + 8)
            x = mon.x + mon.width - sw - 8; // tela estreita: por cima, no canto
        this._side.set_size(sw, H);
        this._side.set_position(Math.round(x), Math.round(py));
    }

    async _openSide() {
        if (!this._panel || this._side)
            return;
        const side = new St.BoxLayout({vertical: true, reactive: true, style: LSTYLE.side});
        try {
            side.add_effect(new Shell.BlurEffect({radius: 24, brightness: 0.6, mode: Shell.BlurMode.BACKGROUND}));
        } catch (_e) {}
        const head = new St.BoxLayout({style: LSTYLE.sideHead});
        head.add_child(new St.Icon({icon_name: 'edit-paste-symbolic', icon_size: 14, style: `color: ${C.accent};`,
            y_align: Clutter.ActorAlign.CENTER}));
        head.add_child(new St.Label({text: 'Área de transferência', style: LSTYLE.sideTitle, x_expand: true,
            y_align: Clutter.ActorAlign.CENTER}));
        head.add_child(keycap('Alt C'));
        side.add_child(head);
        this._sideScroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: true, x_expand: true, y_expand: true,
        });
        this._sideList = new St.BoxLayout({vertical: true, style: LSTYLE.sideList});
        if (this._sideScroll.set_child)
            this._sideScroll.set_child(this._sideList);
        else
            this._sideScroll.add_actor(this._sideList);
        side.add_child(this._sideScroll);
        smoothScroll(this._sideScroll, () => this._sideList?.vadjustment);
        const sf = makeFooter();
        this._sideFootBar = sf;
        // Compatível com quem só troca o texto (avisos como "Copiado ✓").
        this._sideFoot = {set text(t) {
            sf.set(t, 'edit-paste-symbolic', []);
        }, get text() {
            return sf.primary;
        }};
        side.add_child(sf.actor);
        this._side = side;
        // Solto, o painel ao lado recebe o próprio clique: reativa por aqui.
        side.connect('captured-event', (_a, ev) => (this._grab ? Clutter.EVENT_PROPAGATE : this._onCaptured?.(ev) ?? Clutter.EVENT_PROPAGATE));
        this._sideRows = [];
        this._sideSel = 0;
        Main.layoutManager.addTopChrome(side);
        this._placeSide();
        this._sideList.add_child(new St.Label({text: 'Carregando…', style: `${TX.small} padding: 12px 10px;`}));
        let items = [];
        try {
            items = JSON.parse(await runProcess([BUSCA_CMD, 'historico', ''], null, 5));
        } catch (e) {
            console.log(`${TAG} lançador: histórico falhou (${e.message})`);
        }
        if (this._side !== side)
            return;
        this._sideItems = items;
        this._sideSel = 0;
        this._paintSide();
    }

    // Lista do painel ao lado: o que foi copiado ou, com Tab, as transformações.
    _paintSide() {
        const items = this._sideItems ?? [];
        this._sideList.destroy_all_children();
        if (this._sideTx) {
            const h = this._sideTx;
            const head = new St.Label({text: `Transformar: ${(h.tipo === 'image' ? h.ocr : h.texto || '').replace(/\s+/g, ' ').slice(0, 50)}`,
                style: `${LSTYLE.sideSub} padding: 4px 10px 6px 10px;`});
            head.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            this._sideList.add_child(head);
            this._sideRows = TEXT_TRANSFORMS.map((tf, i) => {
                const row = new St.BoxLayout({style: i === this._sideSel ? LSTYLE.rowOn : LSTYLE.row, reactive: true});
                row.add_child(new St.Icon({gicon: lucide((ICON_MAP[tf.icon] ?? ['wand-sparkles'])[0], 'g'), icon_size: 16,
                    y_align: Clutter.ActorAlign.CENTER}));
                row.add_child(new St.Label({text: tf.title, style: LSTYLE.sideText, x_expand: true, y_align: Clutter.ActorAlign.CENTER}));
                if (tf.ai)
                    row.add_child(new St.Label({text: 'IA', style: LSTYLE.sideSub, y_align: Clutter.ActorAlign.CENTER}));
                this._sideList.add_child(row);
                return row;
            });
            this._sideFootBar.set('', 'edit-paste-symbolic', [{label: 'Colar no campo', key: '↵'}, {label: 'Voltar', key: 'Tab'}]);
            return;
        }
        this._sideFootBar.set('', 'edit-paste-symbolic', [{label: 'Colar', key: '↵'}, {label: 'Transformar', key: 'Tab'},
            {label: 'Fixar', key: 'Ctrl P'}]);
        if (!items.length)
            this._sideList.add_child(emptyState('clipboard-list', '#f97316', 'Nada copiado ainda', 'O que você copiar aparece aqui.'));
        this._sideRows = items.map((h, i) => {
            const row = new St.BoxLayout({style: i === this._sideSel ? LSTYLE.rowOn : LSTYLE.row, reactive: true});
            const img = h.tipo === 'image' && h.imagem;
            if (img) {
                row.add_child(new St.Icon({gicon: new Gio.FileIcon({file: Gio.File.new_for_path(h.imagem)}),
                    icon_size: 36, style: LSTYLE.sideThumb, y_align: Clutter.ActorAlign.CENTER}));
            } else {
                row.add_child(new St.Icon({gicon: lucide(h.fixado ? 'pin' : /^https?:\/\//.test(h.texto) ? 'link' : 'type',
                    h.fixado ? 'acc' : 'g'), icon_size: 16, y_align: Clutter.ActorAlign.CENTER}));
            }
            const col = new St.BoxLayout({vertical: true, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            const ocr = (h.ocr || '').replace(/\s+/g, ' ').trim();
            const text = img ? (ocr || 'Imagem') : (h.texto || '').replace(/\s+/g, ' ').trim();
            const t = new St.Label({text: text.slice(0, 160) || '(vazio)', style: LSTYLE.sideText});
            t.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            col.add_child(t);
            col.add_child(new St.Label({
                text: (h.fixado ? 'Fixado · ' : '') + (img ? (this._view === 'ia' ? 'Imagem · anexa para a IA ver' : ocr ? 'Imagem · cola o texto lido' : 'Imagem · copia a imagem')
                    : /^https?:\/\//.test(h.texto) ? 'Link' : `${(h.texto || '').length} caracteres`),
                style: LSTYLE.sideSub,
            }));
            row.add_child(col);
            this._sideList.add_child(row);
            return row;
        });
    }

    _sideMove(d) {
        const n = this._sideRows?.length ?? 0;
        if (!n)
            return;
        this._sideRows[this._sideSel]?.set_style(LSTYLE.row);
        this._sideSel = Math.max(0, Math.min(n - 1, this._sideSel + d));
        const row = this._sideRows[this._sideSel];
        row.set_style(LSTYLE.rowOn);
        // Mantém a linha escolhida visível.
        const adj = this._sideList.vadjustment;
        const box = row.get_allocation_box();
        if (box.y1 < adj.value)
            adj.value = box.y1;
        else if (box.y2 > adj.value + adj.page_size)
            adj.value = box.y2 - adj.page_size;
    }

    // Enter: cola no campo do lançador (imagem: o texto lido, se houver).
    // Ctrl+Enter (ou imagem sem texto): copia para a área de transferência.
    async _sideUse(copy) {
        const h = this._sideItems?.[this._sideSel];
        if (!h)
            return;
        if (h.tipo === 'image' && h.imagem && this._view === 'ia' && !copy) {
            // Na conversa com a IA, a imagem vai como anexo (ela olha a imagem).
            this._closeSide();
            this._attachToken(null, {tipo: 'imagem', nome: this._nextName('imagem'), caminho: h.imagem});
            this._entry.grab_key_focus();
            return;
        }
        const text = h.tipo === 'image' ? (h.ocr || '').trim() : h.texto;
        if (copy || !text) {
            try {
                await runProcess([BUSCA_CMD, 'copiar', String(h.id)], null, 5);
                if (this._sideFoot)
                    this._sideFoot.text = h.tipo === 'image' ? 'Imagem copiada ✓' : 'Copiado ✓';
            } catch (e) {
                console.log(`${TAG} lançador: copiar falhou (${e.message})`);
            }
            return;
        }
        const ct = this._entry.clutter_text;
        const pos = ct.get_cursor_position();
        ct.insert_text(text, pos < 0 ? this._entry.text.length : pos);
        this._closeSide();
        this._entry.grab_key_focus();
    }

    async _sideApplyTx(tf) {
        const h = this._sideTx;
        const text = h?.tipo === 'image' ? (h.ocr || '').trim() : h?.texto ?? '';
        if (!tf || !text)
            return;
        this._sideFoot.text = `${tf.title}…`;
        let out;
        try {
            out = await runTransform(tf, text);
        } catch (e) {
            if (this._sideFoot)
                this._sideFoot.text = `Não deu: ${e.message}`;
            return;
        }
        if (!this._panel)
            return;
        const ct = this._entry.clutter_text;
        const pos = ct.get_cursor_position();
        ct.insert_text(out, pos < 0 ? this._entry.text.length : pos);
        this._closeSide();
        this._entry.grab_key_focus();
    }

    // O lançador segura o mouse só na janela principal: cliques e rolagem no
    // painel ao lado são tratados aqui.
    _sideEvent(ev) {
        const type = ev.type();
        if (type !== Clutter.EventType.BUTTON_PRESS && type !== Clutter.EventType.SCROLL)
            return false;
        const [x, y] = ev.get_coords();
        const [sx, sy] = this._side.get_transformed_position();
        const [sw, sh] = this._side.get_transformed_size();
        if (x < sx || x > sx + sw || y < sy || y > sy + sh)
            return false;
        if (type === Clutter.EventType.SCROLL) {
            const adj = this._sideList.vadjustment;
            const dir = ev.get_scroll_direction();
            let dy = dir === Clutter.ScrollDirection.UP ? -1 : dir === Clutter.ScrollDirection.DOWN ? 1 : 0;
            if (dir === Clutter.ScrollDirection.SMOOTH)
                dy = ev.get_scroll_delta()[1];
            if (dy)
                smoothStep(adj, dy, this._sideScroll);
            return true;
        }
        const i = (this._sideRows ?? []).findIndex(r => {
            const [rx, ry] = r.get_transformed_position();
            const [rw, rh] = r.get_transformed_size();
            return x >= rx && x <= rx + rw && y >= ry && y <= ry + rh;
        });
        if (i >= 0) {
            this._sideMove(i - this._sideSel);
            if (this._sideTx)
                this._sideApplyTx(TEXT_TRANSFORMS[this._sideSel]);
            else
                this._sideUse((ev.get_state() & Clutter.ModifierType.CONTROL_MASK) !== 0);
        }
        return true;
    }

    // ---- Corretor no campo do lançador --------------------------------------

    _scheduleFix() {
        if (this._fixId) {
            GLib.source_remove(this._fixId);
            this._fixId = 0;
        }
        const text = this._entry?.text ?? '';
        if (this._fix && this._fix.from !== text) {
            this._fix = null;
            this._fixRow.hide();
        }
        if (this._fixChecking && this._fixChecking !== text)
            this._setFixState(null); // mudou o texto: some o "verificando" antigo
        // Só na conversa com a IA (frases); na busca são termos curtos.
        if (this._view !== 'ia' || text.trim().split(/\s+/).length < 2)
            return;
        this._fixId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 700, () => {
            this._fixId = 0;
            this._checkFix(text);
            return GLib.SOURCE_REMOVE;
        });
    }

    // Estados da linha: 'checking' (anel girando + tempo), 'ok' (sem erros,
    // some sozinho), 'error' ou null (escondida). A sugestão usa _fix.
    _setFixState(state, msg = '') {
        if (!this._fixRow)
            return;
        if (this._fixTickId) {
            GLib.source_remove(this._fixTickId);
            this._fixTickId = 0;
        }
        this._fixChecking = state === 'checking' ? this._entry.text : null;
        const checking = state === 'checking';
        this._fixRing.actor.visible = checking;
        if (checking)
            this._fixRing.start();
        else
            this._fixRing.stop();
        this._fixIcon.visible = !checking;
        this._fixBtn.visible = state === 'fix';
        if (!state) {
            this._fixRow.hide();
            return;
        }
        this._fixIcon.icon_name = state === 'error' ? 'dialog-warning-symbolic'
            : state === 'ok' ? 'object-select-symbolic' : state === 'rec' ? 'audio-input-microphone-symbolic'
                : 'tools-check-spelling-symbolic';
        this._fixIcon.style = `color: ${state === 'error' || state === 'rec' ? C.danger : state === 'ok' ? C.ok : C.accent};`;
        if (state !== 'fix')
            this._fixLabel.text = msg;
        this._fixRow.show();
        if (checking) {
            const t0 = GLib.get_monotonic_time();
            this._fixTickId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                const secs = Math.floor((GLib.get_monotonic_time() - t0) / 1e6);
                this._fixLabel.text = `Verificando a escrita…${secs >= 1 ? ` ${secs} s` : ''}`;
                return GLib.SOURCE_CONTINUE;
            });
        }
        if (state === 'ok') {
            this._fixTickId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1500, () => {
                this._fixTickId = 0;
                if (this._fixRow && !this._fix)
                    this._fixRow.hide();
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    async _checkFix(text) {
        if (this._chat?.some(m => m.pending))
            return; // o modelo está ocupado com a pergunta
        this._setFixState('checking', 'Verificando a escrita…');
        let out;
        try {
            out = (await this._c._runCorrector(text)).trim();
        } catch (e) {
            console.log(`${TAG} lançador: corretor falhou (${e.message})`);
            if (this._panel && this._entry.text === text)
                this._setFixState('error', 'Não consegui verificar a escrita agora.');
            return;
        }
        if (!this._panel || this._entry.text !== text)
            return;
        if (!out || out === text.trim()) {
            this._setFixState('ok', 'Sem erros.');
            return;
        }
        this._fix = {from: text, to: out};
        this._setFixState('fix');
        // Palavras trocadas em destaque.
        const before = new Set(text.split(/\s+/));
        this._fixLabel.clutter_text.set_markup(out.split(/(\s+)/).map(w =>
            /\s/.test(w) || before.has(w) ? esc(w) : `<span foreground="${C.text}" underline="single">${esc(w)}</span>`).join(''));
        this._fixRow.show();
    }

    _applyFix() {
        if (!this._fix || this._entry.text !== this._fix.from)
            return false;
        const to = this._fix.to;
        this._fix = null;
        this._setFixState(null);
        this._entry.text = to;
        this._entry.clutter_text.set_cursor_position(-1);
        this._entry.grab_key_focus();
        return true;
    }

    _close() {
        if (this._panel && !this._consumed && (this._entry.text || (this._view === 'ia' && this._chat?.length))) {
            this._saved = {time: GLib.get_monotonic_time(), view: this._view, text: this._entry.text};
        } else if (this._consumed) {
            this._saved = null;
        }
        if (this._outsideId) {
            global.stage.disconnect(this._outsideId);
            this._outsideId = 0;
        }
        if (this._grab) {
            Main.popModal(this._grab);
            this._grab = null;
        }
        if (this._debounceId) {
            GLib.source_remove(this._debounceId);
            this._debounceId = 0;
        }
        if (this._fixId) {
            GLib.source_remove(this._fixId);
            this._fixId = 0;
        }
        if (this._rec)
            this._stopDictation(false);
        if (this._panel) {
            this._setFixState(null);
            this._hideMention();
        }
        this._closeSide();
        if (this._panel) {
            Main.layoutManager.removeChrome(this._panel);
            this._panel.destroy();
            this._panel = null;
            this._c.setSuspended?.(false);
        }
        this._fix = null;
        this._gen = (this._gen ?? 0) + 1;
        this._askGen = (this._askGen ?? 0) + 1;
    }

    _onKey(ev) {
        const sym = ev.get_key_symbol();
        const ctrl = (ev.get_state() & Clutter.ModifierType.CONTROL_MASK) !== 0;
        const alt = (ev.get_state() & Clutter.ModifierType.MOD1_MASK) !== 0;
        // Alt+V: ditado (fala a pergunta).
        if (alt && (sym === Clutter.KEY_v || sym === Clutter.KEY_V)) {
            if (this._view !== 'ia')
                this._enterView('ia');
            this._toggleDictation();
            return Clutter.EVENT_STOP;
        }
        // Alt+C (atalho geral do Clippit): no histórico fecha; nas outras
        // telas abre/fecha a área de transferência ao lado.
        if (alt && (sym === Clutter.KEY_c || sym === Clutter.KEY_C)) {
            if (this._isHistory())
                this._close();
            else
                this._toggleSide();
            return Clutter.EVENT_STOP;
        }
        if (this._mention) {
            const moves = {[Clutter.KEY_Down]: 1, [Clutter.KEY_Up]: -1};
            if (sym in moves) {
                this._mentionMove(moves[sym]);
                return Clutter.EVENT_STOP;
            }
            if ([Clutter.KEY_Return, Clutter.KEY_KP_Enter, Clutter.KEY_Tab].includes(sym) && this._mention.items.length) {
                this._mentionPick();
                return Clutter.EVENT_STOP;
            }
            if (sym === Clutter.KEY_Escape) {
                this._hideMention();
                return Clutter.EVENT_STOP;
            }
        }
        if (this._side) {
            const moves = {[Clutter.KEY_Down]: 1, [Clutter.KEY_Up]: -1, [Clutter.KEY_Page_Down]: 5, [Clutter.KEY_Page_Up]: -5};
            if (sym in moves) {
                this._sideMove(moves[sym]);
                return Clutter.EVENT_STOP;
            }
            if (sym === Clutter.KEY_Return || sym === Clutter.KEY_KP_Enter) {
                if (this._sideTx)
                    this._sideApplyTx(TEXT_TRANSFORMS[this._sideSel]);
                else
                    this._sideUse(ctrl);
                return Clutter.EVENT_STOP;
            }
            if (sym === Clutter.KEY_Tab) {
                const h = this._sideItems?.[this._sideSel];
                this._sideTx = this._sideTx ? null : h ?? null;
                this._sideSel = this._sideTx ? 0 : Math.max(0, this._sideItems.indexOf(h));
                this._paintSide();
                return Clutter.EVENT_STOP;
            }
            if (ctrl && (sym === Clutter.KEY_p || sym === Clutter.KEY_P) && !this._sideTx) {
                const h = this._sideItems?.[this._sideSel];
                if (h)
                    this._togglePin(h);
                return Clutter.EVENT_STOP;
            }
        }
        if (this._fix && ((alt && [Clutter.KEY_s, Clutter.KEY_S, Clutter.KEY_o, Clutter.KEY_O].includes(sym)) ||
            sym === Clutter.KEY_Tab || sym === Clutter.KEY_ISO_Left_Tab) && this._applyFix())
            return Clutter.EVENT_STOP;
        if (this._isHistory() && sym === Clutter.KEY_Tab) {
            // Tab: transformar o item (MAIÚSCULAS, JSON, links, traduzir…); Tab de novo volta.
            const h = this._shown?.[this._sel]?.data;
            this._cbTx = this._cbTx ? null : h?.tipo ? h : null;
            this._sel = 0;
            this._refresh();
            return Clutter.EVENT_STOP;
        }
        if (this._isHistory() && this._cbTx && sym === Clutter.KEY_BackSpace && !this._entry.text) {
            this._cbTx = null;
            this._refresh();
            return Clutter.EVENT_STOP;
        }
        if (this._isHistory() && ctrl && (sym === Clutter.KEY_p || sym === Clutter.KEY_P)) {
            const h = this._cbTx ?? this._shown?.[this._sel]?.data;
            if (h?.tipo)
                this._togglePin(h);
            return Clutter.EVENT_STOP;
        }
        if (this._isHistory() && ctrl && (sym === Clutter.KEY_Delete || sym === Clutter.KEY_KP_Delete)) {
            this._deleteHistory(this._shown?.[this._sel]);
            return Clutter.EVENT_STOP;
        }
        if (this._isHistory() && ctrl && sym === Clutter.KEY_space) {
            this._toggleZoom();
            return Clutter.EVENT_STOP;
        }
        if (this._view === 'texto' && this._tx) {
            if ([Clutter.KEY_Return, Clutter.KEY_KP_Enter, Clutter.KEY_ISO_Enter].includes(sym)) {
                if (!this._tx.pending && this._tx.text) {
                    if (ctrl) {
                        St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, this._tx.text);
                        this._consumed = true;
                        this._close();
                        Main.notify('Clippit', 'Copiado.');
                    } else {
                        this._pasteText(this._tx.text);
                    }
                }
                return Clutter.EVENT_STOP;
            }
            if (sym === Clutter.KEY_BackSpace && !this._entry.text) {
                this._tx = null; // volta para a lista de ações
                this._render();
                return Clutter.EVENT_STOP;
            }
        }
        if (sym === Clutter.KEY_BackSpace && !this._entry.text && this._view !== 'root') {
            this._enterView('root');
            return Clutter.EVENT_STOP;
        }
        if (this._view === 'ia' && ctrl && (sym === Clutter.KEY_h || sym === Clutter.KEY_H)) {
            this._convs = null;
            this._enterView('conversas');
            return Clutter.EVENT_STOP;
        }
        if (this._view === 'conversas' && ctrl && (sym === Clutter.KEY_Delete || sym === Clutter.KEY_KP_Delete)) {
            this._deleteConv(this._shown?.[this._sel]);
            return Clutter.EVENT_STOP;
        }
        if (this._view === 'ia' && ctrl && (sym === Clutter.KEY_n || sym === Clutter.KEY_N)) {
            this._chat = []; // nova conversa (a anterior fica salva em Ctrl+H)
            this._chatId = null;
            this._render();
            return Clutter.EVENT_STOP;
        }
        if (sym === Clutter.KEY_Escape) {
            // IA respondendo: o primeiro Esc só para a resposta.
            if (this._view === 'ia' && this._chat?.some(m => m.pending)) {
                this._stopAsk();
                return Clutter.EVENT_STOP;
            }
            // Esc fecha tudo, de qualquer tela (o texto fica guardado por
            // 5 min). Para voltar ao início: seta ← ou Backspace no campo vazio.
            this._close();
            return Clutter.EVENT_STOP;
        }
        if (this._view === 'emoji' && this._emojiKey(sym, ev))
            return Clutter.EVENT_STOP;
        if (this._view === 'ia' && (sym === Clutter.KEY_Down || sym === Clutter.KEY_Up) && !this._entry.text.trim() &&
            this._pendingChoice()) {
            this._moveChoice(this._pendingChoice(), sym === Clutter.KEY_Down ? 1 : -1);
            return Clutter.EVENT_STOP;
        }
        if (sym === Clutter.KEY_Down || sym === Clutter.KEY_Up || sym === Clutter.KEY_Page_Down || sym === Clutter.KEY_Page_Up) {
            const step = sym === Clutter.KEY_Down ? 1 : sym === Clutter.KEY_Up ? -1 : sym === Clutter.KEY_Page_Down ? 5 : -5;
            const n = this._shown?.length ?? 0;
            if (n)
                this._sel = Math.max(0, Math.min(n - 1, this._sel + step));
            this._paint();
            return Clutter.EVENT_STOP;
        }
        if (sym === Clutter.KEY_Return || sym === Clutter.KEY_KP_Enter || sym === Clutter.KEY_ISO_Enter) {
            if (this._view === 'ia' && !this._entry.text.trim() && this._pendingChoice()) {
                const m = this._pendingChoice();
                this._answerChoice(m, m.choiceSel);
                return Clutter.EVENT_STOP;
            }
            if (this._view === 'ia') {
                // Chat: Enter envia; com o campo vazio, Enter copia a última
                // resposta e Ctrl+Enter abre a fonte dela.
                const q = this._entry.text.trim();
                const last = [...(this._chat ?? [])].reverse().find(m => m.role === 'assistant' && !m.pending);
                if (q) {
                    this._send(q);
                } else if (last?.confirm && !last.confirmState) {
                    this._confirm(last, true);
                } else if (last && ctrl && last.sources?.length) {
                    Gio.AppInfo.launch_default_for_uri(last.sources[0], global.create_app_launch_context(0, -1));
                    this._close();
                } else if (last?.text) {
                    this._consumed = true;
                    St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, last.text);
                    this._close();
                    Main.notify('Clippit', 'Resposta copiada.');
                }
                return Clutter.EVENT_STOP;
            }
            const item = this._shown?.[this._sel];
            if (item)
                this._activate(item, ctrl);
            return Clutter.EVENT_STOP;
        }
        return Clutter.EVENT_PROPAGATE;
    }

    _activate(item, alt) {
        if (item.confirm && !item.confirmed) {
            // Ações que não dá para desfazer pedem um segundo Enter.
            item.confirmed = true;
            item.title = item.confirm;
            this._render();
            return;
        }
        if (item.key)
            this._bump(item.key);
        const fn = alt && item.alt ? item.alt : item.run;
        if (!item.keepOpen) {
            this._consumed = true; // fez o que queria: não guarda o texto
            this._close();
        }
        try {
            fn();
        } catch (e) {
            console.log(`${TAG} lançador: ação falhou: ${e.message}`);
            Main.notify('Clippit', `Não consegui: ${e.message}`);
        }
    }

    // ---- Busca -------------------------------------------------------------

    _onQuery() {
        // ":" na busca inicial abre o seletor de emoji (o resto vira o filtro).
        const raw = this._entry?.text ?? '';
        if (this._view === 'root' && (raw.startsWith(':') || /^emoji\s/i.test(raw))) {
            const rest = raw.startsWith(':') ? raw.slice(1) : raw.replace(/^emoji\s+/i, '');
            this._enterView('emoji');
            this._entry.text = rest;
            this._entry.clutter_text.set_cursor_position(-1);
            return;
        }
        this._updateMention();
        this._scheduleFix();
        this._answer = null;
        this._sel = 0;
        this._emojiSel = 0;
        this._refresh();
        // Buscas mais lentas (arquivos, histórico) esperam a digitação parar.
        if (this._debounceId)
            GLib.source_remove(this._debounceId);
        this._debounceId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 220, () => {
            this._debounceId = 0;
            this._searchAsync(this._entry.text.trim());
            return GLib.SOURCE_REMOVE;
        });
    }

    async _searchAsync(q) {
        if (this._view === 'ia' || this._view === 'emoji')
            return;
        const gen = ++this._gen;
        const cbMode = this._isHistory(q);
        const hq = this._view === 'historico' ? q : q.replace(/^cb\s*/i, '');
        const jobs = [];
        if (cbMode || hq.length >= 3)
            jobs.push(runProcess([BUSCA_CMD, 'historico', hq], null, 5).then(o => ['history', JSON.parse(o)]));
        if (!cbMode && this._view === 'root' && q.length >= 3 && !q.startsWith('?'))
            jobs.push(runProcess([BUSCA_CMD, 'arquivos', q], null, 6).then(o => ['files', JSON.parse(o)]));
        for (const job of jobs) {
            try {
                const [kind, data] = await job;
                if (gen !== this._gen || !this._panel)
                    return;
                this._async[kind] = data;
                this._refresh(true);
            } catch (e) {
                console.log(`${TAG} lançador: busca falhou: ${e.message}`);
            }
        }
    }

    _refresh(keepSel = false) {
        if (!this._panel)
            return;
        const q = this._entry.text.trim();
        const sel = this._shown?.[this._sel];
        this._shown = this._collect(q);
        if (keepSel && sel) {
            const i = this._shown.findIndex(it => it.id === sel.id);
            this._sel = i >= 0 ? i : 0;
        }
        this._sel = Math.min(this._sel, Math.max(0, this._shown.length - 1));
        this._render();
    }

    _collect(q) {
        const items = [];
        const ql = fold(q);

        // Pergunta à IA: tela própria ou "? como faço…"
        if ((this._view === 'ia' || q.startsWith('?')) && !aiOn('chat')) {
            return [{id: 'ia-off', icon: 'starred-symbolic', score: 1000, title: 'A IA está desligada',
                subtitle: 'Configurações do Clippit → Geral → Inteligência artificial', keepOpen: true, run: () => {}}];
        }
        if (this._view === 'ia' || q.startsWith('?')) {
            const question = this._view === 'ia' ? q : q.slice(1).trim();
            items.push({
                id: 'ia', section: 'IA local', icon: 'starred-symbolic', score: 1000,
                title: question ? `Perguntar à IA: ${question}` : 'Pergunte algo à IA (roda no seu computador)',
                subtitle: 'Enter envia · a resposta aparece aqui', hint: 'Perguntar', keepOpen: true,
                run: () => {
                    this._enterView('ia');
                    if (question)
                        this._send(question);
                },
            });
            return items;
        }

        if (this._view === 'conversas')
            return this._convItems(q);
        if (this._view === 'emoji')
            return []; // a grade se desenha sozinha
        if (this._view === 'texto')
            return this._textActionItems(q);
        // ;gatilho → snippets
        if (q.startsWith(';'))
            return this._snippetItems(q);

        // Histórico do Clippit: tela própria (ou "cb texto", atalho escondido)
        if (this._isHistory(q) && this._cbTx)
            return this._cbTxItems(q);
        if (this._isHistory(q)) {
            for (const h of this._async.history)
                items.push(this._historyItem(h, 500));
            if (!items.length)
                items.push({id: 'cb-vazio', title: 'Nada no histórico com esse texto', subtitle: '', icon: 'edit-paste-symbolic', score: 0, run: () => {}});
            return items;
        }

        if (!q) {
            // Sem texto: os apps mais usados e sugestões de comandos.
            const used = this._apps
                .map(a => ({a, n: this._usage[`app:${a.get_id()}`] ?? 0}))
                .filter(x => x.n > 0).sort((x, y) => y.n - x.n).slice(0, 5);
            for (const {a} of used)
                items.push(this._appItem(a, 100, 'Mais usados'));
            const cmds = this._commands();
            for (const id of ['cmd:historico', 'cmd:ia', 'cmd:game', 'cmd:clippit', 'cmd:lock'])
                items.push({...cmds.find(c => c.id === id), section: 'Sugestões', score: 0});
            return items;
        }

        // Atalhos com número: timer 25, volume 40, brilho 70
        for (const it of this._quickNumberItems(q))
            items.push(it);

        // Conta ou conversão
        const conv = convert(q);
        const calc = conv ? null : calculate(q);
        if (conv || calc !== null) {
            const value = conv ? conv.value : calc;
            const shown = `${formatNum(value)}${conv ? ` ${conv.unit}` : ''}`;
            const copy = formatNum(value)?.replace(/\./g, '');
            items.push({
                id: 'calc', section: 'Calculadora', icon: 'accessories-calculator-symbolic', score: 900,
                title: `= ${shown}`, subtitle: `${q} · Enter copia o resultado`, hint: 'Copiar',
                run: () => {
                    St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, copy);
                    Main.notify('Clippit', `Copiado: ${copy}`);
                },
            });
        }

        // Pesquisa na web com prefixo: "yt música", "g clima"
        const quick = this._quicklinks();
        const pm = q.match(/^(\S+)\s+(.+)$/);
        const prefixed = pm && quick.find(w => w.key === fold(pm[1]));
        if (prefixed)
            items.push(this._webItem(prefixed, pm[2], 950));

        // Apps
        for (const a of this._apps) {
            const s = Math.max(
                matchScore(q, a.get_name()),
                matchScore(q, a.get_generic_name?.() ?? '') * 0.7,
                ...((a.get_keywords?.() ?? []).map(k => matchScore(q, k) * 0.6)),
                matchScore(q, a.get_executable?.() ?? '') * 0.6);
            if (s >= 40)
                items.push(this._appItem(a, s + 200 + this._usageBoost(`app:${a.get_id()}`), 'Apps'));
        }

        // Janelas abertas
        const tracker = Shell.WindowTracker.get_default();
        for (const w of global.display.get_tab_list(Meta.TabList.NORMAL_ALL, null)) {
            const app = tracker.get_window_app(w);
            const s = Math.max(matchScore(q, w.get_title() ?? ''), matchScore(q, app?.get_name() ?? '') * 0.9);
            if (s >= 40) {
                items.push({
                    id: `win:${w.get_id()}`, section: 'Janelas abertas', gicon: app?.get_icon(), icon: 'focus-windows-symbolic',
                    score: s + 150, title: w.get_title() ?? app?.get_name() ?? 'Janela', subtitle: app?.get_name() ?? '',
                    hint: 'Ir para', run: () => Main.activateWindow(w),
                });
            }
        }

        // Comandos do sistema
        for (const cmd of this._commands()) {
            const s = Math.max(matchScore(q, cmd.title), ...cmd.words.map(w => matchScore(q, w) * 0.9));
            if (s >= 40)
                items.push({...cmd, section: 'Comandos', score: s + 180 + this._usageBoost(cmd.key)});
        }

        // Histórico (quando combina) e arquivos (chegam um pouco depois)
        for (const h of this._async.history.slice(0, 3))
            items.push(this._historyItem(h, 120));
        for (const f of this._async.files.slice(0, 5)) {
            const s = matchScore(q, f.nome);
            items.push({
                id: `file:${f.uri}`, section: 'Arquivos', gicon: Gio.content_type_get_icon(Gio.content_type_guess(f.nome, null)[0]),
                score: 100 + s * 0.5, title: f.nome, subtitle: f.pasta, hint: 'Abrir',
                run: () => Gio.AppInfo.launch_default_for_uri(f.uri, global.create_app_launch_context(0, -1)),
                alt: () => this._showInFolder(f.uri),
            });
        }

        // Site digitado direto
        if (/^(https?:\/\/)?[\w-]+(\.[\w-]+)+(\/\S*)?$/.test(q) && !/^\d+([.,]\d+)*$/.test(q)) {
            const url = q.startsWith('http') ? q : `https://${q}`;
            items.push({
                id: 'url', section: 'Web', icon: 'web-browser-symbolic', score: 850, title: `Abrir ${url}`,
                subtitle: 'No navegador padrão', hint: 'Abrir',
                run: () => Gio.AppInfo.launch_default_for_uri(url, global.create_app_launch_context(0, -1)),
            });
        }

        items.sort((x, y) => y.score - x.score);
        // Agrupa por seção, na ordem em que cada seção aparece pela 1ª vez.
        const best = items.slice(0, LAUNCH_ROWS);
        const order = [...new Set(best.map(it => it.section))];
        const top = order.flatMap(sec => best.filter(it => it.section === sec));

        // Sempre por último: pesquisar na web e perguntar à IA.
        if (!prefixed)
            top.push(this._webItem(quick[0], q, 1));
        if (ql.split(/\s+/).length >= 3 || /\?$/.test(q)) {
            top.push({
                id: 'ia-fallback', section: 'Web', icon: 'starred-symbolic', score: 0,
                title: `Perguntar à IA: ${q}`, subtitle: 'Resposta do modelo local, aqui mesmo', hint: 'Perguntar',
                keepOpen: true, run: () => {
                    this._enterView('ia');
                    this._send(q);
                },
            });
        }
        return top;
    }

    _appItem(a, score, section) {
        return {
            id: `app:${a.get_id()}`, key: `app:${a.get_id()}`, section, gicon: a.get_icon(), icon: 'application-x-executable-symbolic',
            score, title: a.get_name(), subtitle: a.get_description?.() ?? '', hint: 'Abrir',
            run: () => {
                const app = Shell.AppSystem.get_default().lookup_app(a.get_id());
                if (app)
                    app.activate();
                else
                    a.launch([], global.create_app_launch_context(0, -1));
            },
            alt: () => Shell.AppSystem.get_default().lookup_app(a.get_id())?.open_new_window(-1),
        };
    }

    _historyItem(h, score) {
        const img = h.tipo === 'image';
        const ocr = (h.ocr || '').replace(/\s+/g, ' ').trim();
        const text = img ? `Imagem${ocr ? ` · ${ocr.slice(0, 70)}` : ''}` : h.texto.replace(/\s+/g, ' ');
        return {
            id: `cb:${h.id}`, data: h, section: 'Histórico do Clippit', icon: h.fixado ? 'view-pin-symbolic' : img ? 'image-x-generic-symbolic' : /^https?:\/\//.test(h.texto) ? 'web-browser-symbolic' : 'edit-paste-symbolic',
            score, title: text.slice(0, 90) || '(vazio)', subtitle: 'Histórico',
            hint: 'Colar',
            run: () => this._pasteHistory(h.id, true),
            alt: () => this._pasteHistory(h.id, false),
        };
    }

    async _deleteHistory(item) {
        const h = item?.data;
        if (!h)
            return;
        try {
            await runProcess([BUSCA_CMD, 'apagar', String(h.id)], null, 5);
        } catch (e) {
            Main.notify('Clippit', `Não consegui apagar: ${e.message}`);
            return;
        }
        this._async.history = this._async.history.filter(x => x.id !== h.id);
        this._refresh(true);
    }

    _toggleZoom() {
        this._zoom = !this._zoom;
        this._render();
    }

    _webItem(w, q, score) {
        return {
            id: `web:${w.key}`, key: `web:${w.key}`, section: 'Web', icon: w.icon ?? 'web-browser-symbolic', score,
            title: `Pesquisar “${q}” no ${w.name}`, subtitle: `Atalho: ${w.key} + espaço`, hint: 'Pesquisar',
            run: () => Gio.AppInfo.launch_default_for_uri(w.url.replace('{q}', encodeURIComponent(q)),
                global.create_app_launch_context(0, -1)),
        };
    }

    _quicklinks() {
        // Atalhos próprios em ~/.config/clippit/atalhos-web.json:
        // [{"atalho": "jira", "nome": "Jira", "url": "https://…?q={q}"}]
        let own = [];
        try {
            own = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(QUICKLINKS_FILE)[1]))
                .map(x => ({key: fold(x.atalho), name: x.nome, url: x.url, icon: 'web-browser-symbolic'}));
        } catch (_e) {}
        return [...WEB, ...own];
    }

    _commands() {
        const spawn = argv => () => Gio.Subprocess.new(argv, Gio.SubprocessFlags.NONE);
        return this._aiFilter([
            {id: 'cmd:historico', key: 'cmd:historico', icon: 'edit-paste-symbolic', title: 'Histórico da área de transferência',
                subtitle: 'Tudo o que você copiou, com prévia', words: ['historico', 'clipboard', 'copiados', 'area de transferencia', 'cb'],
                hint: 'Abrir', keepOpen: true, run: () => this._enterView('historico')},
            {id: 'cmd:ia', key: 'cmd:ia', icon: 'starred-symbolic', title: 'Perguntar à IA',
                subtitle: 'Vê o sistema, pesquisa, cotações, clima · Alt+I', words: ['ia', 'ai', 'chat', 'pergunta', 'assistente'],
                hint: 'Abrir', keepOpen: true, run: () => this._enterView('ia')},
            {id: 'cmd:conversas', key: 'cmd:conversas', icon: 'user-available-symbolic', title: 'Conversas anteriores com a IA',
                subtitle: 'Retomar uma conversa · Ctrl+H na IA', words: ['conversas', 'historico ia', 'chats'],
                hint: 'Abrir', keepOpen: true, run: () => {
                    this._convs = null;
                    this._enterView('conversas');
                }},
            ...this._controlCommands(),
            ...this._windowCommands(),
            {id: 'cmd:snippets', key: 'cmd:snippets', icon: 'insert-text-symbolic', title: 'Snippets (textos prontos)',
                subtitle: 'Digite ; para ver · ;gatilho = texto cria um', words: ['snippet', 'texto pronto', 'atalho de texto'],
                hint: 'Abrir', keepOpen: true, run: () => {
                    this._entry.text = ';';
                    this._entry.clutter_text.set_cursor_position(-1);
                }},
            {id: 'cmd:emoji', key: 'cmd:emoji', icon: 'face-smile-symbolic', title: 'Emoji e símbolos',
                subtitle: 'Digite : e o nome (ex.: :coração)', words: ['emoji', 'emoticon', 'simbolo'],
                hint: 'Abrir', keepOpen: true, run: () => this._enterView('emoji')},
            {id: 'cmd:acoes', key: 'cmd:acoes', icon: 'edit-select-all-symbolic', title: 'Ações no texto selecionado',
                subtitle: 'Resumir, traduzir, corrigir… · Alt+A', words: ['resumir', 'traduzir', 'texto selecionado'],
                hint: 'Abrir', keepOpen: true, run: () => this.openTextActions(true)},
            {id: 'cmd:lock', key: 'cmd:lock', icon: 'system-lock-screen-symbolic', title: 'Bloquear tela', subtitle: 'Sistema',
                words: ['travar', 'lock', 'bloquear'], hint: 'Executar', run: () => Main.screenShield.lock(true)},
            {id: 'cmd:suspend', key: 'cmd:suspend', icon: 'weather-clear-night-symbolic', title: 'Suspender', subtitle: 'Sistema',
                words: ['dormir', 'sleep', 'suspend'], hint: 'Executar', run: spawn(['systemctl', 'suspend'])},
            {id: 'cmd:reboot', key: 'cmd:reboot', icon: 'system-reboot-symbolic', title: 'Reiniciar…', subtitle: 'Pede confirmação',
                words: ['restart', 'reboot', 'reiniciar'], hint: 'Executar', run: spawn(['gnome-session-quit', '--reboot'])},
            {id: 'cmd:poweroff', key: 'cmd:poweroff', icon: 'system-shutdown-symbolic', title: 'Desligar…', subtitle: 'Pede confirmação',
                words: ['shutdown', 'desligar', 'power off'], hint: 'Executar', run: spawn(['gnome-session-quit', '--power-off'])},
            {id: 'cmd:logout', key: 'cmd:logout', icon: 'system-log-out-symbolic', title: 'Sair da sessão…', subtitle: 'Pede confirmação',
                words: ['logout', 'sair', 'encerrar sessão'], hint: 'Executar', run: spawn(['gnome-session-quit', '--logout'])},
            {id: 'cmd:trash', key: 'cmd:trash', icon: 'user-trash-full-symbolic', title: 'Esvaziar lixeira',
                subtitle: 'Apaga de vez os arquivos da lixeira (pede Enter duas vezes)', words: ['lixeira', 'trash', 'limpar lixo'],
                hint: 'Executar', confirm: 'Esvaziar a lixeira? Não dá para desfazer. Enter de novo para confirmar',
                run: () => {
                    Gio.Subprocess.new(['gio', 'trash', '--empty'], Gio.SubprocessFlags.NONE);
                    Main.notify('Clippit', 'Lixeira esvaziada.');
                }},
            {id: 'cmd:game', key: 'cmd:game', icon: 'input-gaming-symbolic', title: 'Modo jogo (liga/desliga)',
                subtitle: 'Tira a IA da placa de vídeo · Ctrl+Shift+E', words: ['jogo', 'game', 'liberar placa', 'vram'],
                hint: 'Executar', run: () => this._c._assistente?._toggleGameMode()},
            {id: 'cmd:clippit', key: 'cmd:clippit', icon: 'preferences-system-symbolic', title: 'Configurações do Clippit',
                subtitle: 'Corretor, assistente, aprendizado', words: ['clippit', 'painel', 'dashboard'], hint: 'Abrir',
                run: spawn([clippitTool('clippit-dashboard')])},
            {id: 'cmd:settings', key: 'cmd:settings', icon: 'org.gnome.Settings-symbolic', title: 'Configurações do sistema',
                subtitle: 'GNOME', words: ['settings', 'configurações', 'ajustes'], hint: 'Abrir', run: spawn(['gnome-control-center'])},
            {id: 'cmd:corretor', key: 'cmd:corretor', icon: 'tools-check-spelling-symbolic', title: 'Rodar a prova do corretor',
                subtitle: 'Mede a qualidade (uns 50 s)', words: ['prova', 'teste corretor', 'qualidade'], hint: 'Executar',
                run: async () => {
                    Main.notify('Clippit', 'Rodando a prova do corretor…');
                    const out = await runProcess([CORRECTOR_CMD, '--prova'], null, 600);
                    Main.notify('Clippit — prova do corretor', out.split('\n').slice(1, 3).join(' ').trim());
                }},
        ]);
    }

    // ---- Ações -------------------------------------------------------------

    async _pasteHistory(id, paste) {
        try {
            await runProcess([BUSCA_CMD, 'copiar', String(id)], null, 5);
        } catch (e) {
            Main.notify('Clippit', `Não consegui copiar: ${e.message}`);
            return;
        }
        if (!paste) {
            Main.notify('Clippit', 'Copiado para a área de transferência.');
            return;
        }
        // Volta para a janela de antes e cola.
        const win = this._prevWin;
        if (win && global.display.focus_window !== win)
            win.activate(global.get_current_time());
        await sleep(250);
        this._vk ??= Clutter.get_default_backend().get_default_seat()
            .create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
        const key = (kv, down) => this._vk.notify_keyval(GLib.get_monotonic_time(), kv,
            down ? Clutter.KeyState.PRESSED : Clutter.KeyState.RELEASED);
        key(Clutter.KEY_Control_L, true);
        key(Clutter.KEY_v, true);
        key(Clutter.KEY_v, false);
        key(Clutter.KEY_Control_L, false);
    }

    _showInFolder(uri) {
        Gio.DBus.session.call('org.freedesktop.FileManager1', '/org/freedesktop/FileManager1',
            'org.freedesktop.FileManager1', 'ShowItems', new GLib.Variant('(ass)', [[uri], '']),
            null, Gio.DBusCallFlags.NONE, -1, null, null);
    }

    // ---- Chat com a IA -----------------------------------------------------

    _renderChat() {
        this._scroll.hide();
        this._detailScroll.hide();
        this._vsep.hide();
        this._answerScroll.show();
        this._chatEmpty.hide();
        // Conversa nova depois de 30 min sem uso (a antiga fica em Ctrl+H).
        if (this._chatTime && (GLib.get_monotonic_time() - this._chatTime) / 1e6 > 1800 && !this._chat?.some(m => m.pending)) {
            this._chat = [];
            this._chatId = null;
            this._chatTime = 0;
        }
        this._chat ??= [];
        this._doctorInbox();
        const adjB = this._chatScrollBox?.vadjustment;
        this._chatFollow = !adjB || adjB.value + adjB.page_size >= adjB.upper - 60 || this._chat.at(-1)?.pending;
        this._chatBox.destroy_all_children();
        this._stepsBox = null;
        if (!this._chat.length) {
            this._chatEmpty.get_child()?.destroy();
            this._chatEmpty.set_child(emptyState('sparkles', '#3b82f6', 'Pergunte qualquer coisa',
                'A IA vê o seu computador, cotações, clima, notícias e a web, e lembra da conversa.\n' +
                '@ anexa app, arquivo ou print · Alt+V para falar'));
            this._answerScroll.hide();
            this._chatEmpty.show();
        }
        for (const m of this._chat) {
            if (m.role === 'user') {
                const bubble = new St.Label({text: m.text, style: LSTYLE.bubble, x_align: Clutter.ActorAlign.END});
                if (m.attachments?.length) {
                    let mk = esc(m.text);
                    for (const a of m.attachments)
                        mk = mk.split(esc(`@${a.nome}`)).join(`<span foreground="${C.accent}" weight="bold">${esc(`@${a.nome}`)}</span>`);
                    bubble.clutter_text.set_markup(mk);
                }
                bubble.clutter_text.line_wrap = true;
                bubble.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
                this._chatBox.add_child(bubble);
                continue;
            }
            const block = new St.BoxLayout({vertical: true, style: LSTYLE.aiBlock});
            const steps = new St.BoxLayout({vertical: true, style: LSTYLE.steps});
            block.add_child(steps);
            if (m.pending) {
                this._stepsBox = steps; // atualizado ao vivo
                this._stepsKey = null;
                const live = m.live ?? {passos: [], pensando: true};
                this._renderSteps({...live, pensando: live.pensando !== false && !live.texto}, m.elapsed ?? 0);
                // A resposta aparece enquanto o modelo escreve.
                this._liveText = new St.Label({style: LSTYLE.aiText});
                this._liveText.clutter_text.line_wrap = true;
                this._liveText.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
                this._liveShown = live.texto ?? '';
                if (this._liveShown)
                    this._liveText.clutter_text.set_markup(mdToMarkup(this._liveShown));
                block.add_child(this._liveText);
            } else {
                this._stepsBox = steps;
                this._stepsKey = null;
                this._renderSteps({passos: m.steps ?? [], pensando: false}, 0, true);
                const text = new St.Label({style: LSTYLE.aiText});
                text.clutter_text.line_wrap = true;
                text.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
                const sources = m.sources?.length
                    ? `\n\n<span foreground="${C.muted}" size="small">Fontes</span>\n${m.sources.map(u =>
                        `<span foreground="${C.accent}" size="small">${esc(u)}</span>`).join('\n')}` : '';
                for (const o of m.outputs ?? [])
                    block.add_child(this._outputBox(o));
                text.clutter_text.set_markup(`${mdToMarkup(m.text)}${sources}`);
                block.add_child(text);
                if (m.confirm)
                    block.add_child(this._confirmBox(m));
                if (m.choice)
                    block.add_child(this._choiceBox(m));
                this._stepsBox = null;
            }
            this._chatBox.add_child(block);
        }
        const busy = this._chat.some(m => m.pending);
        const last = [...this._chat].reverse().find(m => m.role === 'assistant' && !m.pending);
        const asking = last?.confirm && !last.confirmState;
        this._setFooter('IA local', 'starred-symbolic', busy ? [{label: 'Parar', key: 'Esc', run: () => this._stopAsk()}]
            : asking ? [{label: 'Rodar', key: '↵'}, {label: 'Cancelar', key: 'Esc'}]
                : [{label: 'Enviar', key: '↵'}, last?.sources?.length && {label: 'Abrir fonte', key: 'Ctrl ↵'},
                    {label: 'Nova', key: 'Ctrl N'}, {label: 'Conversas', key: 'Ctrl H'}, {label: 'Copiados', key: 'Alt C'}]);
        this._scrollChatToEnd();
    }

    // Saída de um comando que a IA rodou (no fundo, sem terminal).
    _outputBox(o) {
        const box = new St.BoxLayout({vertical: true, style: LSTYLE.outBox});
        const cmd = new St.Label({text: `$ ${o.comando}`, style: LSTYLE.outCmd});
        cmd.clutter_text.line_wrap = true;
        cmd.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        box.add_child(cmd);
        let lines = (o.saida || '(sem saída)').split('\n');
        if (lines.length > 14)
            lines = [...lines.slice(0, 6), `… ${lines.length - 12} linhas …`, ...lines.slice(-6)];
        const out = new St.Label({text: lines.join('\n'), style: LSTYLE.outText});
        out.clutter_text.line_wrap = true;
        out.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        box.add_child(out);
        box.add_child(new St.Label({
            text: `${o.codigo === 0 ? 'concluído' : `saiu com código ${o.codigo}`} · ${String(o.segundos ?? 0).replace('.', ',')} s`,
            style: `${LSTYLE.outCode} ${o.codigo === 0 ? '' : `color: ${C.danger};`}`,
        }));
        return box;
    }

    // Dúvida da IA: opções A, B, C… (setas + Enter, clique, ou escreva outra coisa).
    _choiceBox(m) {
        const box = new St.BoxLayout({vertical: true, style: LSTYLE.choiceBox});
        const open = m.choiceAnswered === undefined;
        m._rows = m.choice.opcoes.map((o, i) => {
            const chosen = m.choiceAnswered === i;
            const row = new St.BoxLayout({style: open && i === m.choiceSel ? LSTYLE.choiceOn : LSTYLE.choice,
                reactive: open, track_hover: open, opacity: open || chosen ? 255 : 110});
            row.add_child(new St.Label({text: chosen ? '✓' : LETTERS[i], style: LSTYLE.choiceKey, y_align: Clutter.ActorAlign.CENTER}));
            const t = new St.Label({text: o, style: LSTYLE.choiceText, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            t.clutter_text.line_wrap = true;
            row.add_child(t);
            if (open) {
                row.connect('button-press-event', () => {
                    this._answerChoice(m, i);
                    return Clutter.EVENT_STOP;
                });
                row.connect('notify::hover', () => {
                    if (row.hover && m.choiceSel !== i)
                        this._moveChoice(m, i - m.choiceSel);
                });
            }
            box.add_child(row);
            return row;
        });
        if (open)
            box.add_child(new St.Label({text: '↑↓ escolher · Enter responde · ou escreva outra resposta', style: LSTYLE.outCode}));
        else if (m.choiceAnswered === 'texto')
            box.add_child(new St.Label({text: 'Respondido com outro texto.', style: LSTYLE.outCode}));
        return box;
    }

    _pendingChoice() {
        const last = [...(this._chat ?? [])].reverse().find(x => x.role === 'assistant' && !x.pending);
        return last?.choice && last.choiceAnswered === undefined && last === this._chat.at(-1) ? last : null;
    }

    _moveChoice(m, d) {
        const n = m.choice.opcoes.length;
        m.choiceSel = Math.max(0, Math.min(n - 1, m.choiceSel + d));
        (m._rows ?? []).forEach((r, i) => (r.style = i === m.choiceSel ? LSTYLE.choiceOn : LSTYLE.choice));
    }

    _answerChoice(m, i) {
        if (m.choiceAnswered !== undefined || this._chat.some(x => x.pending))
            return;
        m.choiceAnswered = i;
        this._send(m.choice.opcoes[i], {choiceIndex: i});
    }

    // Comando que muda algo: mostra e espera o "Rodar" (ou Enter).
    _confirmBox(m) {
        const wrap = new St.BoxLayout({vertical: true, style: 'spacing: 6px;'});
        const box = new St.BoxLayout({vertical: true, style: LSTYLE.outBox});
        const cmd = new St.Label({text: `$ ${m.confirm}`, style: LSTYLE.outCmd});
        cmd.clutter_text.line_wrap = true;
        cmd.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        box.add_child(cmd);
        wrap.add_child(box);
        if (m.confirmState) {
            wrap.add_child(new St.Label({text: m.confirmState === 'ok' ? 'Autorizado.' : 'Cancelado.', style: LSTYLE.outCode}));
            return wrap;
        }
        const row = new St.BoxLayout({style: LSTYLE.confirmRow});
        row.add_child(textButton('Rodar  ↵', LSTYLE.runBtn, LSTYLE.runBtnHover, () => this._confirm(m, true)));
        row.add_child(textButton('Cancelar', LSTYLE.cancelBtn, LSTYLE.cancelBtnHover, () => this._confirm(m, false)));
        wrap.add_child(row);
        return wrap;
    }

    _confirm(m, run) {
        if (m.confirmState || this._chat.some(x => x.pending))
            return;
        m.confirmState = run ? 'ok' : 'no';
        if (!run) {
            this._renderChat();
            return;
        }
        // Repete a pergunta que pediu o comando, agora com ele autorizado.
        const i = this._chat.indexOf(m);
        const asked = this._chat.slice(0, i).reverse().find(x => x.role === 'user');
        this._send(asked?.text ?? '', {confirmed: m.confirm, upTo: this._chat.indexOf(asked)});
    }

    // Abre o app/site que a IA pediu pelo próprio GNOME (assim a janela nova
    // ganha o foco) e fecha o lançador.
    _runActions(actions) {
        let opened = false;
        for (const a of actions ?? []) {
            try {
                if (a.tipo === 'app') {
                    const app = Shell.AppSystem.get_default().lookup_app(a.id);
                    if (app) {
                        app.activate();
                        opened = true;
                    }
                } else if (a.tipo === 'janela' && a.id) {
                    const w = this._normalWindows().find(x => String(x.get_id()) === a.id);
                    if (w) {
                        this._prevWin = w; // o lançador não devolve o foco para a antiga
                        w.activate(global.get_current_time());
                        opened = true;
                    }
                } else if (a.tipo === 'escrever' && a.texto) {
                    // Fecha o lançador, volta para o campo de trás e cola o texto.
                    this._pasteText(a.texto);
                    opened = true;
                } else if (a.tipo === 'uri' && a.uri) {
                    // No app pedido (ex.: link no Chrome) ou no padrão do sistema.
                    const info = a.app ? Gio.DesktopAppInfo.new(a.app) : null;
                    if (info)
                        info.launch_uris([a.uri], global.create_app_launch_context(0, -1));
                    else
                        Gio.AppInfo.launch_default_for_uri(a.uri, global.create_app_launch_context(0, -1));
                    opened = true;
                }
            } catch (e) {
                console.log(`${TAG} lançador: não abriu ${JSON.stringify(a)}: ${e.message}`);
            }
        }
        return opened;
    }

    _scrollChatToEnd(force = false) {
        // Só acompanha o fim se você já estava lá embaixo (rolando para cima
        // para ler, a tela não é puxada de volta).
        const adj0 = this._chatScrollBox?.vadjustment;
        const atEnd = !adj0 || adj0.value + adj0.page_size >= adj0.upper - 60;
        if (!force && !atEnd && !this._chatFollow)
            return;
        this._chatFollow = false;
        if (this._endIdle)
            return; // já marcado para depois do layout
        this._endIdle = GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._endIdle = 0;
            if (!this._panel || this._view !== 'ia')
                return GLib.SOURCE_REMOVE; // janela fechada nesse meio-tempo
            // Desce depois do novo tamanho valer (próximo quadro).
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 20, () => {
                const adj = this._panel && this._view === 'ia' ? this._chatScrollBox?.vadjustment : null;
                if (adj) {
                    const end = Math.max(adj.lower, adj.upper - adj.page_size);
                    if (Math.abs(end - adj.value) > 400)
                        adj.value = end; // muito longe: vai direto
                    else
                        adj.ease(end, {duration: 160, mode: Clutter.AnimationMode.EASE_OUT_CUBIC});
                }
                return GLib.SOURCE_REMOVE;
            });
            return GLib.SOURCE_REMOVE;
        });
    }

    async _send(question, opts = {}) {
        this._chat ??= [];
        if (this._chat.some(m => m.pending))
            return; // uma pergunta por vez
        // Escrever outra coisa no lugar de "Rodar" cancela o comando pendente.
        for (const m of this._chat) {
            if (m.confirm && !m.confirmState && !opts.confirmed)
                m.confirmState = 'no';
            if (m.choice && m.choiceAnswered === undefined)
                m.choiceAnswered = opts.choiceIndex ?? 'texto';
        }
        if (this._debounceId) {
            GLib.source_remove(this._debounceId);
            this._debounceId = 0;
        }
        const upTo = opts.confirmed ? Math.max(0, opts.upTo ?? 0) : this._chat.length;
        const history = this._chat.slice(0, upTo).filter(m => !m.pending && m.text)
            .map(m => ({role: m.role, content: m.choice
                ? `${m.text}\n${m.choice.opcoes.map((o, i) => `${LETTERS[i]}) ${o}`).join('\n')}` : m.text}));
        // Anexos (@app, @arquivo) que continuam no texto da pergunta.
        const asked = opts.confirmed ? this._chat[opts.upTo] : null;
        const attachments = opts.confirmed ? (asked?.attachments ?? [])
            : (this._attach ?? []).filter(a => question.includes(`@${a.nome}`));
        if (!opts.confirmed) {
            this._attach = [];
            this._chat.push({role: 'user', text: question, attachments});
        }
        const reply = {role: 'assistant', pending: true, live: {passos: [], pensando: true}, elapsed: 0};
        this._chat.push(reply);
        this._chatTime = GLib.get_monotonic_time();
        if (!opts.confirmed)
            this._entry.text = ''; // a pergunta vai para a conversa
        this._renderChat();

        const gen = this._askGen = (this._askGen ?? 0) + 1;
        const statusFile = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-correct', 'agente-status']);
        try {
            GLib.file_set_contents(statusFile, '');
        } catch (_e) {}
        const started = GLib.get_monotonic_time();
        let pollAlive = true; // remover uma fonte que já saiu sozinha gera aviso do GLib
        const pollId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 250, () => {
            if (gen !== this._askGen || !reply.pending) {
                pollAlive = false;
                return GLib.SOURCE_REMOVE;
            }
            try {
                const raw = new TextDecoder().decode(GLib.file_get_contents(statusFile)[1]).trim();
                if (raw)
                    reply.live = JSON.parse(raw);
            } catch (_e) {}
            reply.elapsed = (GLib.get_monotonic_time() - started) / 1e6;
            if (this._panel && this._view === 'ia' && this._stepsBox) {
                const before = this._stepsBox.get_n_children();
                const live = reply.live ?? {};
                this._renderSteps({...live, pensando: live.pensando !== false && !live.texto}, reply.elapsed);
                let grew = this._stepsBox.get_n_children() !== before;
                if (this._liveText && (live.texto ?? '') !== this._liveShown) {
                    this._liveShown = live.texto ?? '';
                    try {
                        this._liveText.clutter_text.set_markup(mdToMarkup(this._liveShown));
                    } catch (_e) {
                        this._liveText.text = this._liveShown;
                    }
                    grew = true;
                }
                if (grew)
                    this._scrollChatToEnd();
            }
            return GLib.SOURCE_CONTINUE;
        });
        try {
            const raw = (await runProcess([CORRECTOR_CMD, '--perguntar'],
                JSON.stringify({pergunta: question, historico: history, confirmado: opts.confirmed ?? null,
                    anexos: attachments}), 240)).trim();
            if (reply.stopped)
                return; // você apertou Parar: a resposta que chegou depois não vale
            let data = {resposta: raw, passos: [], fontes: []};
            try {
                data = JSON.parse(raw);
            } catch (_e) {}
            let last = {};
            try {
                last = JSON.parse(new TextDecoder().decode(GLib.file_get_contents(statusFile)[1]).trim() || '{}');
            } catch (_e) {}
            Object.assign(reply, {pending: false, text: data.resposta, steps: last.passos ?? [], sources: data.fontes ?? [],
                outputs: data.saidas ?? [], confirm: data.confirmar ?? null, choice: data.escolha ?? null, choiceSel: 0});
            this._saveChat();
            if (data.acoes?.length && this._runActions(data.acoes)) {
                // Abriu um app/site: sai da frente para ele ganhar o foco
                // (a conversa continua aqui na próxima vez).
                this._chatTime = GLib.get_monotonic_time();
                if (this._panel)
                    this._close();
                return;
            }
        } catch (e) {
            if (reply.stopped)
                return;
            Object.assign(reply, {pending: false, text: `A IA não respondeu: ${e.message}`, steps: []});
            this._saveChat();
        } finally {
            if (pollAlive)
                GLib.source_remove(pollId);
        }
        this._chatTime = GLib.get_monotonic_time();
        if (this._panel && this._view === 'ia')
            this._renderChat();
    }

    // Parar: avisa o servidor (ele para no próximo pedaço: resposta,
    // ferramenta ou comando) e encerra a resposta na tela na hora.
    _stopAsk() {
        const reply = this._chat?.find(m => m.pending);
        if (!reply)
            return;
        try {
            const dir = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-correct']);
            GLib.mkdir_with_parents(dir, 0o700);
            GLib.file_set_contents(GLib.build_filenamev([dir, 'cancelar']), '1');
        } catch (e) {
            console.log(`${TAG} lançador: não consegui pedir para parar: ${e.message}`);
        }
        this._askGen = (this._askGen ?? 0) + 1; // o acompanhamento ao vivo para aqui
        const done = (reply.live?.passos ?? []).map(p => (p.estado === 'rodando' ? {...p, estado: 'erro', resumo: 'parado'} : p));
        Object.assign(reply, {pending: false, stopped: true, text: reply.live?.texto ? `${reply.live.texto}\n\n*Parado.*` : '*Parado.*',
            steps: done});
        this._saveChat();
        if (this._panel && this._view === 'ia')
            this._renderChat();
    }

    // Uma linha por ferramenta: círculo girando enquanto roda, ✓ e o tempo
    // quando termina, com o detalhe (o que pesquisou, que página…) apagado.
    _renderSteps(st, elapsed, final = false) {
        const secs = Math.floor(elapsed);
        const thinkText = (st.passos ?? []).length ? 'Juntando o que encontrei…' : `Pensando${secs > 2 ? ` · ${secs} s` : '…'}`;
        // O texto ao vivo não entra na chave (senão os círculos recomeçam a cada letra).
        const key = JSON.stringify({p: st.passos, t: st.pensando}) + (final ? 'f' : '');
        if (key === this._stepsKey) {
            // Só o tempo mudou: troca o texto, sem recriar os círculos girando.
            if (this._thinkLabel)
                this._thinkLabel.text = thinkText;
            return;
        }
        this._thinkLabel = null;
        this._stepsKey = key;
        this._stepsBox.destroy_all_children();
        const steps = st.passos ?? [];
        for (const p of steps) {
            const row = new St.BoxLayout({style: LSTYLE.step});
            let icon;
            if (p.estado === 'rodando') {
                const ring = makeRing(14);
                ring.start();
                icon = ring.actor;
            } else {
                const [n, c] = {erro: ['triangle-alert', 'err'], espera: ['circle-help', 'acc']}[p.estado] ?? ['check', 'ok'];
                icon = new St.Icon({gicon: lucide(n, c), icon_size: 14});
            }
            icon.y_align = Clutter.ActorAlign.CENTER;
            row.add_child(icon);
            row.add_child(new St.Label({text: p.titulo, style: LSTYLE.stepTitle, y_align: Clutter.ActorAlign.CENTER}));
            const detail = new St.Label({text: p.detalhe || p.resumo || '', style: LSTYLE.stepDetail,
                x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            detail.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            row.add_child(detail);
            if (p.segundos !== undefined)
                row.add_child(new St.Label({text: `${String(p.segundos).replace('.', ',')} s`, style: LSTYLE.stepTime,
                    y_align: Clutter.ActorAlign.CENTER}));
            this._stepsBox.add_child(row);
        }
        if (!final && st.pensando !== false) {
            const row = new St.BoxLayout({style: 'spacing: 10px; padding: 5px 10px;'});
            const ring = makeRing(14);
            ring.start();
            ring.actor.y_align = Clutter.ActorAlign.CENTER;
            row.add_child(ring.actor);
            this._thinkLabel = new St.Label({text: thinkText, style: LSTYLE.thinking, y_align: Clutter.ActorAlign.CENTER});
            row.add_child(this._thinkLabel);
            this._stepsBox.add_child(row);
        }
        this._stepsBox.visible = this._stepsBox.get_n_children() > 0;
    }

    // ---- Desenho -------------------------------------------------------------

    _setFooter(where, icon, actions = []) {
        this._foot?.set(where, icon, actions);
    }

    _render() {
        if (!this._panel)
            return;
        if (this._view === 'ia') {
            this._renderChat();
            return;
        }
        if (this._view === 'texto' && this._tx) {
            this._renderTx();
            return;
        }
        if (this._view === 'emoji') {
            this._renderEmoji();
            return;
        }
        this._answerScroll.hide();
        this._chatEmpty.hide();
        this._scroll.show();
        const cbMode = this._isHistory();
        // Histórico: duas colunas com prévia, como o Clipboard History do Raycast.
        if (cbMode !== this._wasHistory) {
            this._wasHistory = cbMode;
            this._applySize(); // "cb" digitado também aumenta a janela
        }
        const W = this._W ?? LAUNCH_WIDTH;
        this._detailScroll.visible = cbMode;
        this._vsep.visible = cbMode && !this._zoom;
        // Ampliado: a prévia ocupa a janela toda.
        this._scroll.visible = !(cbMode && this._zoom);
        this._detailScroll.set_width(cbMode && this._zoom ? W : Math.round(W * 0.6));
        this._back.visible = this._view !== 'root' || cbMode;
        this._modeLabel.text = cbMode ? 'Histórico' : {ia: 'IA local', conversas: 'Conversas', texto: 'Texto selecionado'}[this._view] ?? '';
        this._list.destroy_all_children();
        this._rows = [];
        if (!this._shown.length) {
            this._list.add_child(emptyState('search', '#6b7280', 'Nada encontrado', 'Tente outra palavra.'));
            this._paint();
            return;
        }
        let lastSection = null;
        this._shown.forEach((it, i) => {
            if (it.section && it.section !== lastSection && !cbMode) {
                this._list.add_child(new St.Label({text: it.section, style: LSTYLE.section}));
                lastSection = it.section;
            }
            const row = new St.BoxLayout({style: LSTYLE.row, reactive: true, track_hover: true});
            let icon;
            const color = it.data && HEX_COLOR.test(it.data.texto?.trim() ?? '') ? it.data.texto.trim() : null;
            if (color) {
                icon = new St.Widget({style: `${LSTYLE.swatch} background-color: ${color};`});
            } else if (it.emoji) {
                icon = new St.Label({text: it.emoji, style: 'font-size: 15pt; min-width: 22px;'});
            } else if (it.gicon) {
                icon = new St.Icon({gicon: it.gicon, icon_size: 22});
            } else if (cbMode && it.data?.tipo) {
                // Histórico: ícone fino e cinza (texto, link, imagem, fixado).
                const h = it.data;
                const [n, c] = h.fixado ? ['pin', 'acc'] : h.tipo === 'image' ? ['image', 'g']
                    : /^https?:\/\//.test(h.texto ?? '') ? ['link', 'g'] : ['type', 'g'];
                icon = new St.Icon({gicon: lucide(n, c), icon_size: 16});
            } else {
                const [n, c] = ICON_BY_ID[it.id] ?? ICON_MAP[it.icon] ?? ['app-window', '#6b7280'];
                icon = iconTile(n, c);
            }
            icon.y_align = Clutter.ActorAlign.CENTER;
            row.add_child(icon);
            // Título e subtítulo na mesma linha, o subtítulo apagado.
            const line = new St.Label({x_expand: true, y_align: Clutter.ActorAlign.CENTER});
            line.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            const sub = !cbMode && it.subtitle ? `   <span foreground="${C.muted}" size="9728">${esc(it.subtitle)}</span>` : '';
            line.clutter_text.set_markup(`<span foreground="${C.text}">${esc(it.title)}</span>${sub}`);
            line.style = LSTYLE.title;
            row.add_child(line);
            if (!cbMode)
                row.add_child(new St.Label({text: KIND[it.section] ?? '', style: LSTYLE.kind, y_align: Clutter.ActorAlign.CENTER}));
            row.connect('motion-event', () => {
                if (this._sel !== i) {
                    this._sel = i;
                    this._paint(false);
                }
                return Clutter.EVENT_PROPAGATE;
            });
            row.connect('button-release-event', () => {
                this._activate(it, false);
                return Clutter.EVENT_STOP;
            });
            this._list.add_child(row);
            this._rows.push(row);
        });
        this._paint();
    }

    _paint(scroll = true) {
        (this._rows ?? []).forEach((row, i) => {
            row.style = i === this._sel ? LSTYLE.rowOn : LSTYLE.row;
        });
        const it = this._shown?.[this._sel];
        const cbMode = this._isHistory();
        if (cbMode && this._cbTx)
            this._setFooter('Transformar', 'edit-paste-symbolic', [{label: 'Colar', key: '↵'}, {label: 'Copiar', key: 'Ctrl ↵'},
                {label: 'Voltar', key: 'Tab'}]);
        else if (cbMode)
            this._setFooter('Histórico', 'edit-paste-symbolic', [{label: 'Colar', key: '↵'}, {label: 'Copiar', key: 'Ctrl ↵'},
                {label: 'Transformar', key: 'Tab'}, {label: 'Fixar', key: 'Ctrl P'}, {label: 'Apagar', key: 'Ctrl Del'},
                {label: 'Ampliar', key: 'Ctrl Espaço'}]);
        else if (this._view === 'conversas')
            this._setFooter('Conversas', 'user-available-symbolic', it ? [{label: 'Abrir', key: '↵'}, {label: 'Apagar', key: 'Ctrl Del'}] : []);
        else if (it)
            this._setFooter(this._view === 'texto' ? 'Texto selecionado' : 'Clippit',
                this._view === 'texto' ? 'edit-select-all-symbolic' : 'edit-find-symbolic',
                [{label: it.hint ?? 'Abrir', key: '↵'}, it.alt && {label: 'Ação extra', key: 'Ctrl ↵'},
                    this._view === 'texto' && {label: 'Voltar', key: '⌫'}]);
        else
            this._setFooter('Clippit', 'edit-find-symbolic', []);
        if (cbMode)
            this._showDetail(it);
        // Mantém a linha escolhida à vista.
        const row = this._rows?.[this._sel];
        if (scroll && row) {
            // A caixa dentro da rolagem é quem tem o ajuste vertical.
            const adj = this._list.vadjustment;
            let box;
            try {
                box = row.get_allocation_box();
            } catch (_e) {
                return; // a lista foi redesenhada e essa linha já não existe
            }
            if (adj && box.y2 > 0) {
                if (box.y1 < adj.value)
                    adj.value = Math.max(0, box.y1 - 8);
                else if (box.y2 > adj.value + adj.page_size)
                    adj.value = box.y2 - adj.page_size + 8;
            }
        }
    }

    // Prévia do item do histórico: cor, imagem ou texto + informações.
    _showDetail(it) {
        this._detail.destroy_all_children();
        const h = it?.data;
        if (!h)
            return;
        const text = h.texto?.trim() ?? '';
        const color = HEX_COLOR.test(text) ? text : null;
        if (color) {
            const card = new St.BoxLayout({vertical: true, x_align: Clutter.ActorAlign.CENTER,
                style: 'background-color: #ffffff; border-radius: 10px; padding: 6px; spacing: 6px;'});
            card.add_child(new St.Widget({style: `background-color: ${color}; border-radius: 6px; width: 180px; height: 120px;`}));
            card.add_child(new St.Label({text: color.toUpperCase(), style: `${TX.label} color: #333333;`}));
            this._detail.add_child(card);
        } else if (h.tipo === 'image' && h.imagem) {
            const img = new St.Widget({
                style: `background-image: url("file://${h.imagem}"); background-size: contain;
                    background-position: center; border-radius: 8px;`,
                x_expand: true, height: this._zoom ? 470 : 320,
            });
            this._detail.add_child(img);
            if (h.ocr && h.ocr.length > 15 && !this._zoom) {
                const o = new St.Label({text: h.ocr.slice(0, 400), style: TX.small});
                o.clutter_text.line_wrap = true;
                o.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
                this._detail.add_child(o);
            }
        } else {
            const label = new St.Label({text: text.slice(0, this._zoom ? 2000 : 1200), style: LSTYLE.preview, y_expand: true});
            label.clutter_text.line_wrap = true;
            label.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
            this._detail.add_child(label);
        }
        this._detail.add_child(new St.Widget({y_expand: true}));
        if (this._zoom)
            return; // ampliado: só a prévia
        this._detail.add_child(new St.Label({text: 'Informações', style: LSTYLE.infoTitle}));
        const kind = color ? 'Cor' : h.tipo === 'image' ? 'Imagem' : /^https?:\/\//.test(text) ? 'Link' : 'Texto';
        const info = [['Tipo', kind], ['Copiado', (h.quando ?? '').replace('T', ' ').slice(0, 16)]];
        if (kind === 'Texto')
            info.push(['Caracteres', String([...text].length)]);
        if (h.tipo === 'image' && h.ocr)
            info.push(['Texto na imagem', `${[...h.ocr].length} caracteres`]);
        for (const [k, v] of info) {
            const r = new St.BoxLayout({style: LSTYLE.infoRow});
            r.add_child(new St.Label({text: k, style: LSTYLE.infoKey, x_expand: true}));
            r.add_child(new St.Label({text: v, style: LSTYLE.infoVal}));
            this._detail.add_child(r);
        }
    }
}

export class Corretor {
    enable() {
        this._im = Main.inputMethod;
        this._cache = new Map();      // texto inteiro (flat) → corrigido, depois de aplicar
        this._chunkCache = new Map(); // trecho → trecho corrigido
        this._queue = [];
        this._working = false;
        this._inFlight = null;
        this._ignored = new Set();    // chaves "de→para" ignoradas
        this._checkId = 0;
        this._watchId = 0;
        this._hideId = 0;
        this._reqId = 0;
        this._state = 'idle';
        this._result = null;          // {text, corrected, sugs}

        // Selo: (carregando)  [N sugestões]  [✓]
        // O carregando e o ✓ ficam fora do cartão, um de cada lado.
        this._badge = new St.BoxLayout({style: 'spacing: 6px;', visible: false});

        // Sempre ocupa o espaço (só fica transparente) para o selo não pular de lugar.
        this._ringBox = new St.Bin({style: STYLE.ring, y_align: Clutter.ActorAlign.CENTER, opacity: 0});
        this._ringBox.set_size(BADGE_H, BADGE_H);
        this._ring = new St.DrawingArea({width: 14, height: 14, x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER});
        this._ring.connect('repaint', area => {
            const cr = area.get_context();
            const [w, h] = area.get_surface_size();
            const r = Math.min(w, h) / 2 - 1.5;
            cr.setLineWidth(2);
            cr.setSourceRGBA(1, 1, 1, 0.12);
            cr.arc(w / 2, h / 2, r, 0, 2 * Math.PI);
            cr.stroke();
            cr.setSourceRGBA(0.298, 0.616, 1, 1); // azul do Clippit
            cr.arc(w / 2, h / 2, r, -Math.PI / 2, Math.PI / 4);
            cr.stroke();
            cr.$dispose();
        });
        this._ring.set_pivot_point(0.5, 0.5);
        this._ringBox.set_child(this._ring);
        this._badge.add_child(this._ringBox);

        this._badgeBtn = new St.Button({can_focus: false, track_hover: true, reactive: true, y_align: Clutter.ActorAlign.CENTER});
        this._badgeLabel = new St.Label({y_align: Clutter.ActorAlign.CENTER, x_align: Clutter.ActorAlign.CENTER});
        this._badgeBtn.set_child(this._badgeLabel);
        this._badgeBtn.set_height(BADGE_H);
        this._badgeBtn.connect('clicked', () => this._onBadgeClicked());
        this._badgeBtn.connect('notify::hover', () => this._paintBadge());
        this._badge.add_child(this._badgeBtn);

        this._fixBtn = new St.Button({
            can_focus: false, track_hover: true, reactive: true, visible: false, style: STYLE.fix,
            y_align: Clutter.ActorAlign.CENTER,
            child: new St.Icon({
                icon_name: 'object-select-symbolic', icon_size: 14,
                x_align: Clutter.ActorAlign.CENTER, y_align: Clutter.ActorAlign.CENTER,
            }),
        });
        this._fixBtn.set_size(Math.round(BADGE_H * 1.5), BADGE_H); // mais largo que alto
        this._fixBtn.connect('notify::hover', () => {
            this._fixBtn.style = this._fixBtn.hover ? STYLE.fixHover : STYLE.fix;
        });
        this._fixBtn.connect('clicked', () => this._apply(this._pending()));
        this._badge.add_child(this._fixBtn);

        this._badge.connect('notify::visible', () => this._updateKeyGrabs());
        Main.layoutManager.addTopChrome(this._badge);

        this._im.connectObject('surrounding-text-set', () => this._onSurroundingChanged(), this);
        this._assistente = new Assistente(this);
        this._assistente.enable();
        this._lancador = new Lancador(this);
        this._lancador.enable();
        global.display.connectObject('notify::focus-window', () => this._reset(), this);
        console.log(`${TAG} habilitada`);
        // Foto do painel do corretor com sugestões de exemplo (só para conferir o visual).
        const probe = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'clippit-corretor', 'selftest-corretor-foto']);
        if (GLib.file_test(probe, GLib.FileTest.EXISTS)) {
            const dir = new TextDecoder().decode(GLib.file_get_contents(probe)[1]).trim();
            GLib.unlink(probe);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 700, () => {
                try {
                    this._result = {text: 'quais processo estao masi consumindo', sugs: [
                        {key: 'processo→processos', from: 'processo', to: 'processos', before: 'quais ', after: ' estao masi'},
                        {key: 'estao→estão', from: 'estao', to: 'estão', before: 'processo ', after: ' masi consumindo'},
                        {key: 'masi→mais', from: 'masi', to: 'mais', before: 'estao ', after: ' consumindo'},
                    ]};
                    this._openPanel();
                    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
                        if (this._panel)
                            shootActor(this._panel, `${dir}/corretor.png`).then(() => {
                                this._closePanel();
                                this._result = null;
                                console.log(`${TAG} foto do corretor pronta`);
                            });
                        else
                            console.log(`${TAG} foto do corretor: painel fechou antes`);
                        return GLib.SOURCE_REMOVE;
                    });
                } catch (e) {
                    console.log(`${TAG} foto do corretor falhou: ${e.message}`);
                }
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    disable() {
        this._lancador?.disable();
        this._lancador = null;
        this._assistente?.disable();
        this._assistente = null;
        this._closePanel();
        this._ungrabKeys();
        this._clearTimers();
        this._im?.disconnectObject(this);
        global.display.disconnectObject(this);
        if (this._badge) {
            Main.layoutManager.removeChrome(this._badge);
            this._badge.destroy();
            this._badge = null;
        }
        this._vk = null;
        this._im = null;
    }

    _clearTimers() {
        for (const id of ['_checkId', '_watchId', '_hideId']) {
            if (this[id]) {
                GLib.source_remove(this[id]);
                this[id] = 0;
            }
        }
    }

    // Com o lançador aberto, o selo e os atalhos do campo de trás saem do
    // caminho (senão o Alt+O abriria outro painel por cima do lançador).
    setSuspended(on) {
        this._suspended = on;
        this._reset();
        if (on) {
            for (const k of [...(this._keyGrabs?.keys() ?? [])])
                this._setKey(k, false);
        } else {
            this._updateKeyGrabs();
        }
    }

    _reset() {
        if (this._writing)
            return;
        this._clearTimers();
        this._closePanel();
        this._badge?.hide();
        this._result = null;
        this._reqId++;
    }

    // ";gatilho" + espaço em qualquer campo vira o texto do snippet.
    _expandSnippet() {
        if (this._expanding || this._isSensitive())
            return false;
        const snippets = loadSnippets();
        if (!snippets.length)
            return false;
        let text, cursor;
        try {
            [text, cursor] = this._im.getSurroundingText();
        } catch (_e) {
            return false;
        }
        if (!text || cursor < 2)
            return false;
        const m = text.slice(0, cursor).match(/(?:^|\s)(;[^\s;]+) $/);
        const sn = m && snippets.find(x => x.gatilho === m[1]);
        if (!sn)
            return false;
        this._expanding = true;
        (async () => {
            try {
                const out = await expandSnippet(sn.texto);
                const n = m[1].length + 1;
                this._im.delete_surrounding(-n, n);
                if (out.includes('\n'))
                    await pasteViaClipboard(out); // várias linhas: cola
                else
                    this._im.commit(`${out} `);
                console.log(`${TAG} snippet ${sn.gatilho} expandido`);
            } catch (e) {
                console.log(`${TAG} snippet falhou: ${e.message}`);
            } finally {
                await sleep(300);
                this._expanding = false;
            }
        })();
        return true;
    }

    _appName() {
        const win = global.display.focus_window;
        return win ? (win.get_wm_class() ?? '?') : '(sem janela)';
    }

    _isSensitive() {
        const purpose = this._im._purpose ?? 0;
        const hints = this._im._hints ?? 0;
        return purpose === IBus.InputPurpose.PASSWORD ||
            purpose === IBus.InputPurpose.PIN ||
            (hints & IBus.InputHints.PRIVATE) !== 0;
    }

    _readField() {
        const [text, cursor] = this._im.getSurroundingText();
        return {text, cursor, rect: this._im._cursorRect ?? null};
    }

    _usable(field) {
        if (!confOn('ativo'))
            return false; // desligado no painel do Clippit
        const t = field.text?.trim() ?? '';
        const r = field.rect;
        return this._im.currentFocus && !this._isSensitive() &&
            t.length >= MIN_CHARS && /\s/.test(t) &&
            r && !(r.x === 0 && r.y === 0 && r.width === 0 && r.height === 0);
    }

    // ---- Ciclo: digitou → pausa → verifica → selo ------------------------

    _onSurroundingChanged() {
        if (this._suspended)
            return;
        if (this._writing)
            return; // estados intermediários da nossa própria escrita
        if (this._expandSnippet())
            return;
        const field = this._readField();

        if (this._panel) {
            if (this._result && flat(field.text) === flat(this._result.text))
                return; // nada mudou de verdade (só cursor/representação)
            this._closePanel();
        }

        // Entrou num campo de texto: já começa a carregar o modelo, para a
        // primeira correção não esperar (ele desliga sozinho se não usar).
        const focus = this._im.currentFocus;
        if (focus && focus !== this._warmFocus && confOn('ativo') && !this._isSensitive()) {
            this._warmFocus = focus;
            const now = GLib.get_monotonic_time();
            if (now - (this._lastWarm ?? 0) > 20_000_000) {
                this._lastWarm = now;
                learn(['--warm']);
            }
        }

        if (!this._usable(field)) {
            this._badge.hide();
            return;
        }

        // Correção inteira já conhecida (ex.: logo depois de aplicar).
        const whole = this._cache.get(flat(field.text));
        if (whole !== undefined) {
            this._setResult(field.text, whole);
            this._placeBadge(field.rect);
            return;
        }

        this._refreshFromChunks(field);
        this._placeBadge(field.rect);
        if (!confOn('verificar_sozinho'))
            return;

        // Trechos completos (longe do cursor) entram na fila na hora;
        // o trecho onde está o cursor espera uma pausa curta.
        const {lines, current} = this._split(field);
        for (const line of lines) {
            for (const ch of line.chunks) {
                if (ch !== current)
                    this._enqueue(ch);
            }
        }
        if (this._checkId)
            GLib.source_remove(this._checkId);
        this._checkId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, CHECK_DELAY_MS, () => {
            this._checkId = 0;
            if (current)
                this._enqueue(current);
            return GLib.SOURCE_REMOVE;
        });
    }

    _split(field) {
        const lines = chunksOf(field.text);
        let cursor = field.cursor ?? 0;
        let current = null;
        for (const line of lines) {
            for (const ch of line.chunks) {
                // O trecho em que o cursor está (ou logo antes dele) ainda pode mudar.
                if (cursor >= ch.s && cursor <= ch.e + 1)
                    current = ch;
            }
        }
        return {lines, current};
    }

    // Recalcula as sugestões com o que já está no cache de trechos.
    _refreshFromChunks(field) {
        const {lines} = this._split(field);
        const corrected = assemble(lines, this._chunkCache);
        const missing = lines.some(l => l.chunks.some(ch => !this._chunkCache.has(ch.key)));
        this._setResult(field.text, corrected, missing);
    }

    _enqueue(ch) {
        if (this._chunkCache.has(ch.key) || this._queue.some(q => q.key === ch.key) || this._inFlight === ch.key)
            return;
        this._queue.push({key: ch.key, ctx: ch.ctx, text: ch.text});
        this._work();
    }

    async _work() {
        if (this._working)
            return;
        this._working = true;
        this._setBusy(true);
        try {
            while (this._queue.length && this._im) {
                const chunk = this._queue.shift();
                // Pula trechos que já sumiram do campo (você apagou/editou).
                const field = this._readField();
                if (!field.text || !field.text.includes(chunk.text) || this._chunkCache.has(chunk.key))
                    continue;
                if (this._state !== 'issues')
                    this._setState('checking');
                this._inFlight = chunk.key;
                let out;
                try {
                    const raw = await this._runCorrector(chunk.ctx ? `${chunk.ctx} ${chunk.text}` : chunk.text);
                    out = takeChunkPart(chunk.ctx, chunk.text, raw);
                } catch (e) {
                    console.log(`${TAG} corretor falhou: ${e.message}`);
                    this._setState('error');
                    this._queue = [];
                    break;
                } finally {
                    this._inFlight = null;
                }
                this._chunkCache.set(chunk.key, out);
                while (this._chunkCache.size > CHUNK_CACHE_SIZE)
                    this._chunkCache.delete(this._chunkCache.keys().next().value);

                const now = this._readField();
                if (!this._im || this._writing || this._panel || !this._usable(now))
                    continue;
                this._refreshFromChunks(now);
            }
        } finally {
            this._working = false;
            this._setBusy(false);
        }
        if (this._openWhenDone && this._im && !this._panel) {
            this._openWhenDone = false;
            if (this._pending().length)
                this._openPanel();
        }
    }

    // Clique no selo antes de verificar: verifica tudo agora e abre o painel.
    _check(openAfter) {
        const field = this._readField();
        if (!this._usable(field))
            return;
        this._openWhenDone = openAfter;
        for (const line of chunksOf(field.text)) {
            for (const ch of line.chunks)
                this._enqueue(ch);
        }
        if (!this._working && openAfter && this._pending().length)
            this._openPanel();
    }

    _remember(key, corrected) {
        this._cache.set(key, corrected);
        while (this._cache.size > CACHE_SIZE)
            this._cache.delete(this._cache.keys().next().value);
    }

    _setResult(text, corrected, missing = false) {
        const sugs = suggestions(text, corrected);
        if (!sugs)
            console.log(`${TAG} não alinhei as linhas da correção, ignorando`);
        this._result = {text, corrected, sugs: sugs ?? []};
        this._setState(this._pending().length ? 'issues'
            : !missing ? 'ok' : (this._working || this._queue.length) ? 'checking' : 'idle');
    }

    _pending() {
        return (this._result?.sugs ?? []).filter(s => !this._ignored.has(s.key));
    }

    // ---- Selo -----------------------------------------------------------

    _setState(state) {
        this._state = state;
        const n = this._pending().length;
        this._badgeLabel.text = {
            idle: '✎  Verificar',
            checking: 'Verificando',
            ok: '✓  Sem erros',
            issues: `${n} ${n === 1 ? 'sugestão' : 'sugestões'}`,
            error: '⚠  Corretor indisponível',
        }[state];
        this._paintBadge();
    }

    _paintBadge() {
        if (!this._badge)
            return;
        const base = STYLE.badge[this._state] ?? STYLE.badge.idle;
        this._badgeBtn.style = this._badgeBtn.hover ? `${base} ${STYLE.badgeHover}` : base;
        // Aplicar tudo direto, sem abrir o painel.
        this._fixBtn.visible = this._state === 'issues' && !this._writing;
        this._setBusy(this._working);
        this._updateKeyGrabs();
    }

    // Carregando à esquerda enquanto houver trecho sendo verificado,
    // inclusive quando já existem sugestões na tela.
    _setBusy(busy) {
        if (!this._ring)
            return;
        if (busy && !this._busy) {
            this._busy = true;
            this._ringBox.opacity = 255;
            const spin = new Clutter.PropertyTransition({
                property_name: 'rotation-angle-z', duration: 800, repeat_count: -1,
            });
            spin.set_from(0);
            spin.set_to(360);
            this._ring.add_transition('spin', spin);
        } else if (!busy && this._busy) {
            this._busy = false;
            this._ring.remove_transition('spin');
            this._ringBox.opacity = 0;
        }
    }

    _placeBadge(rect) {
        if (this._suspended) {
            this._badge?.hide();
            return;
        }
        const monitor = this._monitorFor(rect);
        const [, w] = this._badge.get_preferred_width(-1);
        const [, h] = this._badge.get_preferred_height(-1);
        let x = rect.x + rect.width + 8;
        let y = rect.y + rect.height + 6;
        x = Math.min(Math.max(x, monitor.x + 4), monitor.x + monitor.width - w - 8);
        if (y + h > monitor.y + monitor.height - 4)
            y = rect.y - h - 6;
        this._badge.set_position(Math.round(x), Math.round(y));
        if (!this._panel)
            this._badge.show();
        this._shownFocus = this._im.currentFocus;
        this._startWatch();
    }

    _monitorFor(rect) {
        return Main.layoutManager.monitors.find(m =>
            rect.x >= m.x && rect.x < m.x + m.width &&
            rect.y >= m.y && rect.y < m.y + m.height) ?? Main.layoutManager.primaryMonitor;
    }

    _startWatch() {
        if (this._watchId)
            return;
        this._watchId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, WATCH_INTERVAL_MS, () => {
            if (!this._writing && this._im.currentFocus !== this._shownFocus) {
                this._watchId = 0;
                this._reset();
                return GLib.SOURCE_REMOVE;
            }
            if (this._badge.visible || this._panel || this._writing)
                return GLib.SOURCE_CONTINUE;
            this._watchId = 0;
            return GLib.SOURCE_REMOVE;
        });
    }

    _onBadgeClicked() {
        if (this._panel) {
            this._dismissPanel();
            return;
        }
        if (this._state === 'issues')
            this._openPanel();
        else if (this._state === 'idle' || this._state === 'error')
            this._check(true);
    }

    // ---- Painel ---------------------------------------------------------

    _openPanel() {
        this._closePanel();
        const pending = this._pending();
        if (!pending.length)
            return;

        const panel = new St.BoxLayout({vertical: true, reactive: true, style: STYLE.panel});
        addGlass(panel);
        panel.set_width(PANEL_WIDTH);

        const header = new St.BoxLayout({style: STYLE.header});
        header.add_child(new St.Label({text: 'Sugestões', style: STYLE.title, y_align: Clutter.ActorAlign.CENTER}));
        header.add_child(new St.Label({text: String(pending.length), style: STYLE.count, y_align: Clutter.ActorAlign.CENTER}));
        header.add_child(new St.Widget({x_expand: true}));
        const fixAll = new St.Button({label: 'Corrigir tudo', can_focus: false, track_hover: true, y_align: Clutter.ActorAlign.CENTER});
        fixAll.style = STYLE.fixAll;
        fixAll.connect('notify::hover', () => {
            fixAll.style = fixAll.hover ? STYLE.fixAllHover : STYLE.fixAll;
        });
        fixAll.connect('clicked', () => this._apply(this._pending()));
        header.add_child(fixAll);
        header.add_child(iconButton('window-close-symbolic', STYLE.close, STYLE.closeHover, () => this._dismissPanel()));
        panel.add_child(header);

        const list = new St.BoxLayout({vertical: true, style: STYLE.list});
        for (const sug of pending)
            list.add_child(this._item(sug));
        const scroll = new St.ScrollView({
            hscrollbar_policy: St.PolicyType.NEVER,
            vscrollbar_policy: St.PolicyType.AUTOMATIC,
            overlay_scrollbars: false,
        });
        if (scroll.set_child)
            scroll.set_child(list);
        else
            scroll.add_actor(list);
        panel.add_child(scroll);

        // Rodapé padrão (o mesmo do lançador).
        const foot = makeFooter();
        foot.set('Corretor', 'tools-check-spelling-symbolic', [{label: 'Corrigir tudo', key: 'Alt S'},
            {label: 'Fechar', key: 'Esc'}]);
        panel.add_child(foot.actor);

        this._panel = panel;
        this._updateKeyGrabs();
        Main.layoutManager.addTopChrome(panel);

        // Tamanho e posição: acima do cursor se couber, senão abaixo.
        const rect = this._readField().rect ?? {x: 0, y: 0, width: 0, height: 0};
        const monitor = this._monitorFor(rect);
        const spaceAbove = rect.y - monitor.y - 16;
        const spaceBelow = monitor.y + monitor.height - (rect.y + rect.height) - 16;
        const maxH = Math.min(460, Math.max(spaceAbove, spaceBelow));
        const [, natH] = panel.get_preferred_height(PANEL_WIDTH);
        if (natH > maxH) {
            const [, listH] = scroll.get_preferred_height(PANEL_WIDTH);
            scroll.set_height(Math.max(120, listH - (natH - maxH)));
        }
        const [, h] = panel.get_preferred_height(PANEL_WIDTH);
        const x = Math.min(Math.max(rect.x - 20, monitor.x + 12), monitor.x + monitor.width - PANEL_WIDTH - 12);
        const y = spaceAbove >= h ? rect.y - h - 10 : rect.y + rect.height + 10;
        panel.set_position(Math.round(x), Math.round(y));
        this._badge.hide();
        this._startWatch();
    }

    _item(sug) {
        const row = new St.BoxLayout({reactive: true, track_hover: true, style: STYLE.item});
        row.connect('notify::hover', () => {
            row.style = row.hover ? STYLE.itemHover : STYLE.item;
        });

        const textBox = new St.BoxLayout({vertical: true, x_expand: true, y_align: Clutter.ActorAlign.CENTER});
        textBox.add_child(new St.Label({text: kindOf(sug), style: STYLE.kind}));

        const ctx = new St.Label({style: STYLE.context, x_expand: true});
        const parts = [];
        if (sug.before)
            parts.push(`${esc(sug.before)} `);
        if (sug.from)
            parts.push(`<span foreground="${C.danger}" strikethrough="true">${esc(sug.from)}</span>`);
        if (sug.from && sug.to)
            parts.push(' ');
        if (sug.to)
            parts.push(`<span foreground="${C.ok}" weight="bold">${esc(sug.to)}</span>`);
        if (sug.after)
            parts.push(` ${esc(sug.after)}`);
        ctx.clutter_text.set_markup(parts.join(''));
        ctx.clutter_text.line_wrap = true;
        ctx.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        ctx.clutter_text.ellipsize = Pango.EllipsizeMode.NONE;
        textBox.add_child(ctx);
        row.add_child(textBox);

        row.add_child(iconButton('object-select-symbolic', STYLE.iconBtn, STYLE.iconBtnOk, () => this._apply([sug])));
        row.add_child(iconButton('window-close-symbolic', STYLE.iconBtn, STYLE.iconBtnNo, () => this._ignore(sug)));
        return row;
    }

    // Teclas reservadas só enquanto fazem sentido; no resto do tempo voltam
    // para o app:
    //  - Esc: com o painel aberto;
    //  - Alt+O: abre/fecha o painel quando o selo está na tela;
    //  - Alt+S: com o painel aberto ou com o selo mostrando sugestões, em qualquer app;
    //  - Ctrl+S: igual, menos em apps onde Ctrl+S é salvar arquivo (editores etc.).
    _updateKeyGrabs() {
        if (!this._im || this._suspended)
            return;
        const panel = Boolean(this._panel);
        const issues = Boolean(this._badge?.visible) && this._state === 'issues';
        const applyAll = () => this._apply(this._pending());
        this._setKey('Escape', panel, () => this._dismissPanel());
        // Alt+O: abre o painel (ou verifica e abre); com ele aberto, fecha.
        const badgeUp = Boolean(this._badge?.visible) && ['issues', 'idle', 'error'].includes(this._state);
        // Atalho trocado no painel: solta o antigo antes de pegar o novo.
        const open = keyOf('sugestoes'), all = keyOf('aplicar');
        for (const k of [...(this._keyGrabs?.keys() ?? [])]) {
            if (![open, all, 'Escape', '<Control>s'].includes(k))
                this._setKey(k, false);
        }
        if (open !== 'desligado')
            this._setKey(open, panel || badgeUp, () => this._onBadgeClicked());
        if (all !== 'desligado')
            this._setKey(all, panel || issues, applyAll);
        this._setKey('<Control>s', panel || (issues && !SAVE_APPS.test(this._appName())), applyAll);
    }

    _setKey(accel, want, fn) {
        this._keyGrabs ??= new Map();
        const grab = this._keyGrabs.get(accel);
        if (want && !grab) {
            const action = global.display.grab_accelerator(accel, Meta.KeyBindingFlags.NONE);
            if (action === Meta.KeyBindingAction.NONE) {
                console.log(`${TAG} não consegui reservar a tecla ${accel}`);
                return;
            }
            const name = Meta.external_binding_name_for_action(action);
            Main.wm.allowKeybinding(name, Shell.ActionMode.ALL);
            this._keyGrabs.set(accel, {action, name, fn});
        } else if (!want && grab) {
            global.display.ungrab_accelerator(grab.action);
            Main.wm.allowKeybinding(grab.name, Shell.ActionMode.NONE);
            this._keyGrabs.delete(accel);
        }
        if (!this._accelId && this._keyGrabs.size) {
            this._accelId = global.display.connect('accelerator-activated', (_d, action) => {
                for (const g of this._keyGrabs.values()) {
                    if (g.action === action)
                        g.fn();
                }
            });
        }
    }

    _ungrabKeys() {
        for (const accel of [...(this._keyGrabs?.keys() ?? [])])
            this._setKey(accel, false);
        if (this._accelId) {
            global.display.disconnect(this._accelId);
            this._accelId = 0;
        }
    }

    // Fechado por você (✕, Esc, Alt+O, clique no selo): o selo volta para a
    // tela, para dar para abrir de novo.
    _dismissPanel() {
        this._closePanel();
        const field = this._readField();
        if (this._usable(field)) {
            this._setState(this._pending().length ? 'issues' : this._state);
            this._placeBadge(field.rect);
        }
    }

    _closePanel() {
        if (this._panel) {
            Main.layoutManager.removeChrome(this._panel);
            this._panel.destroy();
            this._panel = null;
        }
        this._updateKeyGrabs();
    }

    // ---- Ações ----------------------------------------------------------

    _ignore(sug) {
        this._ignored.add(sug.key);
        // Ignorar uma palavra = ela está certa para você (nome, gíria): vai para o dicionário.
        if (sug.from && !sug.from.includes(' '))
            learn(['--learn-word', sug.from]);
        this._afterChange();
    }

    async _apply(sugs) {
        if (!sugs.length || this._writing)
            return;
        // Trocas aceitas viram aprendizado (vc → você): depois de algumas vezes, saem sozinhas.
        for (const sg of sugs) {
            if (sg.from && sg.to && !sg.from.includes(' ') && !sg.to.includes(' '))
                learn(['--learn-fix', sg.from, sg.to]);
        }
        const field = this._readField();
        if (!this._result || flat(field.text) !== flat(this._result.text)) {
            console.log(`${TAG} texto mudou, ação cancelada`);
            this._closePanel();
            return;
        }
        const text = field.text;
        const cps = [...text];
        let cursor = field.cursor;
        const bytes = utf8Length(text);
        if (bytes !== cps.length && cursor > cps.length)
            cursor = byteToCharOffset(text, cursor);

        const newText = applyEdits(cps, sugs);
        // Aplicou tudo o que faltava: vira caso da prova do corretor (antes → depois).
        if (sugs.length === this._pending().length)
            learn(['--learn-case'], JSON.stringify({antes: text, depois: newText}));
        const lo = Math.min(cursor, ...sugs.map(s => s.s));
        const hi = Math.max(cursor, ...sugs.map(s => s.e));
        const sameLine = !cps.slice(lo, hi).includes('\n');

        this._writing = true;
        try {
            if (sameLine) {
                // Troca só o trecho [lo, hi), que encosta no cursor.
                const span = applyEdits(cps.slice(lo, hi), sugs.map(s => ({...s, s: s.s - lo, e: s.e - lo})));
                this._im.delete_surrounding(lo - cursor, hi - lo);
                this._im.commit(span);
                console.log(`${TAG} corrigido na linha do cursor app=${this._appName()} edições=${sugs.length}`);
            } else if (cps.length <= MAX_PASTE_CHARS) {
                await this._pasteAll(newText);
                console.log(`${TAG} corrigido por colagem app=${this._appName()} edições=${sugs.length} ` +
                    `linhas=${text.split('\n').length}`);
            } else {
                console.log(`${TAG} texto longo demais para colar (${cps.length} chars)`);
                return;
            }
        } finally {
            await sleep(250);
            this._writing = false;
        }

        // O corrigido continua valendo para o texto novo.
        this._remember(flat(newText), this._result.corrected);
        const now = this._readField();
        this._setResult(now.text ?? newText, this._result.corrected);
        this._afterChange();
    }

    // Seleciona tudo no campo e cola o texto novo; devolve o clipboard antigo.
    async _pasteAll(newText) {
        const cb = St.Clipboard.get_default();
        const old = await new Promise(r => cb.get_text(St.ClipboardType.CLIPBOARD, (_c, t) => r(t)));
        cb.set_text(St.ClipboardType.CLIPBOARD, newText);
        this._vk ??= Clutter.get_default_backend().get_default_seat()
            .create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
        const key = (kv, down) => this._vk.notify_keyval(GLib.get_monotonic_time(), kv,
            down ? Clutter.KeyState.PRESSED : Clutter.KeyState.RELEASED);
        await sleep(30);
        key(Clutter.KEY_Control_L, true);
        key(Clutter.KEY_a, true);
        key(Clutter.KEY_a, false);
        await sleep(40);
        key(Clutter.KEY_v, true);
        key(Clutter.KEY_v, false);
        key(Clutter.KEY_Control_L, false);
        await sleep(500);
        if (old !== null)
            cb.set_text(St.ClipboardType.CLIPBOARD, old);
    }

    _afterChange() {
        const n = this._pending().length;
        this._setState(n ? 'issues' : 'ok');
        if (n) {
            this._openPanel();
            return;
        }
        this._closePanel();
        const field = this._readField();
        if (field.rect && this._usable(field))
            this._placeBadge(field.rect);
    }

    _runCorrector(text) {
        return new Promise((resolve, reject) => {
            let proc;
            try {
                proc = Gio.Subprocess.new([CORRECTOR_CMD],
                    Gio.SubprocessFlags.STDIN_PIPE |
                    Gio.SubprocessFlags.STDOUT_PIPE |
                    Gio.SubprocessFlags.STDERR_PIPE);
            } catch (e) {
                reject(e);
                return;
            }
            const timeoutId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, CORRECT_TIMEOUT_S, () => {
                proc.force_exit();
                return GLib.SOURCE_REMOVE;
            });
            proc.communicate_utf8_async(text, null, (p, res) => {
                GLib.source_remove(timeoutId);
                try {
                    const [, stdout, stderr] = p.communicate_utf8_finish(res);
                    if (!p.get_successful())
                        throw new Error(`saiu com erro: ${(stderr ?? '').trim().slice(0, 200)}`);
                    resolve((stdout ?? '').replace(/\n+$/, ''));
                } catch (e) {
                    reject(e);
                }
            });
        });
    }
}
