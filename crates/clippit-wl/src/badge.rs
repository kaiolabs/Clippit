//! The spell-check badge: (spinner)  [N sugestões]  [✓]
//! Same look as the GNOME extension: 24 px tall pieces, dark glass, the
//! Clippit blue for the ✓.

use serde::Deserialize;

pub const H: i32 = 24;
const GAP: i32 = 6;
const RING: i32 = H;
const FIX_W: i32 = (H * 3) / 2;
const PAD_X: f64 = 9.0;
/// Room around the pieces for the shadow.
pub const MARGIN: i32 = 4;

#[derive(Clone, Default, Deserialize, PartialEq)]
pub struct Look {
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub label: String,
    /// idle | checking | ok | issues | error
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub busy: bool,
    #[serde(default)]
    pub fix: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Label,
    Fix,
}

pub struct Layout {
    pub width: i32,
    pub height: i32,
    label_x: i32,
    label_w: i32,
    fix_x: i32,
}

impl Layout {
    /// 1×1, for the hidden badge.
    pub fn empty() -> Self {
        Layout {
            width: 1,
            height: 1,
            label_x: 0,
            label_w: 0,
            fix_x: 0,
        }
    }

    pub fn part_at(&self, x: f64, y: f64, look: &Look) -> Option<Part> {
        let (x, y) = (x as i32, y as i32);
        if !(MARGIN..=MARGIN + H).contains(&y) {
            return None;
        }
        if x >= self.label_x && x <= self.label_x + self.label_w {
            return Some(Part::Label);
        }
        if look.fix && x >= self.fix_x && x <= self.fix_x + FIX_W {
            return Some(Part::Fix);
        }
        None
    }
}

fn font() -> pango::FontDescription {
    pango::FontDescription::from_string("Inter, Cantarell, sans-serif Semi-Bold 9")
}

fn text_width(text: &str) -> i32 {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).expect("cairo");
    let cr = cairo::Context::new(&surface).expect("cairo");
    let layout = pangocairo::functions::create_layout(&cr);
    layout.set_font_description(Some(&font()));
    layout.set_text(text);
    layout.pixel_size().0
}

pub fn layout(look: &Look) -> Layout {
    let label_w = text_width(&look.label) + (2.0 * PAD_X) as i32;
    let label_x = MARGIN + RING + GAP;
    let fix_x = label_x + label_w + GAP;
    let right = if look.fix {
        fix_x + FIX_W
    } else {
        label_x + label_w
    };
    Layout {
        width: right + MARGIN,
        height: H + 2 * MARGIN,
        label_x,
        label_w,
        fix_x,
    }
}

fn rounded(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::PI;
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    cr.close_path();
}

fn rgb(hex: u32, a: f64) -> (f64, f64, f64, f64) {
    (
        ((hex >> 16) & 0xff) as f64 / 255.0,
        ((hex >> 8) & 0xff) as f64 / 255.0,
        (hex & 0xff) as f64 / 255.0,
        a,
    )
}

fn piece(
    cr: &cairo::Context,
    x: f64,
    w: f64,
    fill: (f64, f64, f64, f64),
    border: Option<(f64, f64, f64, f64)>,
) {
    let y = MARGIN as f64;
    let h = H as f64;
    // Soft shadow under each piece.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
    rounded(cr, x, y + 2.0, w, h, 6.0);
    let _ = cr.fill();
    cr.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
    rounded(cr, x, y, w, h, 6.0);
    let _ = cr.fill();
    if let Some(b) = border {
        cr.set_source_rgba(b.0, b.1, b.2, b.3);
        cr.set_line_width(1.0);
        rounded(cr, x + 0.5, y + 0.5, w - 1.0, h - 1.0, 6.0);
        let _ = cr.stroke();
    }
}

/// Draws the badge at `scale` into ARGB8888 bytes (premultiplied, as wl_shm wants).
pub fn render(look: &Look, hover: Option<Part>, angle: f64, scale: i32) -> (Layout, Vec<u8>, i32) {
    let lay = layout(look);
    let (w, h) = (lay.width * scale, lay.height * scale);
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).expect("cairo");
    {
        let cr = cairo::Context::new(&surface).expect("cairo");
        cr.scale(scale as f64, scale as f64);
        let glass = rgb(0x1e1e22, 0.85);
        let line = (1.0, 1.0, 1.0, 0.12);

        if look.busy {
            let x = MARGIN as f64;
            piece(
                &cr,
                x,
                RING as f64,
                rgb(0x1e1e1e, 1.0),
                Some(rgb(0x333333, 1.0)),
            );
            let (cx, cy, r) = (x + RING as f64 / 2.0, MARGIN as f64 + H as f64 / 2.0, 5.5);
            cr.set_line_width(2.0);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.12);
            cr.arc(cx, cy, r, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.stroke();
            cr.set_source_rgba(0.298, 0.616, 1.0, 1.0);
            let a = angle - std::f64::consts::FRAC_PI_2;
            cr.arc(cx, cy, r, a, a + 0.75 * std::f64::consts::PI);
            let _ = cr.stroke();
        }

        let hovered = hover == Some(Part::Label);
        let fill = if hovered { rgb(0x2c2c2c, 1.0) } else { glass };
        let border = if hovered { rgb(0x424242, 1.0) } else { line };
        piece(
            &cr,
            lay.label_x as f64,
            lay.label_w as f64,
            fill,
            Some(border),
        );
        let color = match look.state.as_str() {
            "ok" => rgb(0x4ade80, 1.0),
            "issues" | "error" => rgb(0xff6b6b, 1.0),
            _ => rgb(0x9a9aa0, 1.0),
        };
        let layout = pangocairo::functions::create_layout(&cr);
        layout.set_font_description(Some(&font()));
        layout.set_text(&look.label);
        let (tw, th) = layout.pixel_size();
        cr.set_source_rgba(color.0, color.1, color.2, color.3);
        cr.move_to(
            lay.label_x as f64 + (lay.label_w - tw) as f64 / 2.0,
            MARGIN as f64 + (H - th) as f64 / 2.0,
        );
        pangocairo::functions::show_layout(&cr, &layout);

        if look.fix {
            let fill = if hover == Some(Part::Fix) {
                rgb(0x6cb0ff, 1.0)
            } else {
                rgb(0x4c9dff, 1.0)
            };
            piece(&cr, lay.fix_x as f64, FIX_W as f64, fill, None);
            // ✓
            let (x, y) = (
                lay.fix_x as f64 + FIX_W as f64 / 2.0,
                MARGIN as f64 + H as f64 / 2.0,
            );
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            cr.set_line_width(2.0);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.set_line_join(cairo::LineJoin::Round);
            cr.move_to(x - 4.5, y + 0.5);
            cr.line_to(x - 1.5, y + 3.5);
            cr.line_to(x + 4.5, y - 3.5);
            let _ = cr.stroke();
        }
    }
    let stride = surface.stride();
    let data = surface.data().expect("cairo data").to_vec();
    (lay, data, stride)
}
