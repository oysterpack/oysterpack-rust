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
    messaging::reqrep::{self, metrics::*, *},
};
use oysterpack_trust::metrics::TimerBuckets;
use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

steps!(World => {
    // Feature: [01D4RW7WRVBBGTBZEQCXMFN51V] The ReqRep client can be shared by cloning it.

    // Scenario: [01D4RW8V6K8HR8R1QR8DMN2AQC] Clone the ReqRep client and send requests from multiple threads
    then regex "01D4RW8V6K8HR8R1QR8DMN2AQC" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        const REQ_COUNT: usize = 10;
        for _ in 0..REQ_COUNT {
            let mut client = client.clone();
            executor.spawn(async move {
                let count = await!(client.send_recv(CounterRequest::Inc)).unwrap();
                println!("count = {}", count);
            }).unwrap();
        }
        thread::yield_now();
        loop  {
            let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
            if count == REQ_COUNT {
                break;
            }
            println!("waiting for tasks to complete: count = {}",count);
            thread::yield_now();
        }
    };

    // Feature: [01D4RV5JQPQHXQNNJR8740J39J] Sending request is coupled with receiving reply

    // Scenario: [01D4RDC36HSVM7M65SCQK13T2S] Coupled async request / reply
    then regex "01D4RDC36HSVM7M65SCQK13T2S" | _world, _matches, _step | {
        let mut executor = global_executor();
        let mut client = counter_service();
        assert_eq!(executor.run(client.send_recv(CounterRequest::Inc)).unwrap(), 1);
    };

    // Scenario: [01D52268F786CHS3EE0QD5Y785] Backend service panics while processing request
    then regex "01D52268F786CHS3EE0QD5Y785-1" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        let result = executor.run(client.send_recv(CounterRequest::Panic));
        println!("result: {:?}", result);
        assert!(result.is_err());
    };

    then regex "01D52268F786CHS3EE0QD5Y785-2" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        {
            let mut client = client.clone();
            let result = executor.run(client.send_recv(CounterRequest::Panic));
            assert!(result.is_err());
        }
        let result = executor.run(client.send_recv(CounterRequest::Inc));
        println!("result: {:?}", result);
        assert!(result.is_err());
    };

    // Feature: [01D4R2Y8D8FCJGJ1JTFDVT4KD5] Sending request is decoupled from receiving reply

    // Scenario: [01D4R3896YCW74JWQH2N3CG4Y0] Send decoupled request
    then regex "01D4R3896YCW74JWQH2N3CG4Y0" | _world, _matches, _step | {
        let mut executor = global_executor();
        let mut client = counter_service();
        let count = executor.run(async {
            let receiver = await!(client.send(CounterRequest::Inc)).unwrap();
            await!(receiver.recv()).unwrap()
        });
        assert_eq!(count, 1);
    };

    // Scenario: [01D4RDHVTVKWHBBRX1P1EHX30A] Fire and forget - closing the ReplyReceiver
    then regex "01D4RDHVTVKWHBBRX1P1EHX30A" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        const REQ_COUNT: usize = 10;
        for _ in 0..REQ_COUNT {
            let mut client = client.clone();
            executor.spawn(async move {
                let receiver = await!(client.send(CounterRequest::Inc)).unwrap();
                receiver.close();
            }).unwrap();
        }
        thread::yield_now();
        loop  {
            let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
            if count == REQ_COUNT {
                break;
            }
            println!("waiting for tasks to complete: count = {}",count);
            thread::yield_now();
        }
    };

    // Scenario: [01D521HRC5G63GTWMJ7QQ1KHVS] Fire and forget - discarding the ReplyReceiver
    then regex "01D521HRC5G63GTWMJ7QQ1KHVS" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        const REQ_COUNT: usize = 10;
        for _ in 0..REQ_COUNT {
            let mut client = client.clone();
            executor.spawn(async move {
                let _receiver = await!(client.send(CounterRequest::Inc)).unwrap();
                // receiver falls out od scope and is discarded
            }).unwrap();
        }
        thread::yield_now();
        loop  {
            let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
            if count == REQ_COUNT {
                break;
            }
            println!("waiting for tasks to complete: count = {}",count);
            thread::yield_now();
        }
    };

    // Scenario: [01D5213GKE1JCB85WET013V681] Backend service panics while processing request
    then regex "01D5213GKE1JCB85WET013V681-1" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        let result = executor.run(async move {
            let receiver = await!(client.send(CounterRequest::Panic)).unwrap();
            await!(receiver.recv())
        });
        println!("result: {:?}", result);
        assert!(result.is_err());
    };

    then regex "01D5213GKE1JCB85WET013V681-2" | _world, _matches, _step | {
        let mut client = counter_service();
        let mut executor = global_executor();
        {
            let mut client = client.clone();
            let result = executor.run(async move {
                let receiver = await!(client.send(CounterRequest::Panic)).unwrap();
                await!(receiver.recv())
            });
            assert!(result.is_err());
        }

        let result = executor.run(client.send(CounterRequest::Inc));
        println!("result: {:?}", result);
        assert!(result.is_err());
    };

    // Scenario: [01D524VZWM925RKEBVP5C0WXYJ] Submit requests when the channel is full
    then regex "01D524VZWM925RKEBVP5C0WXYJ" | _world, _matches, _step | {
        let mut client = counter_service_with_channel_size(0);
        let mut executor = global_executor();
        const REQ_COUNT: usize = 5;
        for _ in 0..REQ_COUNT {
            let mut client = client.clone();
            executor.spawn(async move {
                await!(client.send(CounterRequest::SleepAndInc(Duration::from_millis(10)))).unwrap();
            }).unwrap();
        }
        thread::sleep(Duration::from_millis(10));
        loop  {
            let count = executor.run(client.send_recv(CounterRequest::Get)).unwrap();
            if count == REQ_COUNT {
                break;
            }
            println!("waiting for tasks to complete: count = {}",count);
            thread::yield_now();
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
pub struct World {}
