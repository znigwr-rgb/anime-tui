use std::process::{Command, Stdio};

pub struct Download {
    pub path: String,
}

pub fn start_download(magnet: &str) -> Result<Download, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let dir = format!("{}/Descargas/anime", home);

    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;

    let mut cmd = Command::new("aria2c");
    cmd.arg("--dir")
        .arg(&dir)
        .arg("--seed-time=0")
        .arg("--max-connection-per-server=16")
        .arg("--split=16")
        .arg("--console-log-level=error")
        .arg("--summary-interval=0")
        .arg(magnet)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if cmd.spawn().is_ok() {
        Ok(Download { path: dir })
    } else {
        open_magnet(magnet)?;
        Ok(Download { path: dir })
    }
}

fn open_magnet(magnet: &str) -> Result<(), String> {
    let status = Command::new("xdg-open")
        .arg(magnet)
        .status()
        .map_err(|e| format!("Failed to open magnet: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("xdg-open failed".to_string())
    }
}
