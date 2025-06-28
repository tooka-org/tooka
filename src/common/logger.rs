//! Logging utilities for the Tooka application.
//!
//! This module provides a custom logger setup using `flexi_logger`.
//! It supports separate log files for general logs and file operation logs,
//! with daily log rotation and a maximum number of retained log files.

use crate::{core::context, core::error::TookaError};
use chrono::Local;
use flexi_logger::writers::LogWriter;
use flexi_logger::{LogSpecification, Logger, Record, WriteMode};
use log::Record as LogRecord;
use std::path::Path;
use std::{
    fs::{OpenOptions, create_dir_all},
    io::{self, Write},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

/// Mutex to ensure thread-safe logging
static LOG_MUTEX: Mutex<()> = Mutex::new(());
/// Static logger handle to ensure logger is initialized only once
static LOGGER_HANDLE: OnceLock<flexi_logger::LoggerHandle> = OnceLock::new();
/// Maximum number of log files to keep
const MAX_LOG_FILES: usize = 10;

/// Writer that routes logs based on target
struct DualWriter {
    /// Directory for main logs
    main_dir: PathBuf,
    /// Directory for file operation logs
    ops_dir: PathBuf,
}

/// Initializes the Tooka logger.
///
/// Sets up logging directories, configures log levels and targets,
/// and ensures that the logger is only initialized once.
///
/// # Errors
/// Returns a [`TookaError`] if initialization fails or config cannot be loaded.
pub fn init_logger() -> Result<(), TookaError> {
    let config = context::get_locked_config()
        .map_err(|e| TookaError::ConfigError(format!("Failed to get config: {e}")))?;
    let logs_folder = &config.logs_folder;

    // Ensure folders exist
    create_dir_all(logs_folder.join("ops"))?;

    let log_spec = LogSpecification::parse("debug, file_ops=info")?;

    let logger = Logger::with(log_spec)
        .log_to_writer(Box::new(DualWriter::new(logs_folder)))
        .write_mode(WriteMode::BufferAndFlush)
        .format(custom_format)
        .start()?;

    LOGGER_HANDLE
        .set(logger)
        .map_err(|_| TookaError::ConfigError("Logger already initialized".into()))?;

    Ok(())
}

/// Logs a file operation message.
///
/// Sends the message to the `file_ops` log target for separate logging.
pub fn log_file_operation(msg: &str) {
    log::info!(target: "file_ops", "{msg}");
}

/// Custom formatter
fn custom_format(
    w: &mut dyn Write,
    now: &mut flexi_logger::DeferredNow,
    record: &LogRecord,
) -> io::Result<()> {
    writeln!(
        w,
        "{} [{}] {} - {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        record.target(),
        record.args()
    )
}

/// Implementation of the DualWriter
impl DualWriter {
    /// Creates a new DualWriter with the specified base path
    fn new(base: &Path) -> Self {
        Self {
            main_dir: base.to_path_buf(),
            ops_dir: base.join("ops"),
        }
    }

    // Main log path: just main.log at base folder
    fn get_main_log_path(&self) -> PathBuf {
        self.main_dir.join("main.log")
    }

    // Ops log path: figure out today's file, add -1, -2 if needed
    fn get_ops_log_path(&self) -> std::io::Result<PathBuf> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let base_path = self.ops_dir.join(format!("{date_str}.log"));

        // If base_path does not exist, use it directly
        if !base_path.exists() {
            return Ok(base_path);
        }

        // Otherwise, check for -1, -2, ... suffixes, find latest file
        for i in 1..=MAX_LOG_FILES {
            let candidate = self.ops_dir.join(format!("{date_str}-{i}.log"));
            if !candidate.exists() {
                // Use the first non-existent file
                return Ok(candidate);
            }
        }

        // If all numbered files exist, just return the last one
        Ok(self.ops_dir.join(format!("{date_str}-{MAX_LOG_FILES}.log")))
    }

    // Helper: check if file modified less than 1 hour ago
    fn is_file_recent(path: &Path) -> std::io::Result<bool> {
        if !path.exists() {
            return Ok(false);
        }
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        let age = Local::now().signed_duration_since(chrono::DateTime::<Local>::from(modified));
        Ok(age.num_minutes() < 60)
    }
}

/// Implementation of the LogWriter trait for DualWriter
impl LogWriter for DualWriter {
    /// Writes a log record to the appropriate file based on its target
    fn write(
        &self,
        now: &mut flexi_logger::DeferredNow,
        record: &Record,
    ) -> Result<(), std::io::Error> {
        // Try to acquire the lock, but avoid indefinite waiting

        let Ok(_guard) = LOG_MUTEX.try_lock() else {
            // If the lock is poisoned or already held, skip logging to avoid deadlock
            return Ok(());
        };

        if record.target() == "file_ops" {
            // Ops logger: use numbered daily file
            let path = self.get_ops_log_path()?;
            // Rotate ops logs: keep only MAX_LOG_FILES newest files
            let dir = path.parent().unwrap();

            let mut log_files: Vec<_> = std::fs::read_dir(dir)?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("log") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            log_files.sort();

            while log_files.len() > MAX_LOG_FILES {
                if let Some(oldest) = log_files.first() {
                    let _ = std::fs::remove_file(oldest);
                    log_files.remove(0);
                }
            }

            let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

            let mut buf = Vec::new();
            custom_format(&mut buf, now, record)?;
            file.write_all(&buf)?;
        } else {
            // Main logger
            let path = self.get_main_log_path();

            let recent = Self::is_file_recent(&path)?;
            let mut open_opts = OpenOptions::new();
            open_opts.create(true);
            if recent {
                // append
                open_opts.append(true);
            } else {
                // overwrite
                open_opts.write(true).truncate(true);
            }

            let mut file = open_opts.open(&path)?;
            let mut buf = Vec::new();
            custom_format(&mut buf, now, record)?;
            file.write_all(&buf)?;
        }
        Ok(())
    }

    /// Flushes the log writer
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
