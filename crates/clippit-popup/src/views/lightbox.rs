//! Visualização ampliada de imagens dentro da própria janela do popup (overlay).
//! Fica na mesma janela de propósito: uma janela separada tiraria o foco do popup
//! e dispararia o auto-close. Enquanto aberta, a janela é maximizada.
//!
//! Zoom: pinça no touchpad, Ctrl+roda, duplo clique, teclas + / - / 0 e botões.
//! Arrastar com o mouse move a imagem quando ela é maior que a tela.

use gtk::prelude::*;
use rust_i18n::t;
use std::cell::{Cell, RefCell};

const MAX_ZOOM: f64 = 8.0;

struct Lightbox {
    root: gtk::Box,
    scrolled: gtk::ScrolledWindow,
    picture: gtk::Picture,
    info: gtk::Label,
    zoom_label: gtk::Label,
    entry_id: Cell<i64>,
    img_w: Cell<f64>,
    img_h: Cell<f64>,
    zoom: Cell<f64>,
    /// Modo "ajustar à tela": refaz o ajuste quando a janela muda de tamanho
    fit: Cell<bool>,
    last_viewport: Cell<(i32, i32)>,
    /// Ponto que deve ficar parado sob o cursor depois do zoom (fração, posição)
    pending_x: Cell<Option<(f64, f64)>>,
    pending_y: Cell<Option<(f64, f64)>>,
    pointer: Cell<(f64, f64)>,
    maximized_by_us: Cell<bool>,
}

thread_local! {
    static LIGHTBOX: RefCell<Option<Lightbox>> = const { RefCell::new(None) };
}

fn with_lb<R>(f: impl FnOnce(&Lightbox) -> R) -> Option<R> {
    LIGHTBOX.with(|l| l.borrow().as_ref().map(f))
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::from_icon_name(icon);
    b.add_css_class("clip-action");
    b.set_tooltip_text(Some(tooltip));
    b
}

