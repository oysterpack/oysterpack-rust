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
use actor::{self, events, GetServiceInfo, Service, ServiceInfo};
use futures::prelude::*;
use oysterpack_events::{
    Eventful,
    event::ModuleSource
};
use oysterpack_log::log::{
    Record,
    Level
};

/// Logger ServiceId (01CWXSW61VREYK48PZ7QE549G5)
pub const SERVICE_ID: actor::Id = actor::Id(1865243759930187031543830440471307781);

/// Logger Actor.
/// - should run in its own dedicated Arbiter, i.e., thread.
#[derive(Debug)]
pub struct Logger {
    service_info: ServiceInfo,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID),
        }
    }
}

op_actor_service! {
  Service(Logger)
}

/// LogRecord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    target: LogTarget,
    level: Level,
    msg: LogMessage,
    module_source: Option<ModuleSource>
}

/// Log target
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogTarget(String);

/// Log message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogMessage(String);

impl<'a> From<&'a Record<'a>> for LogRecord {
    fn from(record: &Record) -> Self {
        LogRecord {
            target: LogTarget(record.target().to_string()),
            level: record.level(),
            msg: LogMessage(format!("{}", record.args())),
            module_source: record.module_path().and_then(|module_path| {
                record.line().map(|line| ModuleSource::new(module_path,line))
            })
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

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_message_from_log_record() {
        let record = Record::builder()
            .args(format_args!("Error!"))
            .level(Level::Error)
            .target("myApp")
            .file(Some("server.rs"))
            .line(Some(144))
            .module_path(Some("server"))
            .build();

        let log_record: LogRecord = LogRecord::from(&record);
        println!("{:?}", log_record);
        assert_eq!(*log_record.target(),LogTarget(record.target().to_string()));
        assert_eq!(log_record.level(),record.level());
        assert_eq!(*log_record.message(),LogMessage(format!("{}",record.args())));
        assert_eq!(log_record.module_source(),Some(ModuleSource::new(record.module_path().unwrap(), record.line().unwrap())).as_ref());
    }
}
