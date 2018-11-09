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
use futures::{
    prelude::*,
    sync::mpsc::{self, Receiver, Sender},
};
use tests::run_test;
use tokio::prelude::{
    future::{self, Loop},
    Async, IntoFuture,
};
use tokio_threadpool::blocking;

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

#[test]
fn futures_mpsc() {
    run_test("futures_mpsc", || {
        struct SendMessage<T> {
            sender: Sender<T>,
            msg: Option<T>,
        }

        impl<T> Future for SendMessage<T> {
            type Item = ();
            type Error = SendMessageError<T>;

            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                try_ready!(self.poll_ready());
                let msg = self.msg.take().unwrap();
                if let Err(err) = self.sender.try_send(msg) {
                    warn!("failed to send message: {}", err);
                    if err.is_full() {
                        self.msg = Some(err.into_inner());
                        return self.poll_ready();
                    }
                    if err.is_disconnected() {
                        return Err(SendMessageError::Disconnected(self.msg.take().unwrap()));
                    }
                }
                info!("SENT MESSAGE !!!");
                Ok(Async::Ready(()))
            }
        }

        impl<T> SendMessage<T> {
            fn poll_ready(&mut self) -> Poll<(), SendMessageError<T>> {
                self.sender
                    .poll_ready()
                    .map_err(|err| SendMessageError::Disconnected(self.msg.take().unwrap()))
            }
        }

        enum SendMessageError<T> {
            Disconnected(T),
        }

        let (mut tx, mut rx) = mpsc::channel::<String>(0);

        let mut tasks = vec![];
        for i in 0..3 {
            let task = SendMessage {
                sender: tx.clone(),
                msg: Some(format!("MSG #{}", i)),
            }.map_err(|err| panic!(err));
            tasks.push(task);
        }
        let task_count = tasks.len();
        info!("SendMessage task count = {}", task_count);

        // join the send tasks
        let task = future::join_all(tasks).map(|_| ());
        // once all is sent, then close the initial sender channel.
        // once all sender channels are closed, then the receiver channel will close cleanly
        let task = task.then(move |_| {
            tx.close();
            future::ok(())
        });
        // process all of the messages
        let task = task.join(rx.for_each(|msg|{
                info!("received msg: {}", msg);
                future::ok(())
            })
        ).map(|_| ());

        tokio::run(task);
        info!("tasks completed");
    });
}
