use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ErrorLogConfig {
    pub log_dir: PathBuf,
    pub max_file_size: u64,
}

impl Default for ErrorLogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            max_file_size: 5 * 1024 * 1024,
        }
    }
}

pub struct ErrorLogWriter {
    log_dir: PathBuf,
    max_file_size: u64,
}

impl ErrorLogWriter {
    pub fn new(config: &ErrorLogConfig) -> Self {
        let _ = fs::create_dir_all(&config.log_dir);
        Self {
            log_dir: config.log_dir.clone(),
            max_file_size: config.max_file_size,
        }
    }

    fn get_current_file(&self) -> PathBuf {
        self.log_dir.join("error.log")
    }

    fn rotate_if_needed(&self) {
        let error_path = self.get_current_file();
        if !error_path.exists() {
            return;
        }
        if let Ok(metadata) = fs::metadata(&error_path) {
            if metadata.len() >= self.max_file_size {
                self.rotate_logs();
            }
        }
    }

    fn rotate_logs(&self) {
        for i in (1..=9).rev() {
            let src = self.log_dir.join(format!("error.{}.log", i));
            let dst = self.log_dir.join(format!("error.{}.log", i + 1));
            if src.exists() {
                let _ = fs::rename(src, dst);
            }
        }
        let current = self.get_current_file();
        let archived = self.log_dir.join("error.1.log");
        if current.exists() {
            let _ = fs::rename(current, archived);
        }
    }
}

impl Write for ErrorLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.rotate_if_needed();

        let error_path = self.get_current_file();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_path)?;

        file.write(buf)?;
        file.flush()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn get_error_log_paths(log_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(error_log) = log_dir.join("error.log").exists().then(|| log_dir.join("error.log"))
    {
        paths.push(error_log);
    }

    for i in 1..=10 {
        let rotated = log_dir.join(format!("error.{}.log", i));
        if rotated.exists() {
            paths.push(rotated);
        }
    }

    paths.sort_by(|a, b| b.cmp(a));
    paths
}
