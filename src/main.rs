use omen_ry::{animations, cli, config, engine, hal, ipc};

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use animations::wave::WaveAnimation;
use animations::Animation;
use cli::client::{print_status, send_request};
use config::DaemonConfig;
use engine::event_loop::run_event_loop;
use engine::ipc_server::IpcServer;
use engine::Engine;
use hal::mock::MockBackend;
use hal::sysfs::SysfsBackend;
use hal::ZoneBackend;
use ipc::get_socket_path;
use ipc::protocol::{IpcRequest, IpcResponse};

#[derive(Parser)]
#[command(name = "omen-ry")]
#[command(author = "bearcut")]
#[command(version = "0.1.0")]
#[command(about = "Ultra-lightweight event-driven 4-zone RGB daemon for HP OMEN laptops", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Daemon(DaemonArgs),

    /// Query the status of the running daemon
    Status,

    /// Change animation modes
    Mode(ModeArgs),

    /// Set static colors on zones
    Color(ColorArgs),

    /// Set keyboard brightness (0 - 100)
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
    },

    /// Pause active animation
    Pause,

    /// Resume paused animation
    Resume,

    /// Interactive terminal ANSI color preview without affecting keyboard hardware
    Preview(PreviewArgs),
}

#[derive(Args)]
struct DaemonArgs {
    /// Force mock/terminal visualizer instead of hardware sysfs
    #[arg(long)]
    mock: bool,

    /// Custom config file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Override target FPS
    #[arg(long)]
    fps: Option<u32>,

    /// Override initial brightness
    #[arg(short, long)]
    brightness: Option<u8>,
}

#[derive(Args)]
struct ModeArgs {
    #[command(subcommand)]
    subcommand: ModeSubcommands,
}

#[derive(Subcommand)]
enum ModeSubcommands {
    /// Smooth directional wave across keyboard zones
    Wave {
        /// Color palette (cyberpunk, rainbow, sunset, matrix, ocean, fire)
        #[arg(short, long)]
        palette: Option<String>,

        /// Animation speed in cycles per second (default: 0.5)
        #[arg(short, long)]
        speed: Option<f32>,

        /// Flow wave right-to-left instead of left-to-right
        #[arg(short, long)]
        reverse: bool,
    },

    /// Static 4-zone colors
    Static {
        /// 4 Hex colors for zones 0, 1, 2, 3 (e.g. FF0000 00FF00 0000FF FFFFFF)
        #[arg(num_args = 4, required = true)]
        colors: Vec<String>,
    },
}

#[derive(Args)]
struct ColorArgs {
    /// Set all 4 zones to the same hex color (e.g. FF0055)
    #[arg(long, conflicts_with_all = ["zone", "color"])]
    all: Option<String>,

    /// Specific zone index (0 to 3)
    #[arg(short, long, requires = "color")]
    zone: Option<usize>,

    /// Hex color value (e.g. 00FFCC)
    #[arg(short, long, requires = "zone")]
    color: Option<String>,
}

#[derive(Args)]
struct PreviewArgs {
    /// Palette to preview
    #[arg(short, long, default_value = "cyberpunk")]
    palette: String,

    /// Speed multiplier
    #[arg(short, long, default_value_t = 0.5)]
    speed: f32,

    /// Flow right-to-left
    #[arg(short, long)]
    reverse: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let socket_path = get_socket_path();

