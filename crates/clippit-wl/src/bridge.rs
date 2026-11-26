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

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<String>>>>;

static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

/// Delivers Quickshell's answer to the D-Bus call waiting for it.
pub fn reply(id: u64, value: String) {
    let pending = PENDING.lock().unwrap().clone();
    if let Some(tx) = pending.and_then(|p| p.lock().unwrap().remove(&id)) {
        let _ = tx.send(value);
    }
}

struct Lancador {
    tx: Mutex<Sender<Input>>,
    pending: Pending,
    next: AtomicU64,
}

impl Lancador {
    async fn ask(&self, method: &'static str, args: Vec<String>, timeout: Duration) -> String {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        if self
            .tx
            .lock()
            .unwrap()
            .send(Input::Call(Call { id, method, args }))
            .is_err()
        {
            return String::new();
        }
        let timer = async {
            // zbus runs on its own executor: a plain thread is enough for the timeout.
            let (ttx, trx) = oneshot::channel::<()>();
            std::thread::spawn(move || {
                std::thread::sleep(timeout);
                let _ = ttx.send(());
            });
            let _ = trx.await;
        };
        let answer = futures_or(rx, timer).await;
        self.pending.lock().unwrap().remove(&id);
        answer.unwrap_or_default()
    }
}

/// The first of the two to finish: the answer, or None on timeout.
async fn futures_or(
    rx: oneshot::Receiver<String>,
    timer: impl std::future::Future<Output = ()>,
) -> Option<String> {
    use std::future::Future;
    use std::pin::pin;
    use std::task::Poll;
    let mut rx = pin!(rx);
    let mut timer = pin!(timer);
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

pub fn serve(tx: Sender<Input>) -> zbus::Result<zbus::blocking::Connection> {
    let pending: Pending = Arc::default();
    *PENDING.lock().unwrap() = Some(pending.clone());
    let service = Lancador {
        tx: Mutex::new(tx),
        pending,
        next: AtomicU64::new(1),
    };
    let conn = zbus::blocking::connection::Builder::session()?
        .serve_at("/com/clippit/Lancador", service)?
        .build()?;
    conn.request_name_with_flags(
        "com.clippit.Lancador",
        RequestNameFlags::ReplaceExisting | RequestNameFlags::AllowReplacement,
    )?;
    Ok(conn)
}
