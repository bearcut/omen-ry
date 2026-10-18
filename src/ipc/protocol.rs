use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcRequest {
    GetStatus,
    SetModeWave {
        palette: Option<String>,
        speed: Option<f32>,
        reverse: Option<bool>,
    },
    SetModeStatic {
        colors: [String; 4],
    },
    SetColorAll {
        color: String,
    },
    SetColorZone {
        zone: usize,
        color: String,
    },
    SetBrightness {
        brightness: u8,
    },
    Pause,
    Resume,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub mode: String,
    pub brightness: u8,
    pub fps: u32,
    pub backend: String,
    pub current_colors: [String; 4],
    pub is_animating: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum IpcResponse {
    Ok { message: String },
    Status(DaemonStatus),
    Error { message: String },
}
