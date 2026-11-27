#!/usr/bin/env node
// Testa o trecho de atalhos da extensão (entre "@shortcuts:begin" e "@shortcuts:end" no core.js):
// a leitura de `atalho_<nome>` do corretor.conf, a comparação exata de modificadores e os textos.
// Também confere que cada atalho declarado é usado de verdade por algum tratador de teclas.
//
//   node scripts/test-shortcuts.mjs

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = readFileSync(join(root, 'gnome-extension/clippit-corretor@clippit/core.js'), 'utf8');
const block = source.split('// @shortcuts:begin')[1].split('\n').slice(1).join('\n').split('// @shortcuts:end')[0];

// Máscaras e teclas no formato do Clutter (só o que o trecho usa).
const MASK = { shift: 1, ctrl: 4, alt: 8, mod4: 64, super: 67108864 };
const KEYS = {
    p: 'p', P: 'P', n: 'n', e: 'e', Tab: 'Tab', ISO_Left_Tab: 'ISO_Left_Tab', Return: 'Return', KP_Enter: 'KP_Enter',
    Escape: 'Escape', BackSpace: 'BackSpace', Delete: 'Delete', space: 'space', Right: 'Right', semicolon: 'semicolon',
};
let conf = {};
const Clutter = {
    ModifierType: { CONTROL_MASK: MASK.ctrl, MOD1_MASK: MASK.alt, SHIFT_MASK: MASK.shift, SUPER_MASK: MASK.super, MOD4_MASK: MASK.mod4 },
    keyval_name: sym => KEYS[sym],
};
const api = new Function(
    'confValue', 'Clutter',
    `${block}\nreturn {keyIs, modIs, hasMods, keyLabel, keyOf, parseAccel, DEFAULT_KEYS, INNER_SHORTCUTS, SHORTCUTS};`,
)((key, fallback) => conf[key] ?? fallback, Clutter);
const { keyIs, modIs, hasMods, keyLabel, keyOf, parseAccel, DEFAULT_KEYS, INNER_SHORTCUTS, SHORTCUTS } = api;

const ev = (sym, state = 0) => ({ get_key_symbol: () => sym, get_state: () => state });
const reset = () => { conf = {}; };

// padrões
reset();
assert(keyIs(ev('p', MASK.ctrl), 'fixar'));
assert(keyIs(ev('P', MASK.ctrl), 'fixar'), 'letra maiúscula (Caps Lock) também vale');
assert(!keyIs(ev('p'), 'fixar'), 'sem Ctrl não é o atalho');
assert(keyIs(ev('Escape'), 'fechar'));
assert(keyIs(ev('Delete', MASK.ctrl), 'apagar'));
assert(keyIs(ev('space', MASK.ctrl), 'ampliar'));
assert(keyIs(ev('BackSpace'), 'voltar') && !hasMods('voltar'));

// modificadores exatos: um a mais não dispara o atalho
assert(!keyIs(ev('p', MASK.ctrl | MASK.alt), 'fixar'));
assert(!keyIs(ev('n', MASK.ctrl | MASK.shift), 'nova_conversa'));
assert(!keyIs(ev('Tab', MASK.ctrl), 'transformar'));

// Shift+Tab chega como ISO_Left_Tab e continua sendo Tab
assert(keyIs(ev('Tab'), 'transformar'));
assert(keyIs(ev('ISO_Left_Tab', MASK.shift), 'transformar'));
assert(keyIs(ev('Tab', MASK.shift), 'categoria', true), 'categoria ignora o Shift (ele só inverte)');
assert(!keyIs(ev('Tab', MASK.shift), 'categoria'));

// valor do corretor.conf é o que vale, e "desligado" tira o atalho
conf = { atalho_fixar: '<Alt>x', atalho_apagar: 'desligado', atalho_fechar: 'F1' };
assert.equal(keyOf('fixar'), '<Alt>x');
assert(!keyIs(ev('p', MASK.ctrl), 'fixar'), 'o padrão deixa de valer');
assert(!keyIs(ev('Delete', MASK.ctrl), 'apagar'));
assert(!keyIs(ev('Escape'), 'fechar'));
assert.equal(keyLabel('apagar'), '—');
conf = { atalho_nova_conversa: '<Control><Shift>n' };
assert(keyIs(ev('n', MASK.ctrl | MASK.shift), 'nova_conversa'));
assert(!keyIs(ev('n', MASK.ctrl), 'nova_conversa'));

// modificador que a extensão não conhece: não vira "tecla sozinha"
conf = { atalho_fixar: '<Hyper>p', atalho_apagar: '<Mod2>Delete' };
assert.equal(parseAccel('<Hyper>p'), null);
assert(!keyIs(ev('p'), 'fixar'), 'digitar p não pode disparar <Hyper>p');
assert(!keyIs(ev('Delete'), 'apagar'));

// "copiar" só vale com Enter; outra tecla volta para o padrão
reset();
assert(modIs(ev('Return', MASK.ctrl), 'copiar'));
assert(!modIs(ev('Return'), 'copiar'));
assert(!modIs(ev('Return', MASK.ctrl | MASK.alt), 'copiar'));
conf = { atalho_copiar: '<Alt>Return' };
assert(modIs(ev('Return', MASK.alt), 'copiar'));
conf = { atalho_copiar: '<Control>k' };
assert.equal(keyOf('copiar'), DEFAULT_KEYS.copiar);
assert.equal(keyLabel('copiar'), 'Ctrl ↵');
conf = { atalho_copiar: 'desligado' };
assert(!modIs(ev('Return', MASK.ctrl), 'copiar'));

// textos na tela
reset();
assert.equal(keyLabel('ampliar'), 'Ctrl Espaço');
assert.equal(keyLabel('modo_jogo', '+'), 'Ctrl+Shift+E');
assert.equal(keyLabel('snippets', '+'), 'Alt+;');
assert.equal(keyLabel('voltar'), '⌫');
conf = { atalho_lado: '<Super>x' };
assert.equal(keyLabel('lado', '+'), 'Super+X');

// cada atalho declarado é lido do conf e usado por algum tratador
assert(source.includes('`atalho_${id}`'), 'keyOf tem que ler atalho_<nome> do corretor.conf');
for (const id of Object.keys(INNER_SHORTCUTS)) {
    const used = new RegExp(`(keyIs|modIs|keyLabel|hasMods)\\([^)]*'${id}'`).test(source);
    assert(used, `atalho "${id}" está declarado mas nenhum tratador o usa`);
}
for (const id of Object.keys(SHORTCUTS)) {
    const used = new RegExp(`\\['${id}',`).test(source) || new RegExp(`(keyOf|keyIs|keyLabel)\\('${id}'`).test(source);
    assert(used, `atalho global "${id}" está declarado mas nada o reserva`);
}

console.log('atalhos: ok');
