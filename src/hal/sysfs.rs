use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use crate::color::Color;
use super::ZoneBackend;

pub const DEFAULT_SYSFS_PATH: &str = "/sys/devices/platform/omen-rgb-keyboard/rgb_zones";

pub struct SysfsBackend {
    zone_files: [File; 4],
    all_file: Option<File>,
    brightness_file: Option<File>,
    cached_colors: [Color; 4],
    cached_brightness: Option<u8>,
}

impl SysfsBackend {
    pub fn is_available() -> bool {
        Path::new(DEFAULT_SYSFS_PATH).join("zone00").exists()
    }

    pub fn new() -> io::Result<Self> {
        Self::with_base_path(DEFAULT_SYSFS_PATH)
    }

    pub fn with_base_path<P: AsRef<Path>>(base: P) -> io::Result<Self> {
        let base = base.as_ref();

        let open_write = |name: &str| -> io::Result<File> {
            let path: PathBuf = base.join(name);
            OpenOptions::new()
                .read(false)
                .write(true)
                .open(&path)
                .map_err(|e| {
                    io::Error::new(
                        e.kind(),
                        format!("Failed to open sysfs attribute {}: {}", path.display(), e),
                    )
                })
        };

        let zone_files = [
            open_write("zone00")?,
            open_write("zone01")?,
            open_write("zone02")?,
            open_write("zone03")?,
        ];

        let all_file = open_write("all").ok();
        let brightness_file = open_write("brightness").ok();

        Ok(Self {
            zone_files,
            all_file,
            brightness_file,
            cached_colors: [Color::BLACK; 4],
            cached_brightness: None,
        })
    }
}

impl ZoneBackend for SysfsBackend {
    fn set_zones(&mut self, colors: &[Color; 4]) -> io::Result<()> {
        // Optimization: If all 4 zones are identical and 'all' file is available, write once
        let all_identical = colors[0] == colors[1] && colors[1] == colors[2] && colors[2] == colors[3];
        if all_identical {
            if let Some(ref mut all) = self.all_file {
                if self.cached_colors != *colors {
                    let hex = format!("{}\n", colors[0].to_hex());
                    all.seek(SeekFrom::Start(0))?;
                    all.write_all(hex.as_bytes())?;
                    all.flush()?;
                    self.cached_colors = *colors;
                }
                return Ok(());
            }
        }

        // Zone-by-zone dirty checking to eliminate redundant kernel ACPI calls
        for (i, zone_file) in self.zone_files.iter_mut().enumerate() {
            if colors[i] != self.cached_colors[i] {
                let hex = format!("{}\n", colors[i].to_hex());
                zone_file.seek(SeekFrom::Start(0))?;
                zone_file.write_all(hex.as_bytes())?;
                zone_file.flush()?;
                self.cached_colors[i] = colors[i];
            }
        }

        Ok(())
    }

    fn set_brightness(&mut self, brightness: u8) -> io::Result<()> {
        let b = brightness.min(100);
        if self.cached_brightness == Some(b) {
            return Ok(());
        }

        if let Some(ref mut file) = self.brightness_file {
            let s = format!("{}\n", b);
            file.seek(SeekFrom::Start(0))?;
            file.write_all(s.as_bytes())?;
            file.flush()?;
            self.cached_brightness = Some(b);
        }
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "Linux sysfs (omen-rgb-keyboard)"
    }
}
