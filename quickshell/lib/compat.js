.pragma library

// The Quickshell JS engine (QV4) lacks a few things the GNOME extension used:
// async/await, Array.at, String.replaceAll, matchAll, flatMap.

// Runs a generator as an async function: `yield promise` works like `await`.
function co(gen, self) {
    return new Promise((resolve, reject) => {
        const it = gen.call(self);
        function step(value, failed) {
            let r;
            try {
                r = failed ? it.throw(value) : it.next(value);
            } catch (e) {
                reject(e);
                return;
            }
            if (r.done) {
                resolve(r.value);
                return;
            }
            Promise.resolve(r.value).then(v => step(v, false), e => step(e, true));
        }
        step(undefined, false);
    });
}

function last(list) {
    return list.length ? list[list.length - 1] : undefined;
}

function replaceAll(text, from, to) {
    return String(text).split(from).join(to);
}

function matchAll(re, text) {
    const flags = re.flags.indexOf("g") >= 0 ? re.flags : re.flags + "g";
    const g = new RegExp(re.source, flags);
    const out = [];
    let m;
    while ((m = g.exec(text)) !== null) {
        out.push(m);
        if (m[0] === "")
            g.lastIndex++;
    }
    return out;
}

function flatMap(list, fn) {
    const out = [];
    list.forEach((x, i) => {
        const r = fn(x, i);
        if (Array.isArray(r))
            r.forEach(y => out.push(y));
        else
            out.push(r);
    });
    return out;
}

function utf8Length(str) {
    return unescape(encodeURIComponent(str)).length;
}

// Code points (Array.from keeps surrogate pairs together).
function chars(str) {
    return Array.from(str ?? "");
}

function sleep(ms) {
    return new Promise(resolve => {
        const t = Qt.createQmlObject("import QtQuick; Timer {}", Qt.application, "sleep");
        t.interval = ms;
        t.triggered.connect(() => {
            t.destroy();
            resolve();
        });
        t.start();
    });
}

// Letters with accents, for the "Iniciais Maiúsculas" transform (no \p{L} here).
const LETTER = "A-Za-zÀ-ÖØ-öø-ÿĀ-ſ";
