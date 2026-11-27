//! clippit-wl — the Wayland side of the Quickshell interface of Clippit.
//!
//! What GNOME Shell gives the extension for free, done here for wlroots-style
//! compositors (Hyprland, Sway, niri…):
//!  - input-method-v2: the text of the focused field, the cursor and the field
//!    type (password fields are left alone), and editing it (delete + commit);
//!  - the spell-check badge next to the cursor, drawn as an input popup so the
//!    compositor places it at the caret (a client cannot know where it is);
//!  - the D-Bus service com.clippit.Lancador that the scripts call
//!    (clippit-historico, clippit-correct, clippit-doutor). Each call is handed
//!    to Quickshell, which answers on stdin.
//!
//! Quickshell runs this program and talks to it with one JSON object per line.
//! The text of the field never goes to a log.

mod badge;
mod bridge;
mod ime;

use std::io::Write;

use anyhow::Result;
use serde_json::{json, Value};
use smithay_client_toolkit::reexports::calloop::{self, EventLoop};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use wayland_client::{globals::registry_queue_init, Connection};

pub fn emit(value: Value) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{value}");
    let _ = out.flush();
}

pub enum Input {
    Line(String),
    Call(bridge::Call),
}

fn main() -> Result<()> {
    let conn = Connection::connect_to_env()?;
    let (globals, queue) = registry_queue_init::<ime::State>(&conn)?;
    let qh = queue.handle();

    let mut event_loop: EventLoop<ime::State> = EventLoop::try_new()?;
    let handle = event_loop.handle();
    WaylandSource::new(conn.clone(), queue)
        .insert(handle.clone())
        .map_err(|e| anyhow::anyhow!("wayland source: {e}"))?;

    let (tx, rx) = calloop::channel::channel::<Input>();
    handle
        .insert_source(rx, |event, _, state| {
            if let calloop::channel::Event::Msg(input) = event {
                match input {
                    Input::Line(line) => state.command(&line),
                    Input::Call(call) => state.call(call),
                }
            }
        })
        .map_err(|e| anyhow::anyhow!("channel: {e}"))?;

    let stdin_tx = tx.clone();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            match stdin.read_line(&mut line) {
                Ok(0) | Err(_) => std::process::exit(0), // Quickshell went away
                Ok(_) => {
                    if stdin_tx.send(Input::Line(line.trim().to_string())).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let (_bus, replies) = match bridge::serve(tx) {
        Ok((bus, replies)) => (Some(bus), replies),
        Err(e) => {
            eprintln!("clippit-wl: D-Bus indisponível: {e}");
            (None, bridge::Replies::default())
        }
    };

    let mut state = ime::State::new(&globals, &qh, handle.clone(), replies)?;
    emit(json!({"ev": "caps", "im": state.has_input_method()}));

    loop {
        event_loop.dispatch(None, &mut state)?;
    }
}
