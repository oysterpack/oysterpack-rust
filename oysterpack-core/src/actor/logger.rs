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

//! Actor Logging Service for async logging.

use actix::prelude::*;
use actor::{self, events, DisplayName, GetServiceInfo, Service, ServiceInfo};
use chrono::prelude::*;
use futures::{prelude::*, sync::oneshot};
use oysterpack_events::{event::ModuleSource, Eventful};
use oysterpack_log::{
    log::{Level, Record},
    manager::RecordLogger,
};
use std::fmt;

/// Logger ServiceId (01CWXSW61VREYK48PZ7QE549G5)
pub const SERVICE_ID: actor::Id = actor::Id(1865243759930187031543830440471307781);

/// Logger Actor.
/// - should run in its own dedicated Arbiter, i.e., thread.
/// - logs to stderr (for now - long term we need remote centralized logging)
#[derive(Debug)]
pub struct Logger {
    service_info: ServiceInfo,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID, Self::TYPE),
        }
    }
}

op_actor_service! {
  Service(Logger)
}

impl crate::actor::LifeCycle for Logger {}

impl DisplayName for Logger {
    fn name() -> &'static str {
        "Logger"
    }
}

/// LogRecord is a threadsafe version of log::Record, i.e., it implements Send + Sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    target: LogTarget,
    level: Level,
    msg: LogMessage,
    module_source: Option<ModuleSource>,
}

/// Log target
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogTarget(String);

impl fmt::Display for LogTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

/// Log message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogMessage(String);

impl fmt::Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl<'a> From<&'a Record<'a>> for LogRecord {
    fn from(record: &Record) -> Self {
        LogRecord {
            target: LogTarget(record.target().to_string()),
            level: record.level(),
            msg: LogMessage(format!("{}", record.args())),
            module_source: record.module_path().and_then(|module_path| {
                record
                    .line()
                    .map(|line| ModuleSource::new(module_path, line))
            }),
        }
    }
}

impl LogRecord {
    /// LogTarget getter
    pub fn target(&self) -> &LogTarget {
        &self.target
    }

    /// Level getter
    pub fn level(&self) -> Level {
        self.level
    }

    /// LogMessage getter
    pub fn message(&self) -> &LogMessage {
        &self.msg
    }

    /// ModuleSource getter
    pub fn module_source(&self) -> Option<&ModuleSource> {
        self.module_source.as_ref()
    }
}

impl Message for LogRecord {
    type Result = ();
}

impl Handler<LogRecord> for Logger {
    type Result = MessageResult<LogRecord>;

    fn handle(&mut self, request: LogRecord, _: &mut Self::Context) -> Self::Result {
        let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        match request.module_source() {
            Some(module_source) => eprintln!(
                "[{}][{}][{}][{}] {}",
                request.level(),
                now,
                request.target(),
                module_source,
                request.message()
            ),
            None => eprintln!(
                "[{}][{}][{}] {}",
                request.level(),
                now,
                request.target(),
                request.message()
            ),
        }

        MessageResult(())
    }
}

/// Sends LogRecord to the Logger Actor service.
#[derive(Debug)]
struct LogRecordSender {
    logger: Addr<Logger>,
}

impl LogRecordSender {
    /// The default Arbiter that hosts the Logger service actor
    pub const DEFAULT_ARBITER: actor::arbiters::Name = actor::arbiters::Name("OP_LOG");

    /// constructor
    pub fn new(logger: Addr<Logger>) -> Self {
        LogRecordSender { logger }
    }
}

/// Initializes the [log](https://crates.io/crates/log) system.
/// - log records are converted to LogRecord and sent asynchronously to the Logger service actor.
/// - the Logger service actor runs in a dedicated Arbiter, i.e., thread
///
/// # Panics
/// This function panics if actix system is not running.
pub fn init_logging(config: oysterpack_log::LogConfig) -> impl Future<Item = (), Error = ()> {
    let arbiters_addr = actor::app_service::<actor::arbiters::Arbiters>();
    let task = arbiters_addr
        .send(actor::arbiters::GetArbiter::from(
            LogRecordSender::DEFAULT_ARBITER,
        )).and_then(|arbiter| {
            arbiter.send(actix::msgs::Execute::new(move || -> Result<(), ()> {
                let logger = Arbiter::registry().get::<Logger>();
                oysterpack_log::init(config, LogRecordSender::new(logger));
                Ok(())
            }))
        });
    actor::into_task(task)
}

impl RecordLogger for LogRecordSender {
    fn log(&self, record: &Record) {
        self.logger.do_send(LogRecord::from(record));
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use actix::{dev::*, Arbiter, System};
    use actor::{app_service, arbiters};
    use futures::{future, prelude::*};

    fn stop_system() {
        println!("Sending System stop signal ...");
        System::current().stop();
        println!("System stop signalled");
    }

    #[test]
    fn log_message_from_log_record() {
        let record = Record::builder()
            .args(format_args!("BOOM!!!"))
            .level(Level::Error)
            .target("myApp")
            .file(Some("server.rs"))
            .line(Some(144))
            .module_path(Some("server"))
            .build();

        let log_record: LogRecord = LogRecord::from(&record);
        println!("{:?}", log_record);
        assert_eq!(*log_record.target(), LogTarget(record.target().to_string()));
        assert_eq!(log_record.level(), record.level());
        assert_eq!(
            *log_record.message(),
            LogMessage(format!("{}", record.args()))
        );
        assert_eq!(
            log_record.module_source(),
            Some(ModuleSource::new(
                record.module_path().unwrap(),
                record.line().unwrap()
            )).as_ref()
        );
    }

    #[test]
    fn logger_actor() {
        System::run(|| {
            let record = Record::builder()
                .args(format_args!("BOOM!"))
                .level(Level::Error)
                .target("myApp")
                .file(Some(file!()))
                .line(Some(line!()))
                .module_path(Some(module_path!()))
                .build();
            let record: LogRecord = LogRecord::from(&record);

            let arbiters_addr = app_service::<arbiters::Arbiters>();
            let task = arbiters_addr
                .send(arbiters::GetArbiter::from("log"))
                .and_then(|arbiter| {
                    arbiter.send(actix::msgs::Execute::new(|| -> Result<(), ()> {
                        let logger = Arbiter::registry().get::<Logger>();
                        actor::spawn_task(logger.send(record));
                        Ok(())
                    }))
                }).then(|_| {
                    System::current().stop();
                    future::ok::<(), ()>(())
                });

            actor::spawn_task(task);
        });
    }

    #[test]
    fn init_logging() {
        use oysterpack_log;

        fn log_config() -> oysterpack_log::LogConfig {
            oysterpack_log::config::LogConfigBuilder::new(Level::Info).build()
        }

        System::run(|| {
            let task = super::init_logging(log_config());
            let task = task
                .and_then(|_| {
                    for i in 0..10 {
                        info!("LOG MSG #{}", i);
                    }
                    Ok(())
                }).then(|_| {
                    // Not all log messages may have been processed. Queued messages will simply get dropped.
                    info!("STOPPING ACTOR SYSTEM");
                    System::current().stop();
                    future::ok::<(), ()>(())
                });
            actor::spawn_task(task);
        });
    }
}
