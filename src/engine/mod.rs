pub mod event_loop;
pub mod ipc_server;

use std::time::Duration;
use crate::animations::static_color::StaticColorAnimation;
use crate::animations::wave::WaveAnimation;
use crate::animations::Animation;
use crate::color::Color;
use crate::hal::ZoneBackend;
use crate::ipc::protocol::{DaemonStatus, IpcRequest, IpcResponse};

pub struct Engine {
    backend: Box<dyn ZoneBackend>,
    animation: Box<dyn Animation>,
    brightness: u8,
    fps: u32,
    paused: bool,
    current_colors: [Color; 4],
}

impl Engine {
    pub fn new(backend: Box<dyn ZoneBackend>, brightness: u8, fps: u32) -> Self {
        let mut engine = Self {
            backend,
            animation: Box::new(WaveAnimation::default()),
            brightness: brightness.min(100),
            fps: fps.clamp(1, 120),
            paused: false,
            current_colors: [Color::BLACK; 4],
        };
        let _ = engine.backend.set_brightness(engine.brightness);
        engine
    }

    pub fn is_animating(&self) -> bool {
        !self.paused && self.animation.is_animating()
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn render_frame(&mut self, dt: Duration) -> std::io::Result<()> {
        let raw_colors = self.animation.tick(dt);
        self.current_colors = raw_colors;

        let scaled_colors = [
            raw_colors[0].scale_brightness(self.brightness),
            raw_colors[1].scale_brightness(self.brightness),
            raw_colors[2].scale_brightness(self.brightness),
            raw_colors[3].scale_brightness(self.brightness),
        ];

        self.backend.set_zones(&scaled_colors)
    }

    pub fn handle_ipc(&mut self, req: IpcRequest) -> IpcResponse {
        match req {
            IpcRequest::GetStatus => IpcResponse::Status(self.get_status()),

            IpcRequest::SetModeWave { palette, speed, reverse } => {
                let p = palette.as_deref().unwrap_or("cyberpunk");
                let s = speed.unwrap_or(0.5);
                let r = reverse.unwrap_or(false);

                self.animation = Box::new(WaveAnimation::new(p, s, r));
                self.paused = false;
                IpcResponse::Ok {
                    message: format!("Wave animation activated (palette: {}, speed: {:.2}, reverse: {})", p, s, r),
                }
            }

            IpcRequest::SetModeStatic { colors } => {
                let mut parsed = [Color::BLACK; 4];
                for i in 0..4 {
                    match Color::from_hex(&colors[i]) {
                        Ok(c) => parsed[i] = c,
                        Err(e) => return IpcResponse::Error { message: e },
                    }
                }

                self.animation = Box::new(StaticColorAnimation::new(parsed));
                self.paused = false;
                IpcResponse::Ok {
                    message: format!("Static colors set: {}", parsed.map(|c| c.to_hex_hash()).join(", ")),
                }
            }

            IpcRequest::SetColorAll { color } => {
                match Color::from_hex(&color) {
                    Ok(c) => {
                        self.animation = Box::new(StaticColorAnimation::solid(c));
                        self.paused = false;
                        IpcResponse::Ok {
                            message: format!("All zones set to {}", c.to_hex_hash()),
                        }
                    }
                    Err(e) => IpcResponse::Error { message: e },
                }
            }

            IpcRequest::SetColorZone { zone, color } => {
                if zone >= 4 {
                    return IpcResponse::Error {
                        message: format!("Zone index {} out of range (must be 0..3)", zone),
                    };
                }

                match Color::from_hex(&color) {
                    Ok(c) => {
                        let mut colors = self.current_colors;
                        colors[zone] = c;
                        self.animation = Box::new(StaticColorAnimation::new(colors));
                        self.paused = false;
                        IpcResponse::Ok {
                            message: format!("Zone {} set to {}", zone, c.to_hex_hash()),
                        }
                    }
                    Err(e) => IpcResponse::Error { message: e },
                }
            }

            IpcRequest::SetBrightness { brightness } => {
                let b = brightness.min(100);
                self.brightness = b;
                let _ = self.backend.set_brightness(b);
                IpcResponse::Ok {
                    message: format!("Brightness set to {}%", b),
                }
            }

            IpcRequest::Pause => {
                self.paused = true;
                IpcResponse::Ok {
                    message: "Animation paused".to_string(),
                }
            }

            IpcRequest::Resume => {
                self.paused = false;
                IpcResponse::Ok {
                    message: "Animation resumed".to_string(),
                }
            }

            IpcRequest::Shutdown => {
                IpcResponse::Ok {
                    message: "Shutting down daemon...".to_string(),
                }
            }
        }
    }

    fn get_status(&self) -> DaemonStatus {
        DaemonStatus {
            mode: self.animation.name().to_string(),
            brightness: self.brightness,
            fps: self.fps,
            backend: self.backend.backend_name().to_string(),
            current_colors: [
                self.current_colors[0].to_hex_hash(),
                self.current_colors[1].to_hex_hash(),
                self.current_colors[2].to_hex_hash(),
                self.current_colors[3].to_hex_hash(),
            ],
            is_animating: self.is_animating(),
        }
    }
}
