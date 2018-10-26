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

//! Log config

use log::Level;
use std::collections::BTreeMap;

/// Log config
#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    root_level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    crate_level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_levels: Option<BTreeMap<Target, Level>>,
}

impl LogConfig {

    /// Returns the root log level.
    pub fn root_level(&self) -> Level {
        self.root_level
    }

    /// Returns the configured crate log level
    pub fn crate_level(&self) -> Option<Level> {
        self.crate_level
    }

    /// Returns the configured target log levels
    pub fn target_levels(&self) -> Option<&BTreeMap<Target, Level>> {
        self.target_levels.as_ref()
    }
}

impl Default for LogConfig {
    /// Creates a default LogConfig with the root log level set to Warn
    fn default() -> Self {
        LogConfig {
            root_level: Level::Warn,
            crate_level: None,
            target_levels: None,
        }
    }
}

/// LogConfig builder
#[derive(Debug)]
pub struct LogConfigBuilder {
    config: LogConfig,
}

impl LogConfigBuilder {
    /// Constructs a new LogConfigBuilder with the specified root log level
    pub fn new(root_level: Level) -> Self {
        LogConfigBuilder {
            config: LogConfig {
                root_level,
                crate_level: None,
                target_levels: None,
            },
        }
    }

    /// Sets the log level for this crate
    pub fn crate_level(mut self, level: Level) -> Self {
        self.config.crate_level = Some(level);
        self
    }

    /// Sets the log level for the specified target
    pub fn target_level(mut self, target: Target, level: Level) -> Self {
        self.config
            .target_levels
            .get_or_insert(BTreeMap::new())
            .insert(target, level);
        self
    }

    /// Builds and returns the LogConfig
    pub fn build(self) -> LogConfig {
        self.config
    }
}

op_newtype! {
    /// Represents a log target
    #[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Target(pub String)
}

impl Target {
    /// Constructs a new Target by appending the specified target.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use oysterpack_log::config::Target;
    /// let foo = Target("foo".to_string());
    /// let foo_bar = foo.append(Target("bar".to_string()));
    /// assert_eq!(Target("foo::bar".to_string()), foo_bar);
    /// ```
    pub fn append(&self, target: Target) -> Target {
        Target::new(format!("{}::{}", self.0, target.0))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json;

    #[test]
    fn root_log_level_configured() {
        ::run_test("root_log_level_configured", || {
            let config = LogConfigBuilder::new(Level::Info).build();
            info!("{}", serde_json::to_string(&config).unwrap());
            assert_eq!(config.root_level(),Level::Info);
        });
    }

    #[test]
    fn default_log_config() {
        ::run_test("default_log_config", || {
            let config : LogConfig = Default::default();
            info!("{}", serde_json::to_string(&config).unwrap());
            assert_eq!(config.root_level(),Level::Warn);
            assert!(config.crate_level().is_none());
            assert!(config.target_levels().is_none());
        });
    }

}
