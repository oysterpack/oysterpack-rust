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

use oysterpack_app_metadata::Build;
use fern::{Dispatch, Output};
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use LogConfig;
use config::LogOutput;

const LOG_NOT_INITIALIZED: usize = 0;
const LOG_INITIALIZING: usize = 1;
const LOG_INITIALIZED: usize = 2;
// LOG_STATE transitions: LOG_NOT_INITIALIZED -> LOG_INITIALIZING -> LOG_INITIALIZED
static LOG_STATE: AtomicUsize = ATOMIC_USIZE_INIT;

static mut LOG_CONFIG: Option<LogConfig> = None;

/// Initializes the logging system
/// - if the logging system is already initialized (or initializing), then
pub fn init(config: LogConfig, build: &Build) {
    match LOG_STATE.compare_and_swap(LOG_NOT_INITIALIZED, LOG_INITIALIZING, Ordering::SeqCst) {
        LOG_NOT_INITIALIZED => {
            let mut dispatch = fern::Dispatch::new()
                .level(config.root_level().to_level_filter());

            if let Some(crate_log_level) = config.crate_level() {
                let crate_name = build.package().id().name().to_string();
                dispatch = dispatch.level_for(crate_name, crate_log_level.to_level_filter());
            }

            if let Some(target_levels) = config.target_levels() {
                for (target,level) in target_levels {
                    dispatch = dispatch.level_for(target.as_ref().to_string(), level.to_level_filter());
                }
            }

            dispatch = configure_output(&config,dispatch);

            dispatch.apply().unwrap();
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            unsafe {
                LOG_CONFIG = Some(config)
            }
            LOG_STATE.swap(LOG_INITIALIZED, Ordering::SeqCst);
            info!("logging has been initialized using config: {}", config_json);
        }
        LOG_INITIALIZING => {
            warn!("logging is being initialized ...");
            while LOG_STATE.load(Ordering::SeqCst) != LOG_INITIALIZED {
                std::thread::yield_now();
            }
        },
        _ => warn!("logging has already been initialized"),
    }
}

fn configure_output(config: &LogConfig, dispatch: Dispatch) -> fern::Dispatch {
    match config.output() {
        LogOutput::Stdout(line_sep) => {
            configure_console_format(dispatch.chain(Output::stdout(line_sep.as_ref().to_string())))
        },
        LogOutput::Stderr(line_sep) => {
            configure_console_format(dispatch.chain(Output::stderr(line_sep.as_ref().to_string())))
        },
    }
}

fn configure_console_format(dispatch: Dispatch) -> Dispatch {
    dispatch.format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}][{}:{}] {}",
            chrono::Local::now().format("[%H:%M:%S%.3f]"),
            record.level(),
            record.target(),
            record.file().unwrap(),
            record.line().unwrap(),
            message
        ))
    })
}

/// Shutdown the logging system.
/// This should be called on application shutdown.
pub fn shutdown() {}

/// Returns the LogConfig used to initialize the log system.
pub fn config() -> Option<&'static LogConfig> {
    unsafe {
        LOG_CONFIG.as_ref()
    }
}
