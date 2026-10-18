pub mod protocol;

use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("omen-ry.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        std::env::temp_dir().join(format!("omen-ry-{}.sock", user))
    }
}
