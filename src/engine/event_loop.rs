use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::MissedTickBehavior;
use tracing::info;
use crate::ipc::protocol::{IpcRequest, IpcResponse};
use super::Engine;

pub async fn run_event_loop(
    mut engine: Engine,
    mut ipc_rx: mpsc::Receiver<(IpcRequest, oneshot::Sender<IpcResponse>)>,
) -> std::io::Result<()> {
    let frame_duration = Duration::from_micros((1_000_000.0 / engine.fps() as f64) as u64);
    let mut interval = tokio::time::interval(frame_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut last_tick = Instant::now();

    // Render initial frame
    let _ = engine.render_frame(Duration::from_millis(0));

    info!(
        "Engine event loop started (FPS: {}, Animating: {})",
        engine.fps(),
        engine.is_animating()
    );

    loop {
        let is_animating = engine.is_animating();

        tokio::select! {
            // Adaptive timer tick: ONLY ACTIVE WHEN ANIMATION IS RUNNING!
            // When is_animating == false, this branch is completely disabled.
            // Result: 0.0% CPU usage while idle/static.
            _ = interval.tick(), if is_animating => {
                let now = Instant::now();
                let dt = now.duration_since(last_tick);
                last_tick = now;

                if let Err(e) = engine.render_frame(dt) {
                    tracing::error!("Failed to render animation frame: {}", e);
                }
            }

            // IPC Request from CLI client
            Some((req, resp_tx)) = ipc_rx.recv() => {
                let is_shutdown = matches!(req, IpcRequest::Shutdown);
                let resp = engine.handle_ipc(req);
                let _ = resp_tx.send(resp);

                if is_shutdown {
                    info!("Shutdown requested via IPC");
                    break;
                }

                // If mode changed to static or dynamic, update frame immediately
                last_tick = Instant::now();
                let _ = engine.render_frame(Duration::from_millis(0));
            }

            // OS Termination signal (Ctrl+C / SIGTERM)
            _ = tokio::signal::ctrl_c() => {
                info!("Received termination signal, shutting down...");
                break;
            }
        }
    }

    info!("Event loop terminated cleanly");
    Ok(())
}
