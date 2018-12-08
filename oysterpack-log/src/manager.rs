// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module is the anchor point for configuring and initializing the [log](https://crates.io/crates/log) system.

use crate::config::{LogConfig, Target};
use fern::Output;
use log::Record;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

const LOG_NOT_INITIALIZED: usize = 0;
const LOG_INITIALIZING: usize = 1;
const LOG_INITIALIZED: usize = 2;
// LOG_STATE transitions: LOG_NOT_INITIALIZED -> LOG_INITIALIZING -> LOG_INITIALIZED
static LOG_STATE: AtomicUsize = ATOMIC_USIZE_INIT;

static mut LOG_CONFIG: Option<LogConfig> = None;

/// Initializes the logging system
/// - if the logging system is already initialized (or initializing), then a warning log message is logged.
/// - the build is provided to get the crate's package name. This is used to configure the crate log level.
///   - Build is used instead of the more specific PackageId because using Build ensures that the crate's
///     package name was obtained during build time. Otherwise, it's too easy to "hardcode" the crate package
///     name
pub fn init<F: RecordLogger>(config: LogConfig, logger: F) {
    match LOG_STATE.compare_and_swap(LOG_NOT_INITIALIZED, LOG_INITIALIZING, Ordering::SeqCst) {
        LOG_NOT_INITIALIZED => {
            let mut dispatch = fern::Dispatch::new().level(config.root_level().to_level_filter());

            if let Some(crate_log_level) = config.crate_level() {
                let crate_name = Target::for_crate().to_string();
                dispatch = dispatch.level_for(crate_name, crate_log_level.to_level_filter());
            }

            if let Some(target_levels) = config.target_levels() {
                for (target, level) in target_levels {
                    dispatch =
                        dispatch.level_for(target.as_ref().to_string(), level.to_level_filter());
                }
            }

            let _ = dispatch
                .chain(Output::call(move |record| logger.log(record)))
                .apply();

            let config_json = serde_json::to_string_pretty(&config).unwrap();
            unsafe { LOG_CONFIG = Some(config) }
            LOG_STATE.swap(LOG_INITIALIZED, Ordering::SeqCst);
            info!("logging has been initialized using config: {}", config_json);
        }
        LOG_INITIALIZING => {
            warn!("logging is being initialized ...");
            while LOG_STATE.load(Ordering::SeqCst) != LOG_INITIALIZED {
                std::thread::yield_now();
            }
        }
        _ => warn!("logging has already been initialized"),
    }
}

/// Logs the record
pub trait RecordLogger: Send + Sync + 'static {
    /// log the record
    fn log(&self, record: &Record);
}

/// Formats the record using the following format:
/// `[UTC_TIMESTAMP_rfc3339][LEVEL][TARGET][MODULE_PATH:LINE] MESSAGE`
///
///  For example:
/// `[2018-11-23T17:06:46.543Z][INFO][oysterpack_log::manager][oysterpack_log::manager:70] logging has been initialized`
pub fn format(record: &Record) -> String {
    if let (Some(module_path), Some(line)) = (record.module_path(), record.line()) {
        format!(
            "[{}][{}][{}][{}:{}]\n{}",
            record.level(),
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            record.target(),
            module_path,
            line,
            record.args()
        )
    } else {
        format!(
            "[{}][{}][{}]\n{}",
            record.level(),
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            record.target(),
            record.args()
        )
    }
}
/// logs the record to stdout
#[derive(Debug)]
pub struct StdoutLogger;

impl RecordLogger for StdoutLogger {
    fn log(&self, record: &Record) {
        println!("{}", format(record))
    }
}

/// logs the record to stderr
#[derive(Debug)]
pub struct StderrLogger;

impl RecordLogger for StderrLogger {
    fn log(&self, record: &Record) {
        eprintln!("{}", format(record))
    }
}

/// Returns the LogConfig that was used to initialize the log system.
/// If the logging system is not yet initialized, then None is returned.
pub fn config() -> Option<&'static LogConfig> {
    unsafe { LOG_CONFIG.as_ref() }
}
