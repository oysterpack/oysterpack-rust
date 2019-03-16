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
use oysterpack_trust::metrics::timer_buckets;
use oysterpack_trust::{
    concurrent::{
        execution::{self, *},
        messaging::reqrep::{self, metrics::*, *},
    },
    metrics,
};
use prometheus::Encoder;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

steps!(World => {
    // Feature: [01D4ZS3J72KG380GFW4GMQKCFH] Message processing timer metrics are collected

    // Scenario: [01D5028W0STBFHDAPWA79B4TGG] Processor sleeps for 10 ms
    given regex "01D5028W0STBFHDAPWA79B4TGG" | world, _matches, _step | {
        world.client = Some(counter_service());
        let buckets = timer_buckets(vec![
            Duration::from_millis(5),
            Duration::from_millis(10),
            Duration::from_millis(15),
            Duration::from_millis(20),
        ]).unwrap();
        world.client = Some(counter_service_with_timer_buckets(buckets));
    };

    when regex "01D5028W0STBFHDAPWA79B4TGG" | world, _matches, _step | {
        const REQ_COUNT: usize = 5;
        world.send_requests(REQ_COUNT, CounterRequest::SleepAndInc(Duration::from_millis(10)));

        // wait until all requests have been sent
        'outer: loop {
            for client in world.client.as_ref() {
                if request_send_count(client.id()) == REQ_COUNT as u64 {
                    break 'outer;
                }
                println!("request_send_count = {}", request_send_count((client.id())));
                thread::sleep(Duration::from_millis(10));
            }
        }
        thread::sleep(Duration::from_millis(20));
    };

    then regex "01D5028W0STBFHDAPWA79B4TGG" | world, _matches, _step | {
        let histogram_timer = world.histogram_timer();
        println!("{:#?}", histogram_timer);
        assert_eq!(histogram_timer.get_sample_count(), 5);
        assert!(histogram_timer.get_sample_sum() > 0.050 && histogram_timer.get_sample_sum() < 0.052);
        let bucket = histogram_timer.get_bucket().iter().find(|bucket| bucket.get_cumulative_count() == 5).unwrap();
        println!("{:#?}", bucket);
        assert!(bucket.get_upper_bound().approx_eq(&0.015, std::f64::EPSILON, 2));
    };

    // Feature: [01D52CH5BJQM4D903VN1MJ10CC] The number of requests sent per ReqRepId is tracked

    // Scenario: [01D52CJPG0THNTQR04ZVTDJ3WR] Send 100 requests from multiple tasks
    then regex "01D52CJPG0THNTQR04ZVTDJ3WR" | world, _matches, _step | {
        let client = counter_service();
        let reqrep_id = client.id();
        world.client = Some(client);
        world.send_requests(100, CounterRequest::Inc);
        while request_send_count(reqrep_id) < 100 {
            thread::yield_now();
        }
    };

    // Feature: [01D4ZHRS7RV42RXN1R83Q8QDPA] The number of running ReqRep service backend instances will be tracked

    // Scenario: [01D4ZJJJCHMHAK12MGEY5EF6VF] start up 10 instances of a ReqRep service using the same ReqRepId
    then regex "01D4ZJJJCHMHAK12MGEY5EF6VF-1" | world, _matches, _step | {
        let reqrep_id = ReqRepId::generate();
        let clients: Vec<_> = (0..10).map(|_| counter_service_with_reqrep_id(reqrep_id)).collect();
        world.clients = Some(clients);
        while service_instance_count(reqrep_id) < 10 {
            thread::yield_now();
        }
    };

    when regex "01D4ZJJJCHMHAK12MGEY5EF6VF-2" | world, _matches, _step | {
        let mut clients = world.clients.take().unwrap();
        for _ in 0..3 {
            clients.pop();
        }
        world.clients = Some(clients);
    };

    then regex "01D4ZJJJCHMHAK12MGEY5EF6VF-3" | world, _matches, _step | {
        for clients in &world.clients {
            let reqrep_id = clients.first().unwrap().id();
            assert_eq!(reqrep::metrics::service_instance_count(reqrep_id), 7);
        }
    };

    // Scenario: [01D5891JGSV2PPAM9G22FV9T42] Processor::process() panics but Processor is designed to recover from the panic
    when regex "01D5891JGSV2PPAM9G22FV9T42" | world, _matches, _step | {
        let mut client = counter_service_ignoring_panics();
        let mut executor = execution::global_executor();
        executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        world.client = Some(client);
    };

    then regex "01D5891JGSV2PPAM9G22FV9T42-1" | world, _matches, _step | {
        for client in &world.client {
            assert_eq!(reqrep::metrics::service_instance_count(client.id()), 1);
        }
    };

    then regex "01D5891JGSV2PPAM9G22FV9T42-2" | world, _matches, _step | {
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

    then regex "01D5891JGSV2PPAM9G22FV9T42-3" | world, _matches, _step | {
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

    // Feature: [01D59X5KJ7Q72C2F2FP2VYVGS1] ReqRep related metrics can be easily gathered

    // Scenario: [01D59X6B8A40S941CMTRKWAMAB] Start up multiple ReqRep services
    given regex "01D59X6B8A40S941CMTRKWAMAB-1" | world, _matches, _step | {
        let mut executor = execution::global_executor();
        let clients: Vec<_> = (0..3).map(|_| counter_service_ignoring_panics())
            .map(|client| {
                {
                    let mut client = client.clone();
                    executor.spawn(async move {
                        await!(client.send(CounterRequest::Inc)).unwrap();
                    }).unwrap();
                }
                {
                    let mut client = client.clone();
                    executor.spawn(async move {
                        await!(client.send(CounterRequest::Get)).unwrap();
                    }).unwrap();
                }
                {
                    let mut client = client.clone();
                    executor.spawn(async move {
                        await!(client.send(CounterRequest::Panic)).unwrap();
                    }).unwrap();
                }
                client
            })
            .collect();
        world.clients = Some(clients);
        thread::yield_now();
    };

    when regex "01D59X6B8A40S941CMTRKWAMAB-2" | world, _matches, _step | {
        world.metrics = Some(reqrep::metrics::gather())
    };

    then regex "01D59X6B8A40S941CMTRKWAMAB-3" | world, _matches, _step | {
        let reqrep_label_id = reqrep::metrics::REQREPID_LABEL_ID.name();
        let reqrep_ids: Vec<_> = world.clients.iter()
            .flat_map(|clients| clients)
            .map(|client| client.id())
            .map(|id| id.to_string())
            .collect();
        for reqrep_metrics in &world.metrics {
            let metric_names: Vec<_> = reqrep::metrics::metric_ids().iter().map(|id|id.to_string()).collect();
            metric_names.iter().for_each(|metric_name| {
                let exists = reqrep_ids.iter().all(|reqrep_id| {
                    reqrep_metrics.iter().any(|mfs| {
                        mfs.get_name() == metric_name.as_str() &&
                        mfs.get_metric().iter().any(|m| m.get_label().iter().any(|l| {
                            l.get_name() == reqrep_label_id.as_str() && l.get_value() == reqrep_id.as_str()
                        }))
                    })
                });
                assert!(exists, format!("metric was not found for: {}", metric_name));
            });
        }
    };

    then regex "01D59X6B8A40S941CMTRKWAMAB-4" | world, _matches, _step | {
        // because of task scheduling issues, the service may not yet even be started
        let now = Instant::now();
        'outer: loop {
            let reqrep_metrics = reqrep::metrics::gather();
            let counts_match = reqrep_metrics.iter()
                    .filter_map(|mf| {
                        if mf.get_name() == reqrep::metrics::SERVICE_INSTANCE_COUNT_METRIC_ID.name().as_str() {
                            mf.get_metric().iter().next()
                        } else {
                            None
                        }
                    })
                    .all(|metric| {
                        let reqrep_id = metric.get_label().iter()
                            .find(|label_pair| {
                                label_pair.get_name() == reqrep::metrics::REQREPID_LABEL_ID.name()
                            })
                            .map(|label_pair| label_pair.get_value().parse::<ReqRepId>().unwrap())
                            .unwrap();
                            if (metric.get_gauge().get_value() as u64) != reqrep::metrics::service_instance_count(reqrep_id) {
                                println!("{} != {}", metric.get_gauge().get_value(),reqrep::metrics::service_instance_count(reqrep_id));
                            }
                            (metric.get_gauge().get_value() as u64) == reqrep::metrics::service_instance_count(reqrep_id)
                    });
            if counts_match {
                break;
            }
            // if the counts are not synced up after 100 ms, then fail the test
            if now.elapsed() < Duration::from_millis(100){
                thread::yield_now();
            } else {
                panic!("counts did not match")
            }
        }
    };

    then regex "01D59X6B8A40S941CMTRKWAMAB-5" | world, _matches, _step | {
        // because of task scheduling issues, the service may not yet even be started
        let now = Instant::now();
        'outer: loop {
            let reqrep_metrics = reqrep::metrics::gather();
            let counts_match = reqrep_metrics.iter()
                    .filter_map(|mf| {
                        if mf.get_name() == reqrep::metrics::REQREP_SEND_COUNTER_METRIC_ID.name().as_str() {
                            mf.get_metric().iter().next()
                        } else {
                            None
                        }
                    })
                    .all(|metric| {
                        let reqrep_id = metric.get_label().iter()
                            .find(|label_pair| label_pair.get_name() == reqrep::metrics::REQREPID_LABEL_ID.name())
                            .map(|label_pair| label_pair.get_value().parse::<ReqRepId>().unwrap())
                            .unwrap();
                            if metric.get_counter().get_value() as u64 != reqrep::metrics::request_send_count(reqrep_id) {
                                println!("{} != {}", metric.get_counter().get_value(),reqrep::metrics::request_send_count(reqrep_id));
                            }
                            metric.get_counter().get_value() as u64 == reqrep::metrics::request_send_count(reqrep_id)
                    });
            if counts_match {
                break;
            }
            // if the counts are not synced up after 100 ms, then fail the test
            if now.elapsed() < Duration::from_millis(100){
                thread::yield_now();
            } else {
                panic!("counts did not match")
            }
        }
    };

    then regex "01D59X6B8A40S941CMTRKWAMAB-6" | world, _matches, _step | {
        // because of task scheduling issues, the service may not yet even be started
        let now = Instant::now();
        'outer: loop {
            let reqrep_metrics = reqrep::metrics::gather();
            let counts_match = reqrep_metrics.iter()
                    .filter_map(|mf| {
                        if mf.get_name() == reqrep::metrics::REQREP_PROCESS_TIMER_METRIC_ID.name().as_str() {
                            mf.get_metric().iter().next()
                        } else {
                            None
                        }
                    })
                    .all(|metric| {
                        let reqrep_id = metric.get_label().iter()
                            .find(|label_pair| label_pair.get_name() == reqrep::metrics::REQREPID_LABEL_ID.name())
                            .map(|label_pair| label_pair.get_value().parse::<ReqRepId>().unwrap())
                            .unwrap();
                            metric.get_histogram().get_sample_count() as u64 == reqrep::metrics::histogram_timer_metric(reqrep_id).unwrap().get_sample_count()
                    });
            if counts_match {
                break;
            }
            // if the counts are not synced up after 100 ms, then fail the test
            if now.elapsed() < Duration::from_millis(100){
                thread::yield_now();
            } else {
                panic!("counts did not match")
            }
        }
    };

    // Feature: [01D59X5KJ7Q72C2F2FP2VYVGS1] ReqRep related metric descriptors can be easily retrieved

    // Scenario: [01D5AKRF2JQJTQZQAHZFTV5CEG] Get ReqRep related metric descriptors
    then regex "01D5AKRF2JQJTQZQAHZFTV5CEG" | world, _matches, _step | {
        let descs = reqrep::metrics::descs();
        println!("{:#?}", descs);
        let metric_ids = reqrep::metrics::metric_ids();
        println!("{:?}", metric_ids);
        assert!(metric_ids.iter().all(|metric_id| {
            descs.iter().any(|desc| desc.fq_name == metric_id.name())
        }));
    };

    // Feature: [01D59WRTHWQRPC8DYMN76RJ5X0] Backend Processor panics are tracked as a metric.

    // Scenario: [01D59WYME5Y6Z8TEHKPEH6ZFTR] Processor panics while processing a request - service terminates
    then regex "01D59WYME5Y6Z8TEHKPEH6ZFTR" | world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 0);
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 1);
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 1);
    };

    // Scenario: [01D5DCAFHBYKKMP4BH89VADGNB] Processor panics while processing a request - service recovers and keeps running
    then regex "01D5DCAFHBYKKMP4BH89VADGNB" | world, _matches, _step | {
        let mut client = counter_service_ignoring_panics();
        let mut executor = global_executor();
        executor.run(client.send_recv(CounterRequest::Inc)).unwrap();
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 0);
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 1);
        assert!(executor.run(client.send_recv(CounterRequest::Panic)).is_err());
        assert_eq!(reqrep::metrics::processor_panic_count(client.id()), 2);
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
    let buckets = timer_buckets(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ])
    .unwrap();
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

fn counter_service_ignoring_panics() -> ReqRep<CounterRequest, usize> {
    let buckets = timer_buckets(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ])
    .unwrap();
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
    let buckets = timer_buckets(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ])
    .unwrap();
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
    let buckets = timer_buckets(vec![
        Duration::from_nanos(100),
        Duration::from_nanos(200),
        Duration::from_nanos(300),
    ])
    .unwrap();
    ReqRepConfig::new(ReqRepId::generate(), buckets)
        .set_chan_buf_size(chan_size)
        .start_service(Counter::default(), global_executor())
        .unwrap()
}

#[derive(Default)]
pub struct World {
    client: Option<ReqRep<CounterRequest, usize>>,
    clients: Option<Vec<ReqRep<CounterRequest, usize>>>,
    metrics: Option<Vec<prometheus::proto::MetricFamily>>,
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
                        }
                    },
                )
                .unwrap();
        }
    }

    /// returns the histogram timer metric corresponding to the ReqRepId for the current world.client
    fn histogram_timer(&self) -> prometheus::proto::Histogram {
        let reqrep_id = self.client.as_ref().iter().next().unwrap().id();
        reqrep::metrics::histogram_timer_metric(reqrep_id).unwrap()
    }
}
