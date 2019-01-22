/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! request/reply messaging bench tests

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![allow(warnings)]

use oysterpack_trust::concurrent::{execution::*, messaging::reqrep::*};

use futures::{
    channel::oneshot,
    future::RemoteHandle,
    stream::StreamExt,
    task::{Spawn, SpawnExt},
};
use oysterpack_log::*;
use std::{
    thread,
    time::{Duration, Instant},
};

/// This is benchmarking how fast messages can flow in a request/reply workflow.
/// - the request message is imply echoed back, and the message type is ()
///
/// avg response time ~11 microseconds
fn reqrep_bench_single_threaded(count: usize) -> Duration {
    const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);

    let executors = EXECUTORS.lock().unwrap();
    let mut executor = executors.global_executor();
    let (req_rep, req_receiver) = ReqRep::<(), ()>::new(REQREP_ID, 1);
    let server = async move {
        let mut req_receiver = req_receiver;
        while let Some(mut msg) = await!(req_receiver.next()) {
            // echo the request message back in the reply
            let req = msg.take_request().unwrap();
            if let Err(err) = msg.reply(req) {
                warn!("{}", err);
            }
        }
        info!("reqrep_bench_single_threaded: message listener has exited");
    };
    executor.spawn(server);

    let req_rep_1 = req_rep.clone();
    let start = Instant::now();
    executor.run(
        async move {
            let mut req_rep = req_rep_1;
            for _ in 0..count {
                let rep_receiver = await!(req_rep.send(())).unwrap();
                await!(rep_receiver.recv()).unwrap();
            }
        },
    );
    let end = Instant::now();
    end.duration_since(start)
}

/// This is benchmarking message throughput in a request/reply workflow running parallel tasks
/// - the request message is imply echoed back, and the message type is ()
///
/// avg response time throughput ~1.8 microseconds
fn reqrep_bench_multi_threaded(count: usize) -> Duration {
    const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);

    let executors = EXECUTORS.lock().unwrap();
    let mut executor = executors.global_executor();
    let (req_rep, req_receiver) = ReqRep::<(), ()>::new(REQREP_ID, num_cpus::get());
    let server = async move {
        let mut req_receiver = req_receiver;
        while let Some(mut msg) = await!(req_receiver.next()) {
            // echo the request message back in the reply
            let req = msg.take_request().unwrap();
            if let Err(err) = msg.reply(req) {
                warn!("{}", err);
            }
        }
        info!("reqrep_bench_multi_threaded: message listener has exited");
    };
    executor.spawn(server);

    let mut handles = Vec::with_capacity(count);
    let start = Instant::now();
    for i in 1..=count {
        let req_rep_1 = req_rep.clone();
        let task = async move {
            let mut req_rep = req_rep_1;
            let rep_receiver = await!(req_rep.send(())).unwrap();
            await!(rep_receiver.recv()).unwrap();
        };
        let handle = executor.spawn_with_handle(task).unwrap();
        handles.push(handle);
    }
    executor.run(
        async move {
            for handle in handles {
                let _ = await!(handle);
            }
        },
    );
    let end = Instant::now();
    end.duration_since(start)
}

fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info)
        .target_level(
            oysterpack_log::Target::from(env!("CARGO_PKG_NAME")),
            oysterpack_log::Level::Debug,
        )
        .build()
}

fn main() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let count = 100000;

    let duration = reqrep_bench_single_threaded(count);
    let nanos_per_req = duration.as_nanos() / (count as u128);
    info!(
        "reqrep_bench_single_threaded: count = {}, duration = {:?}, ns/req = {}",
        count, duration, nanos_per_req
    );

    let duration = reqrep_bench_multi_threaded(count);
    let nanos_per_req = duration.as_nanos() / (count as u128);
    info!(
        "reqrep_bench_multi_threaded: count = {}, duration = {:?}, ns/req = {}",
        count, duration, nanos_per_req
    );
}
