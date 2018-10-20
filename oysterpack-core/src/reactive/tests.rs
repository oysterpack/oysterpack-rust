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

use super::command::*;
use crossbeam_channel as channel;
use errors;
use failure::{self, Fail};
use oysterpack_uid::ulid_u128;
use std::time::SystemTime;
use time::system_time;
use tokio::{self, prelude::*};

use tests::*;

#[derive(Fail, Debug, Clone, Copy)]
#[fail(display = "Foo error.")]
struct FooError;

impl FooError {
    fn error_id() -> errors::ErrorId {
        *FOO_ERROR_ID
    }
}

impl Into<errors::Error> for FooError {
    fn into(self) -> errors::Error {
        errors::Error::new(FooError::error_id(), self, op_src_loc!())
    }
}

lazy_static! {
    static ref FOO_ERROR_ID: errors::ErrorId = errors::ErrorId(ulid_u128());
}

fn line() -> u32 {
    line!()
}

#[test]
fn line_numbers() {
    run_test(|| {
        info!("line = {}", line());
        info!("line = {}", line());
        info!("file {}", file!());
        info!("module_path {}", module_path!());
    });
}

#[test]
fn command_future_success_with_no_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = errors::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(SystemTime::now()))
        }
    }

    let foo_id = CommandId(1);

    run_test(|| {
        let (s, r) = channel::unbounded();

        let foo_cmd = Command::new(foo_id, Foo)
            .and_then(move |result| {
                s.send(result);
                future::finished(result)
            }).map(|ts| {
                info!("{:?}", system_time::to_date_time(ts));
                ()
            }).map_err(|_| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);
    });
}

#[test]
fn command_success_with_no_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = errors::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(SystemTime::now()))
        }
    }

    let foo_id = CommandId(1);

    run_test(|| {
        let (s, r) = channel::unbounded();

        let foo_cmd = Command::new(foo_id, Foo)
            .and_then(move |result| {
                s.send(result);
                future::finished(result)
            }).map(|ts| {
                info!("{:?}", system_time::to_date_time(ts));
                ()
            }).map_err(|_| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);
    });
}

#[test]
fn command_success_with_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = errors::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(SystemTime::now()))
        }
    }

    let foo_id = CommandId(1);

    run_test(|| {
        let (s, r) = channel::unbounded();

        let (progress_sender, progress_receiver) = channel::unbounded();

        let foo_cmd = Builder::new(foo_id, Foo)
            .progress_subscriber_chan(progress_sender)
            .build();
        let foo_cmd = foo_cmd
            .and_then(move |result| {
                s.send(result);
                future::finished(result)
            }).map(|ts| {
                info!("{:?}", system_time::to_date_time(ts));
                ()
            }).map_err(|_| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);

        let progress_events: Vec<_> = progress_receiver.collect();
        info!("Progress events: {:?}", progress_events);
        assert_eq!(progress_events.len(), 1);
        let progress = &progress_events[0];
        assert!(progress.status().success());
        assert_eq!(progress.poll_counter(), 1);
        assert!(progress.poll_duration().subsec_nanos() > 0);
    });
}

#[test]
fn command_failure_with_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = errors::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Err(FooError.into())
        }
    }

    let foo_id = CommandId(1);

    run_test(|| {
        let (s, r) = channel::unbounded();

        let (progress_sender, progress_receiver) = channel::unbounded();

        let foo_cmd = Builder::new(foo_id, Foo)
            .progress_subscriber_chan(progress_sender)
            .build();
        let foo_cmd = foo_cmd
            .then(move |result| {
                s.send(result.clone());
                future::finished(result)
            }).map(|_: Result<SystemTime, errors::Error>| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);
        if let Some(Err(e)) = result {
            info!("Received Err result: {}", e);
            assert_eq!(
                e.error_ids(),
                vec![COMMAND_FAILURE_ERROR_ID, FooError::error_id()]
            );

            // TODO: it's too complicated to inspect the error
            // Error -> ArcFailure -> Context<CommandFailure> ->
            let failure: &errors::ArcFailure = e.failure().downcast_ref().unwrap();
            let failure: &failure::Context<CommandFailure> =
                failure.failure().downcast_ref().unwrap();
            let failure: &errors::Error = failure.cause().unwrap().downcast_ref().unwrap();
            let failure: &errors::ArcFailure = failure.failure().downcast_ref().unwrap();
            let _: &FooError = failure.failure().downcast_ref().unwrap();

            // failure cause chain
            // &failure::Context<CommandFailure> -> &errors::Error -> &FooError
            let failure: &failure::Context<CommandFailure> =
                e.cause().unwrap().downcast_ref().unwrap();
            let failure: &errors::Error = failure.cause().unwrap().downcast_ref().unwrap();
            let _: &FooError = failure.cause().unwrap().downcast_ref().unwrap();
        }

        let progress_events: Vec<_> = progress_receiver.collect();
        info!("Progress events: {:?}", progress_events);
        assert_eq!(progress_events.len(), 1);
        let progress = &progress_events[0];
        assert!(progress.status().failure());
        match progress.status() {
            &Status::FAILURE(ref err) => {
                assert_eq!(err.id(), FooError::error_id());
                let error_id_chain = err.error_ids();
                // the error id chain should not contain a CommandFailure Error
                // it is intentially not included with the Progress because the CommandFailure info
                // would be redundant - it is already provided by Progress
                assert!(!error_id_chain.contains(&COMMAND_FAILURE_ERROR_ID));
                assert_eq!(error_id_chain, vec![FooError::error_id()])
            }
            status => panic!(
                "The command should have failed, but the status is ; {:?}",
                status
            ),
        }
        assert_eq!(progress.poll_counter(), 1);
        assert!(progress.poll_duration().subsec_nanos() > 0);
    });
}