/// Cria o overlay (escondido) por cima do conteúdo do popup
pub fn install(overlay: &gtk::Overlay) {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("clip-lightbox");
    root.set_visible(false);

    // Barra superior: info | − 100% + ajustar | copiar fechar
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let info = gtk::Label::new(None);
    info.add_css_class("clip-lightbox-info");
    info.set_hexpand(true);
    info.set_halign(gtk::Align::Start);

    let zoom_out = icon_button("zoom-out-symbolic", &t!("popup.zoom_out"));
    let zoom_label = gtk::Label::new(Some("100%"));
    zoom_label.add_css_class("clip-zoom-label");
    zoom_label.set_width_chars(5);
    let zoom_in = icon_button("zoom-in-symbolic", &t!("popup.zoom_in"));
    let zoom_fit = icon_button("zoom-fit-best-symbolic", &t!("popup.zoom_fit"));
    let copy_btn = icon_button("edit-copy-symbolic", &t!("popup.copy_tooltip"));
    copy_btn.add_css_class("clip-copy");
    let close_btn = icon_button("window-close-symbolic", &t!("popup.hint_close"));

    let sep = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    sep.set_width_request(10);

    bar.append(&info);
    bar.append(&zoom_out);
    bar.append(&zoom_label);
    bar.append(&zoom_in);
    bar.append(&zoom_fit);
    bar.append(&sep);
    bar.append(&copy_btn);
    bar.append(&close_btn);

    // Imagem dentro de um ScrolledWindow: o tamanho mínimo da Picture define o zoom
    let picture = gtk::Picture::new();
    picture.set_keep_aspect_ratio(true);
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_child(Some(&picture));
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.add_css_class("clip-lightbox-view");
    // Precisa poder receber foco, senão o foco fica no campo de busca por baixo
    scrolled.set_focusable(true);

    root.append(&bar);
    root.append(&scrolled);

    // Mantém o ponto sob o cursor parado depois que o layout novo é calculado
    scrolled.hadjustment().connect_changed(|adj| {
        if let Some(Some((frac, anchor))) = with_lb(|lb| lb.pending_x.take()) {
            adj.set_value(frac * adj.upper() - anchor);
        }
    });
    scrolled.vadjustment().connect_changed(|adj| {
        if let Some(Some((frac, anchor))) = with_lb(|lb| lb.pending_y.take()) {
            adj.set_value(frac * adj.upper() - anchor);
        }
    });

    // Reajusta no modo "ajustar à tela" quando a área muda (ex.: ao maximizar)
    scrolled.add_tick_callback(|sw, _| {
        with_lb(|lb| {
            if !lb.root.is_visible() {
                return;
            }
            let size = (sw.width(), sw.height());
            if size != lb.last_viewport.get() && size.0 > 0 && size.1 > 0 {
                lb.last_viewport.set(size);
                if lb.fit.get() {
                    set_zoom(lb, fit_zoom(lb), None);
                } else {
                    set_zoom(lb, lb.zoom.get(), None);
                }
            }
        });
        gtk::glib::ControlFlow::Continue
    });

    // Posição do ponteiro (para o zoom com Ctrl+roda seguir o cursor)
    let motion = gtk::EventControllerMotion::new();
    motion.connect_motion(|_, x, y| {
        with_lb(|lb| lb.pointer.set((x, y)));
    });
    scrolled.add_controller(motion);

    // Ctrl+roda = zoom (roda sozinha continua rolando)
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    scroll.connect_scroll(|ctrl, _dx, dy| {
        if !ctrl
            .current_event_state()
            .contains(gtk::gdk::ModifierType::CONTROL_MASK)
        {
            return gtk::glib::Propagation::Proceed;
        }
        with_lb(|lb| {
            let factor = 1.15_f64.powf(-dy);
            set_zoom(lb, lb.zoom.get() * factor, Some(lb.pointer.get()));
        });
        gtk::glib::Propagation::Stop
    });
    scrolled.add_controller(scroll);

    // Pinça no touchpad / tela de toque
    let pinch = gtk::GestureZoom::new();
    let start_zoom = std::rc::Rc::new(Cell::new(1.0));
    let start_zoom_b = start_zoom.clone();
    pinch.connect_begin(move |_, _| {
        with_lb(|lb| start_zoom_b.set(lb.zoom.get()));
    });
    pinch.connect_scale_changed(move |g, scale| {
        let anchor = g.bounding_box_center();
        with_lb(|lb| set_zoom(lb, start_zoom.get() * scale, anchor));
    });
    scrolled.add_controller(pinch);

    // Arrastar para mover
    let drag = gtk::GestureDrag::new();
    let drag_start = std::rc::Rc::new(Cell::new((0.0, 0.0)));
    let drag_start_b = drag_start.clone();
    drag.connect_drag_begin(move |g, _, _| {
        with_lb(|lb| {
            drag_start_b.set((
                lb.scrolled.hadjustment().value(),
                lb.scrolled.vadjustment().value(),
            ));
        });
        if let Some(w) = g.widget() {
            w.set_cursor_from_name(Some("grabbing"));
        }
    });
    drag.connect_drag_update(move |_, ox, oy| {
        let (sx, sy) = drag_start.get();
        with_lb(|lb| {
            lb.scrolled.hadjustment().set_value(sx - ox);
            lb.scrolled.vadjustment().set_value(sy - oy);
        });
    });
    drag.connect_drag_end(|g, _, _| {
        if let Some(w) = g.widget() {
            w.set_cursor_from_name(Some("grab"));
        }
    });
    scrolled.add_controller(drag);

    // Duplo clique: alterna entre ajustar à tela e ampliar no ponto clicado
    let dbl = gtk::GestureClick::new();
    dbl.connect_pressed(|_, n, x, y| {
        if n == 2 {
            with_lb(|lb| {
                let fit = fit_zoom(lb);
                if lb.zoom.get() > fit * 1.05 {
                    lb.fit.set(true);
                    set_zoom(lb, fit, None);
                } else {
                    set_zoom(lb, (fit * 2.5).max(1.0), Some((x, y)));
                }
            });
        }
    });
    scrolled.add_controller(dbl);

    zoom_in.connect_clicked(|_| {
        with_lb(|lb| set_zoom(lb, lb.zoom.get() * 1.25, None));
    });
    zoom_out.connect_clicked(|_| {
        with_lb(|lb| set_zoom(lb, lb.zoom.get() / 1.25, None));
    });
    zoom_fit.connect_clicked(|_| {
        with_lb(|lb| {
            lb.fit.set(true);
            set_zoom(lb, fit_zoom(lb), None);
        });
    });
    close_btn.connect_clicked(|_| {
        hide();
    });
    copy_btn.connect_clicked(|btn| {
        if let Some(id) = with_lb(|lb| lb.entry_id.get()) {
            crate::controllers::copy_to_clipboard(id);
            hide();
            if let Some(win) = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok()) {
                win.close();
            }
        }
    });

    overlay.add_overlay(&root);

    LIGHTBOX.with(|l| {
        *l.borrow_mut() = Some(Lightbox {
            root,
            scrolled,
            picture,
            info,
            zoom_label,
            entry_id: Cell::new(0),
            img_w: Cell::new(1.0),
            img_h: Cell::new(1.0),
            zoom: Cell::new(1.0),
            fit: Cell::new(true),
            last_viewport: Cell::new((0, 0)),
            pending_x: Cell::new(None),
            pending_y: Cell::new(None),
            pointer: Cell::new((0.0, 0.0)),
            maximized_by_us: Cell::new(false),
        });
    });
}

