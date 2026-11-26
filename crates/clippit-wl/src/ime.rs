//! input-method-v2 and the badge popup.

use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay_client_toolkit::reexports::calloop::{LoopHandle, RegistrationToken};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shm::slot::SlotPool;
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use smithay_client_toolkit::{
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_shm, registry_handlers,
};
use wayland_client::globals::GlobalList;
use wayland_client::protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::{self, ZwpInputMethodV2},
    zwp_input_popup_surface_v2::{self, ZwpInputPopupSurfaceV2},
};

use crate::badge::{self, Look, Part};
use crate::{bridge, emit};

/// What the input method knows about the focused field (applied on "done").
#[derive(Clone, Default, PartialEq)]
struct Field {
    active: bool,
    text: String,
    cursor: usize, // bytes
    anchor: usize, // bytes
    purpose: u32,
    hints: u32,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
enum Command {
    /// Deletes `before`/`after` characters around the cursor and writes `text`.
    Edit {
        #[serde(default)]
        before: usize,
        #[serde(default)]
        after: usize,
        #[serde(default)]
        text: String,
    },
    Badge(Look),
    Reply {
        id: u64,
        #[serde(default)]
        value: String,
    },
}

pub struct State {
    registry: RegistryState,
    seats: SeatState,
    outputs: OutputState,
    compositor: CompositorState,
    shm: Shm,
    pool: Option<SlotPool>,
    qh: QueueHandle<State>,
    handle: LoopHandle<'static, State>,

    manager: Option<ZwpInputMethodManagerV2>,
    im: Option<ZwpInputMethodV2>,
    serial: u32,
    pending: Field,
    field: Field,
    unavailable: bool,

    surface: Option<wl_surface::WlSurface>,
    popup: Option<ZwpInputPopupSurfaceV2>,
    scale: i32,
    look: Look,
    hover: Option<Part>,
    angle: f64,
    spin: Option<RegistrationToken>,
    layout: Option<badge::Layout>,
}

impl State {
    pub fn new(
        globals: &GlobalList,
        qh: &QueueHandle<State>,
        handle: LoopHandle<'static, State>,
    ) -> Result<Self> {
        let compositor = CompositorState::bind(globals, qh)?;
        let shm = Shm::bind(globals, qh)?;
        let manager = globals
            .bind::<ZwpInputMethodManagerV2, _, _>(qh, 1..=1, ())
            .ok();
        let mut state = State {
            registry: RegistryState::new(globals),
            seats: SeatState::new(globals, qh),
            outputs: OutputState::new(globals, qh),
            compositor,
            shm,
            pool: None,
            qh: qh.clone(),
            handle,
            manager,
            im: None,
            serial: 0,
            pending: Field::default(),
            field: Field::default(),
            unavailable: false,
            surface: None,
            popup: None,
            scale: 1,
            look: Look::default(),
            hover: None,
            angle: 0.0,
            spin: None,
            layout: None,
        };
        state.bind_input_method();
        Ok(state)
    }

    pub fn has_input_method(&self) -> bool {
        self.manager.is_some() && !self.unavailable
    }

    fn bind_input_method(&mut self) {
        if self.im.is_some() {
            return;
        }
        let (Some(manager), Some(seat)) = (self.manager.as_ref(), self.seats.seats().next()) else {
            return;
        };
        let im = manager.get_input_method(&seat, &self.qh, ());
        let surface = self.compositor.create_surface(&self.qh);
        let popup = im.get_input_popup_surface(&surface, &self.qh, ());
        self.im = Some(im);
        self.surface = Some(surface);
        self.popup = Some(popup);
    }

    pub fn command(&mut self, line: &str) {
        let Ok(cmd) = serde_json::from_str::<Command>(line) else {
            return;
        };
        match cmd {
            Command::Edit {
                before,
                after,
                text,
            } => self.edit(before, after, &text),
            Command::Badge(look) => {
                if look != self.look {
                    self.look = look;
                    self.sync_spin();
                    self.draw();
                }
            }
            Command::Reply { id, value } => bridge::reply(id, value),
        }
    }

    pub fn call(&mut self, call: bridge::Call) {
        emit(json!({"ev": "call", "id": call.id, "method": call.method, "args": call.args}));
    }

    /// Characters → bytes around the cursor, as the protocol counts.
    fn edit(&mut self, before: usize, after: usize, text: &str) {
        let Some(im) = self.im.as_ref() else {
            return;
        };
        if !self.field.active {
            return;
        }
        let t = &self.field.text;
        let cur = self.field.cursor.min(t.len());
        let head = &t[..cur];
        let tail = &t[cur..];
        let before_bytes: usize = head.chars().rev().take(before).map(char::len_utf8).sum();
        let after_bytes: usize = tail.chars().take(after).map(char::len_utf8).sum();
        if before_bytes > 0 || after_bytes > 0 {
            im.delete_surrounding_text(before_bytes as u32, after_bytes as u32);
        }
        if !text.is_empty() {
            im.commit_string(text.to_string());
        }
        im.commit(self.serial);
    }

