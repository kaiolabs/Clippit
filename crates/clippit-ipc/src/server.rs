use anyhow::{Context, Result};
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::Listener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use tracing::{error, info};

use crate::protocol::{IpcMessage, IpcResponse, SOCKET_PATH};

pub type ServerCallback = Box<dyn Fn(IpcMessage) -> IpcResponse + Send + Sync>;

pub struct IpcServer {
    callback: ServerCallback,
}

impl IpcServer {
    pub fn new(callback: ServerCallback) -> Self {
        Self { callback }
    }

    pub async fn start(&self) -> Result<()> {
        // Remove existing socket if present
        let socket_path = Path::new(SOCKET_PATH);
        if socket_path.exists() {
            std::fs::remove_file(socket_path).context("Failed to remove existing socket file")?;
        }

        let name = SOCKET_PATH.to_ns_name::<GenericNamespaced>()?;
        let listener = ListenerOptions::new()
            .name(name)
            .create_sync()
            .context("Failed to create socket listener")?;

        info!("IPC server listening on {}", SOCKET_PATH);

        loop {
            match listener.accept() {
                Ok(stream) => {
                    let callback = &self.callback;
                    if let Err(e) = self.handle_connection(stream, callback) {
                        error!("Error handling connection: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    fn handle_connection(
        &self,
        mut stream: LocalSocketStream,
        callback: &ServerCallback,
    ) -> Result<()> {
        let mut line = String::new();
        {
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut line)?;
        }

        if line.is_empty() {
            return Ok(());
        }

        let message: IpcMessage =
            serde_json::from_str(&line).context("Failed to deserialize message")?;

        info!("Received message: {:?}", message);

        let response = callback(message);
        let response_json = serde_json::to_string(&response)?;

        writeln!(stream, "{}", response_json)?;
        stream.flush()?;

        Ok(())
    }
}
