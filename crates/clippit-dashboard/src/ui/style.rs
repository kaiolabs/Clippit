//! Visual do painel no mesmo padrão do popup novo do Clippit:
//! mesmos tokens do lançador (vidro escuro, cantos 4–8 px, Inter, azul #4c9dff só em ação).

use gtk::gdk;

const CSS: &str = r#"
/* Tokens (iguais aos T/C do core.js da extensão). */
@define-color clip_bg #1c1c20;
@define-color clip_raised #26262b;
@define-color clip_text #f2f2f2;
@define-color clip_muted #9a9aa0;
@define-color clip_faint #6e6e74;
@define-color clip_accent #4c9dff;
@define-color clip_accent_hover #6aafff;
@define-color clip_danger #ff6b6b;

@define-color window_bg_color @clip_bg;
@define-color window_fg_color @clip_text;
@define-color view_bg_color @clip_bg;
@define-color view_fg_color @clip_text;
@define-color headerbar_bg_color @clip_bg;
@define-color headerbar_fg_color @clip_text;
@define-color headerbar_border_color transparent;
@define-color headerbar_shade_color transparent;
@define-color card_bg_color alpha(white, 0.04);
@define-color card_fg_color @clip_text;
@define-color popover_bg_color @clip_raised;
@define-color popover_fg_color @clip_text;
@define-color dialog_bg_color @clip_raised;
@define-color dialog_fg_color @clip_text;
@define-color accent_bg_color @clip_accent;
@define-color accent_color @clip_accent;
@define-color accent_fg_color #ffffff;
@define-color destructive_bg_color @clip_danger;
@define-color destructive_color @clip_danger;

window.clippit-dashboard {
    background-color: @clip_bg; background-image: none;
    color: @clip_text;
    font-family: Inter, Cantarell, sans-serif;
    font-size: 10pt;
}

/* Barra de título: sem linha embaixo. */
window.clippit-dashboard headerbar {
    background-color: @clip_bg; background-image: none;
    box-shadow: none;
    border: none;
    min-height: 44px;
}
window.clippit-dashboard headerbar .title { font-weight: 600; font-size: 10pt; }
window.clippit-dashboard headerbar .subtitle { color: @clip_faint; font-size: 8pt; }
window.clippit-dashboard headerbar windowcontrols button {
    min-width: 22px; min-height: 22px; padding: 0; margin: 0 2px;
    background-color: alpha(white, 0.06); background-image: none; color: @clip_muted; border-radius: 6px;
}
window.clippit-dashboard headerbar windowcontrols button:hover { background-color: alpha(white, 0.12); background-image: none; color: @clip_text; }

/* Menu lateral: seleção discreta, como as linhas do lançador. */
list.clip-sidebar {
    background-color: transparent; background-image: none;
    padding: 6px 10px;
}
list.clip-sidebar > row {
    border-radius: 8px;
    margin-bottom: 2px;
    color: @clip_muted;
    outline: none;
    min-height: 0;
}
list.clip-sidebar > row image { color: @clip_faint; -gtk-icon-size: 16px; }
list.clip-sidebar > row:hover { background-color: alpha(white, 0.04); background-image: none; }
list.clip-sidebar > row:selected {
    background-color: alpha(white, 0.08); background-image: none;
    color: @clip_text;
    font-weight: 600;
}
list.clip-sidebar > row:selected image { color: @clip_text; }

/* Sem divisória entre o menu e o conteúdo. */
window.clippit-dashboard paned > separator {
    background: none;
    min-width: 1px;
    border: none;
    box-shadow: none;
}

