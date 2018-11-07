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

use super::*;
use tests::run_test;

#[test]
fn chrono_duration() {
    run_test("chrono_duration", || {
        let now = Utc::now();
        info!("now = {}", now.to_rfc3339());
        let duration = Utc::now().signed_duration_since(now);
        info!("duration = {} millisec", duration.num_milliseconds());
        info!("duration = {:?} microsec", duration.num_microseconds());
        info!("duration = {:?} nanosec", duration.num_nanoseconds());
    });
}

#[test]
fn futures_catch_unwind() {
    run_test("futures_catch_unwind", || {
        use futures::future::*;

        let mut future = ok::<i32, u32>(2);
        assert!(future.catch_unwind().wait().is_ok());

        let mut future = lazy(|| -> FutureResult<i32, u32> {
            panic!("BOOM!");
            ok::<i32, u32>(2)
        });
        let result = future.catch_unwind().wait();
        match result {
            Ok(_) => panic!("should have failed"),
            Err(err) => error!("future failed with err: {:?}", err),
        }
    });
}
