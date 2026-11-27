//! com.clippit.Lancador on the session bus, the same interface the GNOME
//! extension exports. Every method is answered by Quickshell: the call goes
//! out on stdout ({"ev":"call"}) and the answer comes back on stdin
//! ({"cmd":"reply"}).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_channel::oneshot;
use smithay_client_toolkit::reexports::calloop::channel::Sender;
use zbus::fdo::RequestNameFlags;

use crate::Input;

pub struct Call {
    pub id: u64,
    pub method: &'static str,
    pub args: Vec<String>,
}

/// Calls waiting for Quickshell's answer, shared by the D-Bus side and the stdin side.
#[derive(Clone, Default)]
pub struct Replies(Arc<Mutex<HashMap<u64, oneshot::Sender<String>>>>);

impl Replies {
    // A poisoned lock only means another thread panicked mid-insert; the map is still usable.
    fn map(&self) -> std::sync::MutexGuard<'_, HashMap<u64, oneshot::Sender<String>>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Delivers Quickshell's answer to the D-Bus call waiting for it.
    pub fn deliver(&self, id: u64, value: String) {
        if let Some(tx) = self.map().remove(&id) {
            let _ = tx.send(value);
        }
    }
}

struct Lancador {
    tx: Mutex<Sender<Input>>,
    replies: Replies,
    next: AtomicU64,
}

impl Lancador {
    async fn ask(&self, method: &'static str, args: Vec<String>, timeout: Duration) -> String {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.replies.map().insert(id, tx);
        let sent = self
            .tx
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .send(Input::Call(Call { id, method, args }));
        if sent.is_err() {
            self.replies.map().remove(&id);
            return String::new();
        }
        let answer = answer_or_timeout(rx, timeout).await;
        self.replies.map().remove(&id);
        answer.unwrap_or_default()
    }
}

/// The answer, or None if the timeout comes first or the sender is gone.
async fn answer_or_timeout(rx: oneshot::Receiver<String>, timeout: Duration) -> Option<String> {
    use std::future::Future;
    use std::pin::pin;
    use std::task::Poll;
    let mut rx = pin!(rx);
    let mut timer = pin!(async_io::Timer::after(timeout));
    std::future::poll_fn(move |cx| {
        if let Poll::Ready(v) = rx.as_mut().poll(cx) {
            return Poll::Ready(v.ok());
        }
        if timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}

#[zbus::interface(name = "com.clippit.Lancador")]
impl Lancador {
    #[zbus(name = "Abrir")]
    async fn abrir(&self, tela: String) {
        self.ask("Abrir", vec![tela], Duration::from_secs(2)).await;
    }

    #[zbus(name = "Janelas")]
    async fn janelas(&self) -> String {
        self.ask("Janelas", vec![], Duration::from_secs(5)).await
    }

    #[zbus(name = "Janela")]
    async fn janela(&self, acao: String, alvo: String) -> String {
        self.ask("Janela", vec![acao, alvo], Duration::from_secs(5))
            .await
    }

    #[zbus(name = "Print")]
    async fn print(&self, onde: String, caminho: String) -> String {
        self.ask("Print", vec![onde, caminho], Duration::from_secs(15))
            .await
    }

    #[zbus(name = "AreaHash")]
    async fn area_hash(&self) -> String {
        self.ask("AreaHash", vec![], Duration::from_secs(5)).await
    }
}

pub fn serve(tx: Sender<Input>) -> zbus::Result<(zbus::blocking::Connection, Replies)> {
    let replies = Replies::default();
    let service = Lancador {
        tx: Mutex::new(tx),
        replies: replies.clone(),
        next: AtomicU64::new(1),
    };
    let conn = zbus::blocking::connection::Builder::session()?
        .serve_at("/com/clippit/Lancador", service)?
        .build()?;
    conn.request_name_with_flags(
        "com.clippit.Lancador",
        RequestNameFlags::ReplaceExisting | RequestNameFlags::AllowReplacement,
    )?;
    Ok((conn, replies))
}