/// Zoom que faz a imagem caber inteira na área visível (nunca acima de 100%)
fn fit_zoom(lb: &Lightbox) -> f64 {
    let (vw, vh) = (lb.scrolled.width() as f64, lb.scrolled.height() as f64);
    if vw <= 0.0 || vh <= 0.0 {
        return 1.0;
    }
    (vw / lb.img_w.get()).min(vh / lb.img_h.get()).min(1.0)
}

/// Aplica o zoom. `anchor` = ponto (coordenadas do ScrolledWindow) que deve ficar
/// parado; sem anchor, usa o centro da área visível.
fn set_zoom(lb: &Lightbox, zoom: f64, anchor: Option<(f64, f64)>) {
    let fit = fit_zoom(lb);
    // Abaixo do "ajustar" não faz sentido: a imagem já cabe inteira
    let zoom = zoom.clamp(fit, MAX_ZOOM.max(fit));
    lb.fit.set((zoom - fit).abs() < 1e-3);

    let (vw, vh) = (lb.scrolled.width() as f64, lb.scrolled.height() as f64);
    let (ax, ay) = anchor.unwrap_or((vw / 2.0, vh / 2.0));
    let (old_w, old_h) = (
        lb.img_w.get() * lb.zoom.get(),
        lb.img_h.get() * lb.zoom.get(),
    );
    let (new_w, new_h) = (lb.img_w.get() * zoom, lb.img_h.get() * zoom);

    // Fração da imagem que está sob o ponto de âncora, em cada eixo
    let frac = |value: f64, anchor: f64, old_len: f64, view: f64| {
        let offset = ((view - old_len) / 2.0).max(0.0);
        ((value + anchor - offset) / old_len.max(1.0)).clamp(0.0, 1.0)
    };
    let hadj = lb.scrolled.hadjustment();
    let vadj = lb.scrolled.vadjustment();
    lb.pending_x
        .set((new_w > vw).then(|| (frac(hadj.value(), ax, old_w, vw), ax)));
    lb.pending_y
        .set((new_h > vh).then(|| (frac(vadj.value(), ay, old_h, vh), ay)));

    lb.zoom.set(zoom);
    lb.picture
        .set_size_request(new_w.round() as i32, new_h.round() as i32);
    lb.zoom_label.set_text(&format!("{:.0}%", zoom * 100.0));

    let cursor = if new_w > vw + 1.0 || new_h > vh + 1.0 {
        "grab"
    } else {
        "default"
    };
    lb.scrolled.set_cursor_from_name(Some(cursor));
}

