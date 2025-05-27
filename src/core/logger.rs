use crate::context;
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

static LOG_MUTEX: Mutex<()> = Mutex::new(());
static LOGGER_HANDLE: OnceLock<flexi_logger::LoggerHandle> = OnceLock::new();

const MAX_LOG_FILES: usize = 10;

/// Initialize the logger once
pub fn init_logger() -> io::Result<()> {
    let config = context::get_config();
    let config = config.lock().expect("Failed to lock config");
    let logs_folder = &config.logs_folder;

    // Ensure folders exist
    create_dir_all(logs_folder.join("main"))?;
    create_dir_all(logs_folder.join("ops"))?;

    let log_spec = LogSpecification::parse("info, file_ops=info")
        .map_err(io::Error::other)
        .expect("Failed to parse log specification");

    let logger = Logger::with(log_spec)
        .log_to_writer(Box::new(DualWriter::new(logs_folder)))
        .write_mode(WriteMode::BufferAndFlush)
        .format(custom_format)
        .start()
        .map_err(io::Error::other)
        .expect("Failed to start logger");

    LOGGER_HANDLE
        .set(logger)
        .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "Logger already initialized"))
        .expect("Logger handle already set");

    Ok(())
}

/// Logs a file operation using the `file_ops` target
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

/// Writer that routes logs based on target
struct DualWriter {
    main_dir: PathBuf,
    ops_dir: PathBuf,
}

impl DualWriter {
    fn new(base: &Path) -> Self {
        Self {
            main_dir: base.join("main"),
            ops_dir: base.join("ops"),
        }
    }

    fn get_log_path(&self, target: &str) -> PathBuf {
        let timestamp = Local::now().format("%d-%m-%Y").to_string();
        let dir = if target == "file_ops" {
            &self.ops_dir
        } else {
            &self.main_dir
        };
        dir.join(format!("{timestamp}.log"))
    }
}

impl LogWriter for DualWriter {
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

        let path = self.get_log_path(record.target());
        let dir = path.parent().unwrap();

        // Log rotation: keep only MAX_LOG_FILES most recent logs
        let mut log_files: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| io::Error::other(format!("Failed to read dir: {e}")))?
            .filter_map(|entry| {
                let entry = entry.expect("Failed to read directory entry");
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("log") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Sort by file name (date-based, so lexicographical sort works)
        log_files.sort();

        // Remove oldest files if exceeding MAX_LOG_FILES
        while log_files.len() >= MAX_LOG_FILES {
            if let Some(oldest) = log_files.first() {
                let _ = std::fs::remove_file(oldest);
                log_files.remove(0);
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| io::Error::other(format!("Failed to open log file: {e}")))
            .expect("Failed to open log file");

        let mut buf = Vec::new();
        // Use the custom_format function to format the log entry
        custom_format(&mut buf, now, record)
            .map_err(|e| io::Error::other(format!("Failed to format log entry: {e}")))
            .expect("Failed to format log entry");
        file.write_all(&buf)
            .map_err(|e| io::Error::other(format!("Failed to write to log file: {e}")))
            .expect("Failed to write to log file");
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