/* Páginas: título da seção pequeno e apagado, cartão translúcido. */
window.clippit-dashboard preferencespage > scrolledwindow > viewport > clamp > box {
    margin: 8px 0 24px 0;
}
window.clippit-dashboard preferencesgroup > box > box.header {
    margin-bottom: 6px;
}
window.clippit-dashboard preferencesgroup > box > box.header .title,
window.clippit-dashboard preferencesgroup .heading {
    color: @clip_faint;
    font-size: 8pt;
    font-weight: 600;
}
window.clippit-dashboard preferencesgroup > box > box.header .description,
window.clippit-dashboard .dim-label {
    color: @clip_muted;
    font-size: 9pt;
}
window.clippit-dashboard list.boxed-list {
    background-color: alpha(white, 0.04); background-image: none;
    border: none;
    border-radius: 8px;
    box-shadow: none;
}
window.clippit-dashboard list.boxed-list > row,
window.clippit-dashboard list.boxed-list > row:not(:last-child) {
    background-color: transparent; background-image: none;
    border: none;
    box-shadow: none;
    border-radius: 8px;
    min-height: 48px;
}
window.clippit-dashboard list.boxed-list > row.activatable:hover { background-color: alpha(white, 0.04); background-image: none; }
window.clippit-dashboard list > separator { background: none; min-height: 0; }
window.clippit-dashboard row .title { color: @clip_text; font-size: 10pt; font-weight: 500; }
window.clippit-dashboard row .subtitle { color: @clip_muted; font-size: 9pt; }
window.clippit-dashboard row .prefixes image,
window.clippit-dashboard row > box > image { color: @clip_muted; }

