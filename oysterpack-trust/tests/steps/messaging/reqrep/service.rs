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

use float_cmp::ApproxEq;
use futures::{channel::oneshot, prelude::*, task::SpawnExt};
use oysterpack_trust::metrics::DurationBuckets;
use oysterpack_trust::{
    concurrent::{
        execution::{self, *},
        messaging::reqrep::{self, metrics::*, *},
    },
    metrics,
};
use prometheus::Encoder;
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};

steps!(World => {
    // Feature: [01D4Z9P9VVHP7NC4MWV6JQ5XBM] Backend service processing is executed async

    // Scenario: [01D4Z9S1E16YN6YSFXEHHY1KQT] Startup 10 ReqRep services on a single-threaded Executor
    given regex "01D4Z9S1E16YN6YSFXEHHY1KQT" | world, _matches, _step | {
        let buckets = DurationBuckets::Custom(vec![
            Duration::from_nanos(100),
            Duration::from_nanos(200),
            Duration::from_nanos(300),
        ]).buckets().unwrap();
        let executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_pool_size(NonZeroUsize::new(1).unwrap())
            .register()
            .unwrap();
        let clients: Vec<_> = (0..10).map(|_| {
            ReqRepConfig::new(ReqRepId::generate(), buckets.clone())
            .start_service(Counter::default(), executor.clone())
            .unwrap()
        }).collect();
        world.clients = Some(clients);
        world.executor = Some(executor);
    };

    when regex "01D4Z9S1E16YN6YSFXEHHY1KQT" | world, _matches, _step | {
        world.send_requests_using_clients(10, CounterRequest::Inc);
    };

    then regex "01D4Z9S1E16YN6YSFXEHHY1KQT" | world, _matches, _step | {
        let mut executor = world.executor.take().unwrap();
        let mut clients = world.clients.take().unwrap();
        clients.iter_mut().for_each(|client| {
            assert_eq!(executor.run(client.send_recv(CounterRequest::Get)).unwrap(), 10);
        });
    };

    // [01D4ZAMSJRS22CF9FN2WFZGMZM] Startup 10 ReqRep services on a mutli-threaded Executor
    given regex "01D4ZAMSJRS22CF9FN2WFZGMZM" | world, _matches, _step | {
        let clients: Vec<_> = (0..10).map(|_| counter_service()).collect();
        world.clients = Some(clients);
        world.executor = Some(execution::global_executor());
    };

    when regex "01D4ZAMSJRS22CF9FN2WFZGMZM" | world, _matches, _step | {
        world.send_requests_using_clients(10, CounterRequest::Inc);
    };

    then regex "01D4ZAMSJRS22CF9FN2WFZGMZM" | world, _matches, _step | {
        let mut executor = world.executor.take().unwrap();
        let mut clients = world.clients.take().unwrap();
        clients.iter_mut().for_each(|client| {
            loop {
                let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
                if count == 10 {
                    break;
                }

                println!("Waiting for client ({}) requests to complete: {} ...", client.id(), count);
                thread::sleep(Duration::from_millis(1));
            }
        });
    };

    // Scenario: [01D4ZANG5S4SZ07AZ0QJ0A8XJW] Startup 10 ReqRep services on one Executor and send requests on different Executor
    given regex "01D4ZANG5S4SZ07AZ0QJ0A8XJW" | world, _matches, _step | {
        let buckets = DurationBuckets::Custom(vec![
            Duration::from_nanos(100),
            Duration::from_nanos(200),
            Duration::from_nanos(300),
        ]).buckets().unwrap();
        let executor = ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap();
        let clients: Vec<_> = (0..10).map(|_| {
            ReqRepConfig::new(ReqRepId::generate(), buckets.clone())
            .start_service(Counter::default(), executor.clone())
            .unwrap()
        }).collect();
        world.clients = Some(clients);
        world.executor = Some(execution::global_executor());
    };

    when regex "01D4ZANG5S4SZ07AZ0QJ0A8XJW" | world, _matches, _step | {
        world.send_requests_using_clients(10, CounterRequest::Inc);
    };

    then regex "01D4ZANG5S4SZ07AZ0QJ0A8XJW" | world, _matches, _step | {
        let mut executor = world.executor.take().unwrap();
        let mut clients = world.clients.take().unwrap();
        clients.iter_mut().for_each(|client| {
            loop {
                let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
                if count == 10 {
                    break;
                }

                println!("Waiting for client ({}) requests to complete: {} ...", client.id(), count);
                thread::sleep(Duration::from_millis(1));
            }
        });
    };

    // Feature: [01D585SEWBEKBBR0ZY3C5GR7A6] Processor is notified via `Processor::panicked()` if a panic occurred while processing the request.

    // Scenario: [01D4ZGXQ27F3P7MXDW20K4RGR9] Processor::process() panics using default behavior
    when regex "01D4ZGXQ27F3P7MXDW20K4RGR9" | world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = execution::global_executor();
        executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        world.client = Some(client);
    };

    then regex "01D4ZGXQ27F3P7MXDW20K4RGR9-1" | world, _matches, _step | {
        for client in &world.client {
            assert_eq!(reqrep::metrics::service_instance_count(client.id()), 0);
        }
    };

    then regex "01D4ZGXQ27F3P7MXDW20K4RGR9-2" | world, _matches, _step | {
        let mut executor = execution::global_executor();
        for client in world.client.as_mut() {
            assert!(executor.run(client.send_recv(CounterRequest::Get)).is_err());
            assert!(executor.run(client.send(CounterRequest::Get)).is_err());
        }
    };

    // Scenario: [01D586H94GS723PJ2R1W4PTR6B] Processor::process() panics but Processor is designed to recover from the panic
    when regex "01D586H94GS723PJ2R1W4PTR6B" | world, _matches, _step | {
        let mut client = counter_service_ignoring_panics();
        let mut executor = execution::global_executor();
        executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        world.client = Some(client);
    };

    then regex "01D586H94GS723PJ2R1W4PTR6B-1" | world, _matches, _step | {
        for client in &world.client {
            assert_eq!(reqrep::metrics::service_instance_count(client.id()), 1);
        }
    };

    then regex "01D586H94GS723PJ2R1W4PTR6B-2" | world, _matches, _step | {
        let mut executor = execution::global_executor();
        for client in world.client.as_mut() {
            executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
            assert_eq!(executor.run(client.send_recv(CounterRequest::Get)).unwrap(), 2);
            let mut client = client.clone();
            assert_eq!(executor.run(async move {
                let receiver = await!(client.send(CounterRequest::Get)).unwrap();
                await!(receiver.recv())
            }).unwrap(), 2);
        }
    };

    then regex "01D586H94GS723PJ2R1W4PTR6B-3" | world, _matches, _step | {
        let mut executor = execution::global_executor();
        let histogram_timer = world.histogram_timer();
        let count = histogram_timer.get_sample_count();
        for client in world.client.as_mut() {
            // panic request does not count towards timer metric
            assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
            // gets reported against timer metric
            executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        }
        thread::sleep(Duration::from_millis(1));
        let histogram_timer = world.histogram_timer();
        let count2 = histogram_timer.get_sample_count();
        assert_eq!(count2, count + 1);
    };
});

