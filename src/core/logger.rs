use crate::core::config;
use chrono::Local;
use flexi_logger::{
    DeferredNow, Duplicate, Logger, WriteMode,
};
use lazy_static::lazy_static;
use log::Record;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::Mutex,
};

const MAX_LOG_FILES: usize = 10;

lazy_static! {
    static ref MAIN_LOG: Mutex<()> = Mutex::new(());
    static ref OPS_LOG: Mutex<()> = Mutex::new(());
}

/// Initializes the single global logger instance
pub fn init_main_logger() -> io::Result<()> {
    let config = config::Config::load()?;
    let logs_folder = config.logs_folder.clone();

    // Create log folders
    fs::create_dir_all(logs_folder.join("main"))?;
    fs::create_dir_all(logs_folder.join("ops"))?;

    // Start logger (only ONE allowed in the app)
    Logger::try_with_str("info, file_ops=info")
        .map_err(|e| io::Error::other(format!("Logger error: {e}")))?
        .duplicate_to_stdout(Duplicate::Info)
        .write_mode(WriteMode::BufferAndFlush)
        .format(custom_format)
        .start()
        .map_err(|e| io::Error::other(format!("Logger error: {e}")))?;

    Ok(())
}

/// Logs a file operation to `file_ops` target
pub fn log_file_operation(msg: &str) {
    log::info!(target: "file_ops", "{}", msg);
}

/// Custom log formatter that writes to separate files by target
fn custom_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record,
) -> io::Result<()> {
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");
    let log_line = format!(
        "{} [{}] {} - {}\n",
        timestamp,
        record.level(),
        record.target(),
        record.args()
    );

    // Always print to stdout/stderr as usual
    write!(w, "{}", log_line)?;

    // Determine where to write based on log target
    let config = config::Config::load().map_err(|e| {
        io::Error::other(format!("Failed to load config in logger: {e}"))
    })?;

    let (subdir, mutex): (&str, &Mutex<()>) = match record.target() {
        "file_ops" => ("ops", &OPS_LOG as &Mutex<()>),
        _ => ("main", &MAIN_LOG as &Mutex<()>),
    };

    let folder = config.logs_folder.join(subdir);
    let filename = formatted_filename();
    let filepath = folder.join(filename);

    // Rotate log files in that folder
    rotate_logs(&folder, MAX_LOG_FILES)?;

    let _guard = mutex.lock().unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)?;

    file.write_all(log_line.as_bytes())?;
    Ok(())
}

/// Generate filename: "dd-mm-yyyy_HH-MM.log"
fn formatted_filename() -> String {
    Local::now().format("%d-%m-%Y_%H-%M.log").to_string()
}

/// Keeps only the N most recent log files in a folder
fn rotate_logs(folder: &Path, keep: usize) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(folder)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .collect();

    // Sort oldest first
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    while entries.len() > keep {
        if let Some(entry) = entries.first() {
            let _ = fs::remove_file(entry.path());
        }
        entries.remove(0);
    }

    Ok(())
}