    fn emit_field(&self) {
        let f = &self.field;
        let cursor = f
            .text
            .get(..f.cursor.min(f.text.len()))
            .map_or(0, |s| s.chars().count());
        let anchor = f
            .text
            .get(..f.anchor.min(f.text.len()))
            .map_or(0, |s| s.chars().count());
        emit(json!({
            "ev": "field", "active": f.active, "text": f.text, "cursor": cursor, "anchor": anchor,
            "purpose": f.purpose, "hints": f.hints,
        }));
    }

    fn sync_spin(&mut self) {
        let want = self.look.visible && self.look.busy;
        if want && self.spin.is_none() {
            let token = self
                .handle
                .insert_source(
                    Timer::from_duration(Duration::from_millis(33)),
                    |_, _, state| {
                        state.angle = (state.angle + 0.26) % (2.0 * std::f64::consts::PI);
                        state.draw();
                        TimeoutAction::ToDuration(Duration::from_millis(33))
                    },
                )
                .ok();
            self.spin = token;
        } else if !want {
            if let Some(token) = self.spin.take() {
                self.handle.remove(token);
            }
        }
    }

    fn draw(&mut self) {
        let Some(surface) = self.surface.clone() else {
            return;
        };
        if !self.look.visible || !self.field.active {
            surface.attach(None, 0, 0);
            surface.commit();
            self.layout = None;
            return;
        }
        let (lay, data, stride) = badge::render(&self.look, self.hover, self.angle, self.scale);
        let (w, h) = (lay.width * self.scale, lay.height * self.scale);
        let pool = match self.pool.as_mut() {
            Some(pool) => pool,
            None => match SlotPool::new((w * h * 4) as usize, &self.shm) {
                Ok(pool) => self.pool.insert(pool),
                Err(_) => return,
            },
        };
        let Ok((buffer, canvas)) = pool.create_buffer(w, h, stride, wl_shm::Format::Argb8888)
        else {
            return;
        };
        let n = canvas.len().min(data.len());
        canvas[..n].copy_from_slice(&data[..n]);
        surface.set_buffer_scale(self.scale);
        if buffer.attach_to(&surface).is_err() {
            return;
        }
        surface.damage_buffer(0, 0, w, h);
        surface.commit();
        self.layout = Some(lay);
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpInputMethodManagerV2,
        _: <ZwpInputMethodManagerV2 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputPopupSurfaceV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpInputPopupSurfaceV2,
        _: zwp_input_popup_surface_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwp_input_method_v2::Event;
        match event {
            Event::Activate => {
                state.pending = Field {
                    active: true,
                    ..Field::default()
                };
            }
            Event::Deactivate => {
                state.pending.active = false;
            }
            Event::SurroundingText {
                text,
                cursor,
                anchor,
            } => {
                state.pending.text = text;
                state.pending.cursor = cursor as usize;
                state.pending.anchor = anchor as usize;
            }
            Event::ContentType { hint, purpose } => {
                state.pending.hints = u32::from(hint);
                state.pending.purpose = u32::from(purpose);
            }
            Event::Done => {
                state.serial = state.serial.wrapping_add(1);
                if state.pending != state.field {
                    let was_active = state.field.active;
                    state.field = state.pending.clone();
                    state.emit_field();
                    if was_active != state.field.active {
                        state.draw();
                    }
                }
            }
            Event::Unavailable => {
                // Another input method (fcitx, ibus) already has the seat.
                state.unavailable = true;
                state.im = None;
                emit(json!({"ev": "caps", "im": false}));
            }
            _ => {}
        }
    }
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        factor: i32,
    ) {
        if factor != self.scale {
            self.scale = factor.max(1);
            self.draw();
        }
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.outputs
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seats
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
        self.bind_input_method();
    }
    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        cap: Capability,
    ) {
        if cap == Capability::Pointer {
            let _ = self.seats.get_pointer(qh, &seat);
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for State {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for ev in events {
            if Some(&ev.surface) != self.surface.as_ref() {
                continue;
            }
            let part = self
                .layout
                .as_ref()
                .and_then(|l| l.part_at(ev.position.0, ev.position.1, &self.look));
            match ev.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    if part != self.hover {
                        self.hover = part;
                        self.draw();
                    }
                }
                PointerEventKind::Leave { .. } => {
                    if self.hover.take().is_some() {
                        self.draw();
                    }
                }
                PointerEventKind::Press { button: 0x110, .. } => {
                    if let Some(part) = part {
                        emit(
                            json!({"ev": "badge", "part": if part == Part::Fix { "fix" } else { "label" }}),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![SeatState, OutputState];
}

delegate_compositor!(State);
delegate_shm!(State);
delegate_output!(State);
delegate_seat!(State);
delegate_pointer!(State);
delegate_registry!(State);
