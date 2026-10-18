# omen-ry: Lightweight Event-Driven RGB Daemon for HP OMEN

`omen-ry` is an ultra-lightweight, memory-safe, and event-driven background daemon written in **Rust** designed to control and animate the 4-zone RGB keyboard on HP OMEN laptops.

---

## Key Highlights

- **Ultra-low footprint**: Consumes only **~3.5 MB RAM** (RSS) and compiles to a stripped **1.3 MB binary**.
- **Zero-cost quiescence**: The frame timer pauses automatically when static colors are displayed or animations finish, dropping to **0.0% CPU** while sleeping in Linux `epoll`.
- **Non-root execution**: Runs cleanly as a standard `systemd --user` service without requiring `sudo` or root privileges.
- **Instant IPC control**: Companion CLI tool communicating over a fast UNIX domain socket.
- **Hardware dirty-checking**: Avoids unnecessary kernel ACPI WMI transactions by deduplicating and batching zone updates.
- **Terminal preview mode**: Includes a built-in 24-bit ANSI truecolor visualizer (`[████][████][████][████]`) for testing animations in any shell.

---

## 1. Hardware Driver Setup (One-time)

To allow Linux userspace to communicate with the laptop's ACPI WMI keyboard backlight controller, install the `omen-rgb-keyboard` DKMS module and udev rules:

```bash
cd /tmp/omen-rgb-keyboard
sudo ./install.sh
sudo ./install-udev-rules.sh
```

> **Note**: `install-udev-rules.sh` adds your user to the `input` group so the daemon can write to the keyboard sysfs attributes without needing `sudo`. You can activate this group in your current session by running `newgrp input` or logging out and back in.

---

## 2. Quickstart & CLI Usage

`omen-ry` is installed to `~/.local/bin/omen-ry`.

### Check Daemon Status
```bash
omen-ry status
```

### Start Daemon Manually
```bash
# Production mode (detects physical keyboard sysfs)
omen-ry daemon

# Mock mode (prints terminal ANSI colors, for testing)
omen-ry daemon --mock
```

### Controlling Lighting via CLI

```bash
# 1. Activate Wave animation (palettes: cyberpunk, rainbow, sunset, matrix, ocean, fire)
omen-ry mode wave --palette cyberpunk --speed 0.5
omen-ry mode wave --palette sunset --speed 1.0 --reverse

# 2. Set static colors (All zones at once)
omen-ry color --all 00FFCC
omen-ry color --all FF0055

# 3. Set individual zones (Zone 0 = Left/WASD, Zone 3 = Numpad/Right)
omen-ry color --zone 0 --color FF0000
omen-ry color --zone 1 --color 00FF00
omen-ry color --zone 2 --color 0000FF
omen-ry color --zone 3 --color FFFFFF

# 4. Set 4 static zones at once
omen-ry mode static FF0055 00FFCC FFAA00 AA00FF

# 5. Adjust brightness (0 - 100%)
omen-ry brightness 80

# 6. Pause / Resume animation
omen-ry pause
omen-ry resume

# 7. Interactive terminal preview (independent of keyboard hardware)
omen-ry preview --palette rainbow --speed 1.0
```

---

## 3. Background Autostart (systemd user service)

To run `omen-ry` continuously in the background on login:

```bash
# Enable and start user service
systemctl --user enable --now omen-ry.service

# Check service status and logs
systemctl --user status omen-ry.service
journalctl --user -u omen-ry -f
```

---

## 4. Configuration

A configuration file is installed at `~/.config/omen-ry/config.toml`:

```toml
[daemon]
fps = 30
default_brightness = 100
default_mode = "wave"

[wave]
palette = "cyberpunk"
speed = 0.5
reverse = false
```

---

## 5. Adding New Animations in Rust

`omen-ry` is designed around a clean, extensible `Animation` trait (`src/animations/mod.rs`):

```rust
pub trait Animation: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_animating(&self) -> bool;
    fn tick(&mut self, dt: Duration) -> [Color; 4];
}
```

To create a new animation (e.g. breathing or ripple):
1. Create a struct implementing `Animation` in `src/animations/`.
2. In `tick(&mut self, dt: Duration) -> [Color; 4]`, calculate your 4 zone colors.
3. Wire the new mode into `src/engine/mod.rs` and `src/main.rs`.
4. Rebuild with `cargo build --release`!
