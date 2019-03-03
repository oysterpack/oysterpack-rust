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

use cucumber_rust::*;

use futures::{channel::oneshot, prelude::*, task::SpawnExt};
use oysterpack_trust::concurrent::{
    execution::{self, *},
    messaging::reqrep::{self, *},
};
use oysterpack_trust::metrics::TimerBuckets;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};

steps!(World => {
    // Feature: [01D4T5NV48PVFBC2R3Q80B6W72] The request channel buffer size is configurable

    // Scenario: [01D4T5RY9XDB6MYDE6X7R3X766] Use the default channel buffer size - send 10 requests from single ReqRep instance
    given regex "01D4T5RY9XDB6MYDE6X7R3X766" | world, _matches, _step | {
        world.client = Some(counter_service());
    };

    when regex "01D4T5RY9XDB6MYDE6X7R3X766" | world, _matches, _step | {
        // make an additional clone to verify that clones don't increase the shared buffer capacity
        let client = world.client.as_ref().clone();
        world.send_requests(10, CounterRequest::SleepAndInc(Duration::from_secs(10)));
        // this should be plenty of time for all requests to be sent before
        thread::sleep(Duration::from_millis(50));
    };

    then regex "01D4T5RY9XDB6MYDE6X7R3X766" | world, _matches, _step | {
        for client in world.client.as_ref() {
            assert_eq!(request_send_count(client.id()), 2);
        }
    };

    // Scenario: [01D52JKNS9FXDXQYPADGFWM3QK] Use the default channel buffer size - send 10 requests from 10 ReqRep instance
    given regex "01D52JKNS9FXDXQYPADGFWM3QK" | world, _matches, _step | {
        world.client = Some(counter_service());
    };

    when regex "01D52JKNS9FXDXQYPADGFWM3QK" | world, _matches, _step | {
        for _ in 0..10 {
            world.send_requests(10, CounterRequest::SleepAndInc(Duration::from_secs(10)));
        }
        // this should be plenty of time for all requests to be sent before
        thread::sleep(Duration::from_millis(50));
    };

    then regex "01D52JKNS9FXDXQYPADGFWM3QK" | world, _matches, _step | {
        for client in world.client.as_ref() {
            assert_eq!(request_send_count(client.id()), 11);
        }
    };

    // Scenario: [01D4T61JB50KNT3Y7VQ10VX2NR] Set the channel buffer size to 1 - send 10 requests from single ReqRep instance
    given regex "01D4T61JB50KNT3Y7VQ10VX2NR" | world, _matches, _step | {
        world.client = Some(counter_service_with_channel_size(1));
    };

    when regex "01D4T61JB50KNT3Y7VQ10VX2NR" | world, _matches, _step | {
        // make an additional clone to verify that clones don't increase the shared buffer capacity
        let client = world.client.as_ref().clone();
        world.send_requests(10, CounterRequest::SleepAndInc(Duration::from_secs(10)));
        // this should be plenty of time for all requests to be sent before
        thread::sleep(Duration::from_millis(50));
    };

    then regex "01D4T61JB50KNT3Y7VQ10VX2NR" | world, _matches, _step | {
        for client in world.client.as_ref() {
            assert_eq!(request_send_count(client.id()), 3);
        }
    };

    // Scenario: [01D52MG7FVEWE4HK6J05VRR49F] Set the channel buffer size to 1 - send 10 requests from 10 ReqRep instance
    given regex "01D52MG7FVEWE4HK6J05VRR49F" | world, _matches, _step | {
        world.client = Some(counter_service_with_channel_size(1));
    };

    when regex "01D52MG7FVEWE4HK6J05VRR49F" | world, _matches, _step | {
        // make an additional clone to verify that clones don't increase the shared buffer capacity
        let client = world.client.as_ref().clone();
        for _ in 0..10 {
            world.send_requests(10, CounterRequest::SleepAndInc(Duration::from_secs(10)));
        }
        // this should be plenty of time for all requests to be sent before
        thread::sleep(Duration::from_millis(50));
    };

    then regex "01D52MG7FVEWE4HK6J05VRR49F" | world, _matches, _step | {
        for client in world.client.as_ref() {
            assert_eq!(request_send_count(client.id()), 12);
        }
    };

});

#[derive(Debug, Default)]
struct Counter {
    count: Arc<RwLock<usize>>,
}

impl Processor<CounterRequest, usize> for Counter {
    fn process(&mut self, req: CounterRequest) -> FutureReply<usize> {
        let count = self.count.clone();
        async move {
            match req {
                CounterRequest::Inc => {
                    let mut count = count.write().unwrap();
                    *count += 1;
                    *count
                }
                CounterRequest::Get => {
                    let count = count.read().unwrap();
                    *count
                }
                CounterRequest::Panic => panic!("BOOM !!!"),
                CounterRequest::SleepAndInc(sleep) => {
                    println!("sleeping for {:?} ...", sleep);
                    thread::sleep(sleep);
                    let mut count = count.write().unwrap();
                    *count += 1;
                    *count
                }
            }
        }
            .boxed()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CounterRequest {
    Inc,
    Get,
    Panic,
    SleepAndInc(Duration),
}

fn counter_service() -> ReqRep<CounterRequest, usize> {
    let buckets = TimerBuckets::from(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]);
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

fn counter_service_with_channel_size(chan_size: usize) -> ReqRep<CounterRequest, usize> {
    let buckets = TimerBuckets::from(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]);
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .set_chan_buf_size(chan_size)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

#[derive(Default)]
pub struct World {
    client: Option<ReqRep<CounterRequest, usize>>,
}

impl World {
    fn send_requests(&mut self, req_count: usize, request: CounterRequest) {
        for client in self.client.as_ref() {
            let mut executor = global_executor();
            let mut client = client.clone();
            executor
                .spawn(
                    async move {
                        let mut sent_count = 0;
                        for _ in 0..req_count {
                            await!(client.send(request)).unwrap();
                            sent_count += 1;
                            println!("sent_count = {}", sent_count);
                        }
                    },
                )
                .unwrap();
        }
    }
}
