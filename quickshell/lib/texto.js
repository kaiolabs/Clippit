.pragma library
.import "compat.js" as Compat

// Spell-check text logic, the same as the GNOME extension (core.js): words and
// lines with positions, suggestions from the model's correction, chunks that
// are checked one at a time and cached. Positions are in code points.

const CHUNK_WORDS = 16;
const CONTEXT_WORDS = 6;

const last = Compat.last;

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
        out.push({t: cps.slice(s, i).join(""), s, e: i});
    }
    return out;
}

function linesWithPos(cps) {
    const lines = [];
    let s = 0;
    for (let i = 0; i <= cps.length; i++) {
        if (i === cps.length || cps[i] === "\n") {
            lines.push({s, e: i});
            s = i + 1;
        }
    }
    return lines;
}

function lcsOps(a, b) {
    const n = a.length, m = b.length;
    const dp = [];
    for (let i = 0; i <= n; i++) {
        dp.push(new Array(m + 1).fill(0));
    }
    for (let i = n - 1; i >= 0; i--) {
        for (let j = m - 1; j >= 0; j--)
            dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
    const ops = [];
    let i = 0, j = 0;
    while (i < n || j < m) {
        if (i < n && j < m && a[i] === b[j]) {
            ops.push({op: "same", i: i++, j: j++});
        } else if (j < m && (i >= n || dp[i][j + 1] >= dp[i + 1][j])) {
            ops.push({op: "ins", j: j++, i});
        } else {
            ops.push({op: "del", i: i++});
        }
    }
    return ops;
}

function suggestions(original, corrected) {
    const cps = Compat.chars(original);
    const oLines = linesWithPos(cps);
    let cLines = corrected.split("\n");
    if (cLines.length !== oLines.length) {
        // The model changed the line breaks: align only the lines with text.
        const nonEmpty = oLines.filter(l => cps.slice(l.s, l.e).join("").trim());
        const cNon = cLines.filter(l => l.trim());
        if (nonEmpty.length !== cNon.length)
            return null;
        const map = new Map(nonEmpty.map((l, k) => [l, cNon[k]]));
        cLines = oLines.map(l => map.has(l) ? map.get(l) : cps.slice(l.s, l.e).join(""));
    }

    const out = [];
    oLines.forEach((line, li) => {
        const a = wordsWithPos(cps, line.s, line.e);
        const b = cLines[li].split(/\s+/).filter(Boolean);
        const ops = lcsOps(a.map(w => w.t), b);
        let k = 0;
        while (k < ops.length) {
            if (ops[k].op === "same") {
                k++;
                continue;
            }
            const from = [], to = [];
            const firstI = ops[k].i;
            while (k < ops.length && ops[k].op !== "same") {
                if (ops[k].op === "del")
                    from.push(ops[k].i);
                else
                    to.push(b[ops[k].j]);
                k++;
            }
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
    const from = fromWords.map(w => w.t).join(" ");
    const rep = to.join(" ");
    let s, e, r;
    if (fromWords.length) {
        s = fromWords[0].s;
        e = last(fromWords).e;
        r = rep;
        if (!rep) {
            const prev = words[fromIdx[0] - 1];
            const next = words[last(fromIdx) + 1];
            if (prev)
                s = prev.e;
            else if (next)
                e = next.s;
        }
    } else {
        const prev = words[anchorIdx - 1];
        s = e = prev ? prev.e : line.s;
        r = prev ? " " + rep : rep + " ";
    }
    const first = fromIdx.length ? fromIdx[0] : anchorIdx;
    const lastIdx = fromIdx.length ? last(fromIdx) : anchorIdx - 1;
    const before = words.slice(Math.max(0, first - 3), first).map(w => w.t).join(" ");
    const after = words.slice(lastIdx + 1, lastIdx + 4).map(w => w.t).join(" ");
    return {
        s, e, rep: r, from, to: rep,
        key: from + "→" + rep,
        before: (first > 3 ? "…" : "") + before,
        after: after + (lastIdx + 4 < words.length ? "…" : ""),
    };
}

function applyEdits(cps, edits) {
    const out = cps.slice();
    edits.slice().sort((x, y) => y.s - x.s).forEach(ed => {
        const args = [ed.s, ed.e - ed.s].concat(Compat.chars(ed.rep));
        Array.prototype.splice.apply(out, args);
    });
    return out.join("");
}

function flat(text) {
    return (text ?? "").split(/\s+/).filter(Boolean).join(" ");
}

function stripAccents(s) {
    return s.normalize("NFD").replace(/[̀-ͯ]/g, "");
}

function stripPunct(s) {
    return s.replace(/[.,;:!?…"'“”()\-]/g, "");
}

function kindOf(sug) {
    const a = sug.from, b = sug.to;
    if (!a)
        return "ADIÇÃO";
    if (!b)
        return "REMOÇÃO";
    const pa = stripPunct(a), pb = stripPunct(b);
    if (pa === pb)
        return "PONTUAÇÃO";
    if (pa.toLowerCase() === pb.toLowerCase())
        return "MAIÚSCULA";
    if (stripAccents(pa).toLowerCase() === stripAccents(pb).toLowerCase())
        return "ACENTUAÇÃO";
    return "ORTOGRAFIA";
}

function chunksOf(text) {
    const cps = Compat.chars(text);
    return linesWithPos(cps).map(line => {
        const words = wordsWithPos(cps, line.s, line.e);
        const chunks = [];
        let group = [];
        const close = () => {
            if (group.length) {
                const s = group[0].s, e = last(group).e;
                const t = cps.slice(s, e).join("");
                const prev = last(chunks);
                const ctx = prev ? prev.words.slice(-CONTEXT_WORDS).join(" ") : "";
                chunks.push({s, e, text: t, ctx, key: ctx ? ctx + "\u0001" + t : t, words: group.map(w => w.t)});
            }
            group = [];
        };
        for (const w of words) {
            group.push(w);
            if (group.length >= CHUNK_WORDS || (group.length >= 4 && /[.!?…]$/.test(w.t)))
                close();
        }
        close();
        return {s: line.s, e: line.e, chunks};
    });
}

function takeChunkPart(ctx, text, out) {
    if (!ctx)
        return out;
    const inWords = (ctx + " " + text).split(/\s+/).filter(Boolean);
    const outWords = out.split(/\s+/).filter(Boolean);
    const b = ctx.split(/\s+/).filter(Boolean).length;
    const kept = [];
    for (const op of lcsOps(inWords, outWords)) {
        if (op.op === "del")
            continue;
        if (op.i >= b)
            kept.push(outWords[op.j]);
    }
    return kept.length ? kept.join(" ") : text;
}

// cache: a Map chunk key → corrected chunk.
function assemble(lines, cache) {
    return lines.map(line => {
        const parts = [];
        line.chunks.forEach((ch, i) => {
            const checked = cache.has(ch.key);
            let out = checked ? cache.get(ch.key) : ch.text;
            if (checked && i > 0 && !/[.!?…]$/.test(parts[i - 1]) && /^[a-zà-ÿ]/.test(ch.text))
                out = out.charAt(0).toLowerCase() + out.slice(1);
            const next = line.chunks[i + 1];
            if (checked && next && !/[.!?…,;:]$/.test(ch.text) && /^[a-zà-ÿ]/.test(next.text))
                out = out.replace(/\s*(\.\.\.|…|[.?!])$/, "");
            parts.push(out);
        });
        return parts.join(" ");
    }).join("\n");
}
