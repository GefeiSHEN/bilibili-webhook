use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{ErrorKind, Write},
};

use log::error;

pub fn bilili(source: &str, output: &str) {
    let log_file = format!("config/log/{}.log", source);
    log::info!("Writing to log file: {}", log_file);

    match create_dir_all("config/log") {
        Ok(_) => log::info!("Ensured existence of log directory"),
        Err(e) => log::error!("Failed to create log directory: {:?}", e),
    };

    match OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(output.as_bytes()) {
                log::error!("Failed to write output to log file: {:?}", e);
            }
            if let Err(e) = file.write_all(b"\n") {
                log::error!("Failed to write newline to log file: {:?}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to open log file: {:?}", e);
        }
    }
}