/* Controles */
window.clippit-dashboard switch {
    background-color: alpha(white, 0.12); background-image: none;
    border: none;
    box-shadow: none;
}
window.clippit-dashboard switch:checked { background-color: @clip_accent; background-image: none; }
window.clippit-dashboard switch > slider { background-color: #ffffff; background-image: none; box-shadow: none; border: none; }

window.clippit-dashboard spinbutton,
window.clippit-dashboard entry,
window.clippit-dashboard dropdown > button,
window.clippit-dashboard row.combo .combo {
    background-color: alpha(white, 0.06); background-image: none;
    border: 1px solid transparent;
    border-radius: 6px;
    color: @clip_text;
    box-shadow: none;
    outline: none;
}
window.clippit-dashboard entry:focus-within,
window.clippit-dashboard spinbutton:focus-within {
    border-color: alpha(@clip_accent, 0.7);
    box-shadow: none;
    outline: none;
}
window.clippit-dashboard spinbutton > button {
    background: none;
    border: none;
    box-shadow: none;
    color: @clip_muted;
    min-width: 26px;
}
window.clippit-dashboard spinbutton > button:hover { color: @clip_text; background-color: alpha(white, 0.06); background-image: none; }

window.clippit-dashboard scale trough { background-color: alpha(white, 0.10); background-image: none; min-height: 4px; border-radius: 4px; }
window.clippit-dashboard scale highlight { background-color: @clip_accent; background-image: none; border-radius: 4px; }
window.clippit-dashboard scale slider {
    background-color: #ffffff; background-image: none; min-width: 14px; min-height: 14px; margin: -6px;
    border: none; box-shadow: 0 1px 3px alpha(black, 0.4);
}

/* Botões: os mesmos quatro do lançador. */
window.clippit-dashboard button { border-radius: 6px; font-size: 9pt; }
window.clippit-dashboard button.clip-btn {
    background-color: alpha(white, 0.08); background-image: none;
    color: @clip_text;
    border: none;
    padding: 4px 12px;
    box-shadow: none;
}
window.clippit-dashboard button.clip-btn:hover { background-color: alpha(white, 0.14); background-image: none; }
window.clippit-dashboard button.clip-primary {
    background-color: @clip_accent; background-image: none;
    color: #ffffff;
    border: none;
    font-weight: 600;
    padding: 4px 12px;
    box-shadow: none;
}
window.clippit-dashboard button.clip-primary:hover { background-color: @clip_accent_hover; background-image: none; }
window.clippit-dashboard button.clip-danger,
window.clippit-dashboard button.destructive-action {
    background-color: alpha(@clip_danger, 0.12); background-image: none;
    color: @clip_danger;
    border: none;
    font-weight: 600;
    padding: 4px 12px;
    box-shadow: none;
}
window.clippit-dashboard button.clip-danger:hover,
window.clippit-dashboard button.destructive-action:hover { background-color: alpha(@clip_danger, 0.20); background-image: none; }
window.clippit-dashboard button.clip-icon {
    min-width: 28px;
    min-height: 28px;
    padding: 0;
    background: alpha(white, 0.06);
    color: @clip_muted;
    border: none;
    box-shadow: none;
}
window.clippit-dashboard button.clip-icon:hover {
    background-color: alpha(white, 0.12); background-image: none;
    color: @clip_text;
}

/* Tecla: igual ao keycap do lançador. */
label.clip-key {
    font-size: 8pt;
    font-weight: 600;
    color: @clip_muted;
    padding: 1px 6px;
    border-radius: 4px;
    background-color: alpha(white, 0.08); background-image: none;
    border: none;
}

/* Status (ligado/desligado) */
label.clip-pill {
    font-size: 8pt;
    font-weight: 600;
    padding: 1px 8px;
    border-radius: 4px;
}
label.clip-pill.on { color: #4ade80; background-color: alpha(#4ade80, 0.12); background-image: none; }
label.clip-pill.off { color: @clip_muted; background-color: alpha(white, 0.06); background-image: none; }

popover > contents {
    background-color: @clip_raised; background-image: none;
    border: 1px solid alpha(white, 0.10);
    border-radius: 8px;
    box-shadow: 0 12px 32px alpha(black, 0.5);
}
popover modelbutton, popover row { border-radius: 6px; }

/* Caixa de confirmação (Apagar…) */
window.messagedialog, dialog-host > dialog.alert sheet {
    background-color: @clip_raised; background-image: none;
    border-radius: 12px;
}
window.messagedialog .response-area > button {
    background-color: alpha(white, 0.06); background-image: none;
    border: none;
    border-radius: 6px;
    margin: 6px;
}
window.messagedialog .response-area > button.destructive { color: @clip_danger; background-color: alpha(@clip_danger, 0.12); background-image: none; }
window.messagedialog .response-area { border: none; }
window.messagedialog separator { background: none; min-height: 0; min-width: 0; }
/* Funcionalidades: busca, passos numerados e linhas que abrem */
window.clippit-dashboard entry.clip-search, window.clippit-dashboard searchbar entry, window.clippit-dashboard .clip-search {
    min-height: 36px; font-size: 10pt; background-color: alpha(white, 0.06); background-image: none;
    border-radius: 8px; border: 1px solid transparent;
}
label.clip-step {
    font-size: 8pt; font-weight: 600; color: @clip_muted; background-color: alpha(white, 0.08); background-image: none;
    border-radius: 4px; min-width: 18px; padding: 1px 0; margin-top: 1px;
}
label.clip-step-text { font-size: 9pt; color: #c8c8cc; }
window.clippit-dashboard row.expander list.nested,
window.clippit-dashboard row.expander list.nested > row {
    background-color: transparent; background-image: none; border: none; box-shadow: none;
}
window.clippit-dashboard row.expander box.header { min-height: 48px; }
window.clippit-dashboard row.expander image.expander-row-arrow {
    opacity: 0; min-width: 0; min-height: 0; margin: 0; padding: 0; -gtk-icon-size: 0;
}
window.clippit-dashboard image.clip-arrow { color: @clip_faint; margin-left: 6px; }
window.clippit-dashboard row.expander:checked image.clip-arrow { color: @clip_text; }
/* Sem linhas entre os itens que abrem */
window.clippit-dashboard list.boxed-list > row.expander,
window.clippit-dashboard list.boxed-list > row.expander > box,
window.clippit-dashboard list.boxed-list > row.expander > box > list,
window.clippit-dashboard list.boxed-list > row.expander > box > revealer > list {
    border: none; box-shadow: none; outline: none; background-image: none;
}
window.clippit-dashboard list.boxed-list > row.expander > box > list > row,
window.clippit-dashboard list.boxed-list > row.expander > box > revealer > list > row {
    border: none; box-shadow: none; background-color: transparent;
}
window.clippit-dashboard list.boxed-list > row.expander:checked { background-color: alpha(white, 0.03); }
label.clip-key-big { font-size: 12pt; padding: 8px 14px; margin: 6px 0; }
label.clip-warn { color: #f5b14c; font-size: 9pt; }
"#;

pub fn install() {
    let provider = gtk::CssProvider::new();
    provider.connect_parsing_error(|_, section, err| {
        eprintln!("⚠️ CSS do painel: {} em {}", err, section);
    });
    provider.load_from_data(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            u32::MAX / 2, // acima do tema do Zorin (a libadwaita dele aplica o azul-claro por cima)
        );
    }
}
