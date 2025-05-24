use std::sync::Mutex;
use once_cell::sync::OnceCell;

pub static CONFIG_VERSION: usize = 0;
pub static CONFIG_FILE_NAME: &str = "tooka.yaml";
pub static RULES_FILE_NAME: &str = "rules.yaml";
pub static DEFAULT_LOGS_FOLDER: &str = "logs";
pub const APP_QUALIFIER: &str = "io";
pub const APP_ORG: &str = "github.benji377";
pub const APP_NAME: &str = "tooka";

pub static MAIN_LOGGER_HANDLE: OnceCell<flexi_logger::LoggerHandle> = OnceCell::new();
pub static OPS_LOGGER_HANDLE: OnceCell<Mutex<Option<flexi_logger::LoggerHandle>>> = OnceCell::new();