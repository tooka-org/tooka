use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, Naming, WriteMode,
};
use log::Record;
use crate::core::config;

use std::{
    fs,
    io,
    sync::Mutex
};

use crate::globals::{MAIN_LOGGER_HANDLE, OPS_LOGGER_HANDLE};

// Static global logger handles


/// Initialize the main logger with daily rotation, overwrites each day
pub fn init_main_logger() -> io::Result<()> {
    let logs_folder = config::Config::load()?.logs_folder;

    let main_log_dir = logs_folder.join("main");
    fs::create_dir_all(&main_log_dir)?;

    let logger = Logger::try_with_str("info")
        .map_err(map_flexi_err)?
        .log_to_file(FileSpec::default().directory(main_log_dir).basename("main_log"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(1),
        )
        .duplicate_to_stdout(Duplicate::Info)
        .write_mode(WriteMode::BufferAndFlush)
        .format(custom_format)
        .start()
        .map_err(map_flexi_err)?;

    MAIN_LOGGER_HANDLE
        .set(logger)
        .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "Main logger already initialized"))?;

    Ok(())
}

/// Initialize the ops logger with a timestamped basename including hour & minute
pub fn init_ops_logger() -> io::Result<()> {
    let logs_folder = config::Config::load()?.logs_folder;
    let ops_log_dir = logs_folder.join("ops");
    fs::create_dir_all(&ops_log_dir)?;

    let time_str = formatted_datetime()?;

    let logger = Logger::try_with_str("info")
        .map_err(map_flexi_err)?
        .log_to_file(
            FileSpec::default()
                .directory(ops_log_dir)
                .basename(&time_str) // e.g., "24-05-2025_14-30"
                .suppress_timestamp(),
        )
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(10),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .format(custom_format)
        .start()
        .map_err(map_flexi_err)?;

    OPS_LOGGER_HANDLE
        .set(Mutex::new(Some(logger)))
        .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "Ops logger already initialized"))?;

    Ok(())
}

/// Logs a file operation, if the ops logger is initialized
pub fn log_file_operation(msg: &str) {
    if let Some(mutex_logger) = OPS_LOGGER_HANDLE.get() {
        let guard = mutex_logger.lock().unwrap();
        if guard.is_some() {
            log::info!(target: "file_ops", "{}", msg);
        }
    }
}

/// Formats log output consistently
fn custom_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    writeln!(
        w,
        "{} [{}] {} - {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        record.target(),
        &record.args()
    )
}

/// Helper: Returns current datetime as "dd-mm-yyyy_HH-MM"
fn formatted_datetime() -> io::Result<String> {
    let now = chrono::Local::now();
    Ok(now.format("%d-%m-%Y_%H-%M").to_string())
}

/// Helper: Converts `flexi_logger::FlexiLoggerError` to `std::io::Error`
fn map_flexi_err(err: flexi_logger::FlexiLoggerError) -> io::Error {
    io::Error::other(format!("Logger error: {err}"))
}
