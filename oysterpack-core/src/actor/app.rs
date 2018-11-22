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

//! Application Actor is used to represent the application.
//!
//!

use oysterpack_app_metadata::Build;
use oysterpack_log::LogConfig;
use oysterpack_uid::TypedULID;

use actor::{Id as ServiceId, InstanceId as ServiceInstanceId};

/// App represents an application instance.
#[derive(Debug)]
pub struct App {
    build: Build,
    instance_id: InstanceId,
    log_config: LogConfig,
}

/// Application Instance Id
type InstanceId = TypedULID<App>;
