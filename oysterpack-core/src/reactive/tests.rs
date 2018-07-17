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
use chrono::prelude::*;
use crossbeam_channel as channel;
use errors;
use std::time::SystemTime;
use time::system_time;
use tokio::{self, prelude::*};

use tests::*;

#[derive(Fail, Debug, Clone, Copy)]
#[fail(display = "Foo error.")]
struct FooError;

#[test]
fn command_success_with_no_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = FooError;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(SystemTime::now()))
        }
    }

    let foo_id = CommandId::new(1);

    run_test(|| {
        let (s, r) = channel::unbounded();

        let foo_cmd = Command::new(foo_id, Foo)
            .and_then(move |result| {
                s.send(result);
                future::finished(result)
            })
            .map(|ts| {
                info!("{:?}", system_time::to_date_time(ts));
                ()
            })
            .map_err(|_| ());
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
        type Error = FooError;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Ok(Async::Ready(SystemTime::now()))
        }
    }

    let foo_id = CommandId::new(1);

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
            })
            .map(|ts| {
                info!("{:?}", system_time::to_date_time(ts));
                ()
            })
            .map_err(|_| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);

        let progress_events: Vec<_> = progress_receiver.collect();
        info!("Progress events: {:?}", progress_events);
        assert_eq!(progress_events.len(), 1);
        let progress = progress_events[0];
        assert_eq!(progress.status(), Status::SUCCESS);
        assert_eq!(progress.poll_counter(), 1);
        assert!(progress.poll_duration().subsec_nanos() > 0);
    });
}

#[test]
fn command_failure_with_progress_subscriber() {
    struct Foo;

    impl Future for Foo {
        type Item = SystemTime;
        type Error = errors::Error<FooError>;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            Err(errors::Error::new(errors::ErrorId::new(1), FooError))
        }
    }

    let foo_id = CommandId::new(1);

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
            })
            .map(|_: Result<SystemTime, errors::Error<FooError>>| ());
        tokio::run(foo_cmd);

        let result = r.try_recv();
        assert!(result.is_some());
        info!("Received result: {:?}", result);
        if let Some(Err(e)) = result {
            info!("Received Err result: {}", e);
        }

        let progress_events: Vec<_> = progress_receiver.collect();
        info!("Progress events: {:?}", progress_events);
        assert_eq!(progress_events.len(), 1);
        let progress = progress_events[0];
        assert_eq!(progress.status(), Status::FAILURE);
        assert_eq!(progress.poll_counter(), 1);
        assert!(progress.poll_duration().subsec_nanos() > 0);
    });
}