    match cli.command {
        Commands::Daemon(args) => {
            let subscriber = FmtSubscriber::builder()
                .with_max_level(Level::INFO)
                .with_target(false)
                .without_time()
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;

            let cfg = DaemonConfig::load_or_default(args.config.as_deref());
            let fps = args.fps.unwrap_or(cfg.fps);
            let brightness = args.brightness.unwrap_or(cfg.default_brightness);

            let backend: Box<dyn ZoneBackend> = if args.mock {
                info!("Starting in mock mode (terminal visualizer)");
                Box::new(MockBackend::new(true))
            } else if SysfsBackend::is_available() {
                info!("HP Omen sysfs interface found");
                match SysfsBackend::new() {
                    Ok(b) => Box::new(b),
                    Err(e) => {
                        error!("Failed to initialize sysfs backend (insufficient permissions?): {}", e);
                        info!("Falling back to mock backend");
                        Box::new(MockBackend::new(true))
                    }
                }
            } else {
                info!("HP Omen sysfs interface not loaded. Running with mock visualizer.");
                info!("(Install the omen-rgb-keyboard driver to control physical hardware)");
                Box::new(MockBackend::new(true))
            };

            let (ipc_tx, ipc_rx) = tokio::sync::mpsc::channel(32);
            let ipc_server = IpcServer::bind(&socket_path)?;
            tokio::spawn(async move {
                ipc_server.run(ipc_tx).await;
            });

            let mut engine = Engine::new(backend, brightness, fps);

            // Configure initial mode from config
            if cfg.default_mode == "wave" {
                let _ = engine.handle_ipc(IpcRequest::SetModeWave {
                    palette: Some(cfg.wave.palette),
                    speed: Some(cfg.wave.speed),
                    reverse: Some(cfg.wave.reverse),
                });
            }

            info!("omen-ry daemon is ready!");
            run_event_loop(engine, ipc_rx).await?;

            // Cleanup socket on exit
            if socket_path.exists() {
                let _ = std::fs::remove_file(&socket_path);
            }
        }

        Commands::Status => {
            match send_request(&socket_path, IpcRequest::GetStatus).await {
                Ok(IpcResponse::Status(status)) => print_status(&status),
                Ok(other) => println!("{:?}", other),
                Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
            }
        }

        Commands::Mode(args) => match args.subcommand {
            ModeSubcommands::Wave { palette, speed, reverse } => {
                let req = IpcRequest::SetModeWave { palette, speed, reverse: Some(reverse) };
                match send_request(&socket_path, req).await {
                    Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                    Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                    Ok(_) => {}
                    Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
                }
            }
            ModeSubcommands::Static { colors } => {
                let req = IpcRequest::SetModeStatic {
                    colors: [colors[0].clone(), colors[1].clone(), colors[2].clone(), colors[3].clone()],
                };
                match send_request(&socket_path, req).await {
                    Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                    Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                    Ok(_) => {}
                    Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
                }
            }
        },

        Commands::Color(args) => {
            let req = if let Some(all) = args.all {
                IpcRequest::SetColorAll { color: all }
            } else if let (Some(zone), Some(color)) = (args.zone, args.color) {
                IpcRequest::SetColorZone { zone, color }
            } else {
                eprintln!("\x1b[31mError:\x1b[0m Provide either --all <HEX> or --zone <IDX> --color <HEX>");
                return Ok(());
            };

            match send_request(&socket_path, req).await {
                Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                Ok(_) => {}
                Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
            }
        }

        Commands::Brightness { level } => {
            match send_request(&socket_path, IpcRequest::SetBrightness { brightness: level }).await {
                Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                Ok(_) => {}
                Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
            }
        }

        Commands::Pause => {
            match send_request(&socket_path, IpcRequest::Pause).await {
                Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                Ok(_) => {}
                Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
            }
        }

        Commands::Resume => {
            match send_request(&socket_path, IpcRequest::Resume).await {
                Ok(IpcResponse::Ok { message }) => println!("\x1b[32m✓\x1b[0m {}", message),
                Ok(IpcResponse::Error { message }) => eprintln!("\x1b[31mError:\x1b[0m {}", message),
                Ok(_) => {}
                Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
            }
        }

        Commands::Preview(args) => {
            println!("\x1b[1mTerminal Wave Preview (Press Ctrl+C to exit)\x1b[0m");
            println!("Palette: {}, Speed: {:.2}, Reverse: {}", args.palette, args.speed, args.reverse);
            let mut anim = WaveAnimation::new(&args.palette, args.speed, args.reverse);
            let mut backend = MockBackend::new(true);
            let dt = Duration::from_millis(33); // ~30 FPS

            loop {
                let colors = anim.tick(dt);
                let _ = backend.set_zones(&colors);
                tokio::time::sleep(dt).await;
            }
        }
    }

    Ok(())
}
