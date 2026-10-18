use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use crate::color::Color;
use crate::ipc::protocol::{DaemonStatus, IpcRequest, IpcResponse};

pub async fn send_request<P: AsRef<Path>>(
    socket_path: P,
    req: IpcRequest,
) -> Result<IpcResponse, String> {
    let path = socket_path.as_ref();
    if !path.exists() {
        return Err(format!(
            "Cannot connect to omen-ry daemon: socket '{}' not found.\nIs the daemon running? (Start it with 'omen-ry daemon')",
            path.display()
        ));
    }

    let stream = UnixStream::connect(path)
        .await
        .map_err(|e| format!("Failed to connect to daemon at {}: {}", path.display(), e))?;

    let (reader, mut writer) = stream.into_split();

    let mut serialized = serde_json::to_string(&req)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    serialized.push('\n');

    writer
        .write_all(serialized.as_bytes())
        .await
        .map_err(|e| format!("Failed to send command to daemon: {}", e))?;

    let mut lines = BufReader::new(reader).lines();
    if let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| format!("Failed to read response from daemon: {}", e))?
    {
        serde_json::from_str::<IpcResponse>(&line)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err("Daemon closed connection without response".to_string())
    }
}

pub fn print_status(status: &DaemonStatus) {
    println!("\x1b[1mHP OMEN RGB Daemon Status\x1b[0m");
    println!("──────────────────────────────────────────────");
    println!("  Mode:         \x1b[36m{}\x1b[0m", status.mode);
    println!("  Animating:    {}", if status.is_animating { "\x1b[32myes\x1b[0m" } else { "\x1b[33mno (quiescent - 0.0% CPU)\x1b[0m" });
    println!("  Brightness:   {}%", status.brightness);
    println!("  Framerate:    {} FPS", status.fps);
    println!("  Backend:      {}", status.backend);
    println!("  Zones:");

    let swatches: Vec<String> = status
        .current_colors
        .iter()
        .enumerate()
        .map(|(i, hex)| {
            let color = Color::from_hex(hex).unwrap_or(Color::BLACK);
            format!(
                "    Zone {}: \x1b[48;2;{};{};{}m    \x1b[0m  {}",
                i, color.r, color.g, color.b, hex
            )
        })
        .collect();

    for s in swatches {
        println!("{}", s);
    }
    println!("──────────────────────────────────────────────");
}
