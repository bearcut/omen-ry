use std::fs;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};
use crate::ipc::protocol::{IpcRequest, IpcResponse};

pub type IpcSender = mpsc::Sender<(IpcRequest, oneshot::Sender<IpcResponse>)>;

pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    pub fn bind<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let _ = fs::remove_file(path);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(path)?;
        info!("IPC socket listening on: {}", path.display());

        Ok(Self { listener })
    }

    pub async fn run(self, tx: IpcSender) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.into_split();
                        let mut lines = BufReader::new(reader).lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            if line.trim().is_empty() {
                                continue;
                            }

                            let req: IpcRequest = match serde_json::from_str(&line) {
                                Ok(r) => r,
                                Err(e) => {
                                    let err_resp = IpcResponse::Error {
                                        message: format!("Malformed request JSON: {}", e),
                                    };
                                    let _ = writer.write_all(serde_json::to_string(&err_resp).unwrap().as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                    continue;
                                }
                            };

                            let (resp_tx, resp_rx) = oneshot::channel();
                            if tx.send((req, resp_tx)).await.is_err() {
                                warn!("Daemon event loop closed IPC channel");
                                break;
                            }

                            if let Ok(resp) = resp_rx.await {
                                let mut serialized = serde_json::to_string(&resp).unwrap();
                                serialized.push('\n');
                                if let Err(e) = writer.write_all(serialized.as_bytes()).await {
                                    error!("Failed writing IPC response: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting IPC client connection: {}", e);
                }
            }
        }
    }
}