/// Mostra a imagem de uma entrada (precisa estar na lista, para sabermos o caminho)
pub fn show_entry(entry_id: i64) -> bool {
    let Some((path, dims)) = crate::controllers::image_info(entry_id) else {
        return false;
    };
    let texture = match gtk::gdk::Texture::from_filename(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Não foi possível abrir a imagem {}: {}", path, e);
            return false;
        }
    };
    with_lb(|lb| {
        lb.img_w.set(texture.width().max(1) as f64);
        lb.img_h.set(texture.height().max(1) as f64);
        lb.picture.set_paintable(Some(&texture));
        let dims = dims
            .map(|(w, h)| format!("{}x{}", w, h))
            .unwrap_or_else(|| format!("{}x{}", texture.width(), texture.height()));
        lb.info.set_text(&dims);
        lb.entry_id.set(entry_id);
        lb.fit.set(true);
        lb.zoom.set(1.0);
        lb.last_viewport.set((0, 0)); // força o ajuste no próximo frame
        lb.picture.set_size_request(-1, -1);
        lb.root.set_visible(true);

        // Maximiza a janela para ver a imagem grande (desfaz ao fechar)
        if let Some(win) = lb
            .root
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok())
        {
            if !win.is_maximized() {
                win.maximize();
                lb.maximized_by_us.set(true);
            }
        }
        lb.scrolled.grab_focus();
        true
    })
    .unwrap_or(false)
}

/// Esconde; retorna true se estava visível
pub fn hide() -> bool {
    with_lb(|lb| {
        if !lb.root.is_visible() {
            return false;
        }
        lb.root.set_visible(false);
        lb.picture.set_paintable(gtk::gdk::Paintable::NONE);
        lb.picture.set_size_request(-1, -1);
        let win = lb
            .root
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok());
        if lb.maximized_by_us.replace(false) {
            if let Some(ref w) = win {
                w.unmaximize();
            }
        }
        // Devolve o foco para o item selecionado da lista
        if let Some(w) = win {
            if let Some(list) = find_list(w.upcast_ref()) {
                if let Some(row) = list.selected_row() {
                    row.grab_focus();
                }
            }
        }
        true
    })
    .unwrap_or(false)
}

fn find_list(w: &gtk::Widget) -> Option<gtk::ListBox> {
    if let Ok(list) = w.clone().downcast::<gtk::ListBox>() {
        return Some(list);
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(found) = find_list(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

pub fn is_visible() -> bool {
    with_lb(|lb| lb.root.is_visible()).unwrap_or(false)
}

/// Teclas do modo ampliado: + / - / 0. Retorna true se tratou a tecla.
pub fn handle_key(key: gtk::gdk::Key) -> bool {
    use gtk::gdk::Key;
    if !is_visible() {
        return false;
    }
    with_lb(|lb| match key {
        Key::plus | Key::equal | Key::KP_Add => {
            set_zoom(lb, lb.zoom.get() * 1.25, None);
            true
        }
        Key::minus | Key::KP_Subtract => {
            set_zoom(lb, lb.zoom.get() / 1.25, None);
            true
        }
        Key::_0 | Key::KP_0 => {
            lb.fit.set(true);
            set_zoom(lb, fit_zoom(lb), None);
            true
        }
        // Enter copia a imagem aberta e fecha o popup
        Key::Return | Key::KP_Enter => {
            crate::controllers::copy_to_clipboard(lb.entry_id.get());
            let win = lb
                .root
                .root()
                .and_then(|r| r.downcast::<gtk::Window>().ok());
            lb.root.set_visible(false);
            if lb.maximized_by_us.replace(false) {
                if let Some(ref w) = win {
                    w.unmaximize();
                }
            }
            if let Some(w) = win {
                w.close();
            }
            true
        }
        Key::_1 | Key::KP_1 => {
            set_zoom(lb, 1.0, None);
            true
        }
        _ => false,
    })
    .unwrap_or(false)
}

/// Botão "ampliar" para linhas de imagem
pub fn add_expand_button(row: &libadwaita::ActionRow, entry_id: i64) {
    use libadwaita::prelude::*;
    let btn = gtk::Button::from_icon_name("view-fullscreen-symbolic");
    btn.set_valign(gtk::Align::Center);
    btn.add_css_class("clip-action");
    btn.add_css_class("clip-expand");
    btn.set_tooltip_text(Some(&t!("popup.expand_tooltip")));
    btn.connect_clicked(move |_| {
        show_entry(entry_id);
    });
    row.add_suffix(&btn);
}