#[derive(Debug, Default)]
struct Counter {
    count: Arc<RwLock<usize>>,
    ignore_panic: bool,
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
                    //println!("sleeping for {:?} ...", sleep);
                    thread::sleep(sleep);
                    let mut count = count.write().unwrap();
                    *count += 1;
                    *count
                }
            }
        }
            .boxed()
    }

    fn panicked(&mut self, err: PanicError) {
        if !self.ignore_panic {
            panic!(err)
        }
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
    let buckets = DurationBuckets::Custom(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]).buckets().unwrap();
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

fn counter_service_ignoring_panics() -> ReqRep<CounterRequest, usize> {
    let buckets = DurationBuckets::Custom(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]).buckets().unwrap();
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .start_service(
            Counter {
                count: Arc::default(),
                ignore_panic: true,
            },
            global_executor(),
        )
        .unwrap()
}

fn counter_service_with_reqrep_id(reqrep_id: ReqRepId) -> ReqRep<CounterRequest, usize> {
    let buckets = DurationBuckets::Custom(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]).buckets().unwrap();
    ReqRepConfig::new(reqrep_id, buckets)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

fn counter_service_with_timer_buckets(buckets: Vec<f64>) -> ReqRep<CounterRequest, usize> {
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

fn counter_service_with_channel_size(chan_size: usize) -> ReqRep<CounterRequest, usize> {
    let buckets = DurationBuckets::Custom(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ]).buckets().unwrap();
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .set_chan_buf_size(chan_size)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

#[derive(Default)]
pub struct World {
    client: Option<ReqRep<CounterRequest, usize>>,
    clients: Option<Vec<ReqRep<CounterRequest, usize>>>,
    executor: Option<Executor>,
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

    fn send_requests_using_clients(&mut self, req_count: usize, request: CounterRequest) {
        let mut executor = self.executor.take().expect("no executor");

        for clients in &self.clients {
            clients.iter().for_each(|client| {
                (0..req_count).for_each(|_| {
                    let mut client = client.clone();
                    executor
                        .spawn(
                            async move {
                                await!(client.send_recv(request)).unwrap();
                            },
                        )
                        .unwrap();
                });
            });
        }
        self.executor = Some(executor);
    }

    /// returns the histogram timer metric corresponding to the ReqRepId for the current world.client
    fn histogram_timer(&self) -> prometheus::proto::Histogram {
        let reqrep_id = self.client.as_ref().iter().next().unwrap().id();
        reqrep::metrics::histogram_timer_metric(reqrep_id).unwrap()
    }
}
