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
use oysterpack_trust::metrics::TimerBuckets;
use oysterpack_trust::{
    concurrent::{
        execution::{self, *},
        messaging::reqrep::{self, *},
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
    time::Duration,
};

steps!(World => {
    // Feature: [01D4ZS9FX0GZZRG9RF072XGBQD] All ReqRep related metrics can be gathered via reqrep::gather_metrics()

    // Scenario: [01D4ZSJ5XPDVKG33NXDE6TP6QX] send requests and then gather metrics
    given regex "01D4ZSJ5XPDVKG33NXDE6TP6QX" | world, _matches, _step | {
        let client = counter_service();
        let reqrep_id = client.id();
        world.client = Some(client);
        world.send_requests(10, CounterRequest::Inc);

        // wait until all requests have been sent
        while request_send_count(reqrep_id) < 10 {
            thread::yield_now();
        }
        thread::yield_now();

        // gather metrics
        let reqrep_metrics = reqrep::gather_metrics();
        println!("{:#?}", reqrep_metrics);

        // check that all expected metrics have been gathered
        let metric_ids = vec![REQREP_PROCESS_TIMER_METRIC_ID, REQREP_SEND_COUNTER_METRIC_ID, SERVICE_INSTANCE_COUNT_METRIC_ID];
        metric_ids.iter().for_each(|meric_id| {
            let metric_name = REQREP_PROCESS_TIMER_METRIC_ID.name();
            let metric_name = metric_name.as_str();
            assert!(reqrep_metrics.iter().any(|mf| mf.get_name() == metric_name));
        });
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

fn counter_service_with_timer_buckets(buckets: TimerBuckets) -> ReqRep<CounterRequest, usize> {
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

    /// returns the histogram timer metric corresponding to the ReqRepId for the current world.client
    fn histogram_timer(&self) -> prometheus::proto::Histogram {
        let reqrep_id = self.client.as_ref().iter().next().unwrap().id();
        let reqrep_id = reqrep_id.to_string();
        let reqrep_id = reqrep_id.as_str();
        let histogram: Vec<_> = metrics::registry()
            .gather_for_metric_ids(&[REQREP_PROCESS_TIMER_METRIC_ID])
            .iter()
            .filter_map(|mf| {
                let metric = &mf.get_metric().iter().next().unwrap();
                if metric
                    .get_label()
                    .iter()
                    .any(|label_pair| label_pair.get_value() == reqrep_id)
                {
                    Some(metric.get_histogram().clone())
                } else {
                    None
                }
            })
            .collect();
        histogram.first().unwrap().clone()
    }
}
