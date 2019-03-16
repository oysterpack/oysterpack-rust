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

//! Provides support for request/reply messaging via async futures based channels.
//!
//! The design pattern is to decouple the backend service from the client via message channels.
//! The interface is defined by [ReqRep](struct.ReqRep.html) which defines:
//! - request message type
//! - response message type
//! - [ReqRepId](struct.ReqRepId.html) - represents the function identifier
//!
//! <pre>
//! client ---Req--> ReqRep ---Req--> service
//! client <--Rep--- ReqRep <--Rep--- service
//! </pre>
//!
//! The beauty of this design is that the client and service are decoupled. Clients and services can
//! be distributed over the network or be running on the same machine. This also makes it easy to mock
//! services for testing purposes.
//!
//! ## Client Features
//! - *[01D4R2Y8D8FCJGJ1JTFDVT4KD5]* Sending request is decoupled from receiving reply
//!   - [ReqRep::send()](struct.ReqRep.html#method.send) is used to send the request and returns a ReplyReceiver
//!   - [ReplyReceiver](struct.ReplyReceiver.html) is used to receive the reply at the client's leisure
//!     - this enables the client to do additional work between sending the request and receiving the reply
//! - *[01D4RV5JQPQHXQNNJR8740J39J]* Sending request is coupled with receiving reply
//!   - [ReqRep::send_recv()](struct.ReqRep.html#method.send_recv)
//! - *[01D4RW7WRVBBGTBZEQCXMFN51V]* The ReqRep client can be shared by cloning it
//!
//! ## Service Features
//! - *[01D4Z9P9VVHP7NC4MWV6JQ5XBM]* Backend service processing is executed async
//!   - Backend services receive messages in a non-blocking fashion, i.e., threads are not blocked waiting for messages.
//!   - Processor::process() is designed to return an async task which is scheduled to run on the the same Executor thread that
//!    is running the service task
//! - *[01D4ZAQBNT7MF2E0PWW77BJ6HS]* The backend service Processor lifecycle hooks are invoked on service startup and shutdown
//!   - when the backend service starts up and before processing any messages, the [Processor::init()](trait.Processor.html#method.init) lifecycle method is called
//!   - when the backend service is shutdown, [Processor::destroy()](trait.Processor.html#method.destroy) is invoked
//! - *[01D585SEWBEKBBR0ZY3C5GR7A6]* Processor is notified via [Processor::panicked()](trait.Processor.html#method.panicked) if a panic occurred while processing the request.
//!   - The default implementation simply cascades the panic, which terminates the ReqRep service
//! - *[01D4RWGKRYAJCQ4Q5SD3Z6WG6P]* When all ReqRep client references fall out of scope, then the backend service will automatically shutdown
//!
//! ## Config Features
//! - *[01D4RVW8XQCSZKNQEBGWKG57S5]* Each request / reply service is assigned a [ReqRepId](struct.ReqRepId.html)
//! - *[01D4T5NV48PVFBC2R3Q80B6W72]* The request channel buffer size is configurable
//!   - This is referring to the channel used by the ReqRep client to send requests to the backend service.
//!   - By default, the channel buffer size is 0.
//!     - The channel's capacity is equal to buffer + num-senders. In other words, each sender gets a guaranteed slot in the
//!       channel capacity, and on top of that there are buffer "first come, first serve" slots available to all senders.
//! - *[01D4V1PZ43Z5P7XGED38V6DXHA]* [TimerBuckets](../../../metrics/struct.TimerBuckets.html) are configurable per ReqRep
//!   - TimerBuckets are used to configure a histogram metric used to time message processing in the backend service.
//!   - TimerBuckets are not a one size fits all, and need to be tailored to the performance requirements for the backend Processor.
//!
//! ## Metric Features
//! - *[01D52CH5BJQM4D903VN1MJ10CC]* The number of requests sent per ReqRepId is tracked
//! - *[01D4ZHRS7RV42RXN1R83Q8QDPA]* The number of running ReqRep service backend instances are tracked
//! - *[01D4ZS3J72KG380GFW4GMQKCFH]* Message processing timer metrics are collected
//! - *[01D59WRTHWQRPC8DYMN76RJ5X0]* Backend Processor panics are tracked
//! - *[01D59X5KJ7Q72C2F2FP2VYVGS1]* ReqRep related metric descriptors can be easily retrieved
//! - *[01D59X5KJ7Q72C2F2FP2VYVGS1]* ReqRep related metrics can be easily gathered
//!
//! ```rust
//! # #![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
//! # use oysterpack_trust::concurrent::messaging::reqrep::{self, *};
//! # use oysterpack_trust::concurrent::execution::*;
//! # use oysterpack_trust::metrics;
//! # use futures::{task::*, future::FutureExt};
//! # use std::time::*;
//! // ReqRep backend processor
//! struct Inc;
//!
//! impl Processor<usize, usize> for Inc {
//!   fn process(&mut self, req: usize) -> reqrep::FutureReply<usize> {
//!      async move { req + 1 }.boxed()
//!   }
//! }
//!
//! // the ReqRepId should be defined as a constant
//! const REQREP_ID: ReqRepId = ReqRepId(1872692872983539779132843447162269015);
//!
//! // configure the timer histogram bucket according to your use case
//! let timer_buckets = metrics::DurationBuckets::Custom(vec![
//!     Duration::from_millis(500),
//!     Duration::from_millis(1000)]
//! ).buckets().unwrap();
//! let config = ReqRepConfig::new(REQREP_ID, timer_buckets);
//! let mut client = config.start_service(Inc, global_executor()).unwrap();
//! global_executor().spawn( async move {
//!   // send the request async
//!   let reply_receiver = await!(client.send(1)).unwrap();
//!   // or send the request and await for the reply async
//!   assert_eq!(2, await!(client.send_recv(1)).unwrap());
//!   // await for the reply from the request sent above
//!   assert_eq!(2, await!(reply_receiver.recv()).unwrap());
//! });
//! ```

use crate::concurrent::{execution::Executor, messaging::errors::ChannelError};
use futures::{
    channel,
    prelude::*,
    task::{SpawnError, SpawnExt},
};
use maplit::hashmap;
use oysterpack_log::*;
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    fmt::{self, Debug},
    panic::AssertUnwindSafe,
    pin::Pin,
    time::Instant,
};

pub mod metrics;

/// ReqRep is used to configure and start a ReqRep service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqRepConfig {
    reqrep_id: ReqRepId,
    chan_buf_size: usize,
    metric_timer_buckets: Vec<f64>,
}

impl ReqRepConfig {
    /// ReqRepId getter
    pub fn reqrep_id(&self) -> ReqRepId {
        self.reqrep_id
    }

    /// Returns the channel buffer size
    pub fn chan_buf_size(&self) -> usize {
        self.chan_buf_size
    }

    /// Returns TimerBuckets used to configure the Histogram timer metric
    pub fn metric_timer_buckets(&self) -> &[f64] {
        &self.metric_timer_buckets
    }

    /// constructor
    /// - the chan_buf_size default = 1
    /// - the TimerBuckets should be based on expected response times
    pub fn new(reqrep_id: ReqRepId, metric_timer_buckets: Vec<f64>) -> Self {
        Self {
            reqrep_id,
            chan_buf_size: 0,
            metric_timer_buckets,
        }
    }

    /// sets the channel buffer size
    ///
    /// The channel's capacity is equal to buffer + num-senders. In other words, each sender gets a
    /// guaranteed slot in the channel capacity, and on top of that there are buffer "first come, first serve"
    /// slots available to all senders.
    pub fn set_chan_buf_size(mut self, chan_buf_size: usize) -> ReqRepConfig {
        self.chan_buf_size = chan_buf_size;
        self
    }

    /// Starts the backend service message processor and returns the frontend ReqRep client, which
    /// communicates with the backend service via a channel.
    pub fn start_service<Req, Rep, Service>(
        self,
        processor: Service,
        executor: Executor,
    ) -> Result<ReqRep<Req, Rep>, SpawnError>
    where
        Req: Debug + Send + 'static,
        Rep: Debug + Send + 'static,
        Service: Processor<Req, Rep> + Send + 'static,
    {
        ReqRep::start_service(
            self.reqrep_id,
            self.chan_buf_size,
            processor,
            executor,
            self.metric_timer_buckets,
        )
    }
}

/// Implements a request/reply messaging pattern. Think of it as a generic function: `Req -> Rep`
/// - each ReqRep is assigned a unique ReqRepId - think of it as the function identifier
#[derive(Clone)]
pub struct ReqRep<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    request_sender: channel::mpsc::Sender<ReqRepMessage<Req, Rep>>,
    reqrep_id: ReqRepId,
    request_send_counter: prometheus::IntCounter,
}

impl<Req, Rep> ReqRep<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    /// Returns the ReqRepId
    pub fn id(&self) -> ReqRepId {
        self.reqrep_id
    }

    /// Send the request async
    /// - the ReplyReceiver is used to receive the reply via an async Future
    pub async fn send(&mut self, req: Req) -> Result<ReplyReceiver<Rep>, ChannelError> {
        let (rep_sender, rep_receiver) = channel::oneshot::channel::<Rep>();
        let msg = ReqRepMessage {
            req: Some(req),
            rep_sender,
        };
        await!(self.request_sender.send(msg))?;
        self.request_send_counter.inc();
        Ok(ReplyReceiver {
            receiver: rep_receiver,
        })
    }

    /// Send the request and await to receive a reply
    pub async fn send_recv(&mut self, req: Req) -> Result<Rep, ChannelError> {
        let receiver = await!(self.send(req))?;
        let rep = await!(receiver.recv())?;
        Ok(rep)
    }

    /// constructor
    ///
    /// ## Notes
    /// - the backend service channel is returned, which needs to be wired up to a backend service
    ///   implementation
    ///   - see [start_service()](struct.ReqRep.html#method.start_service)
    fn new(
        reqrep_id: ReqRepId,
        chan_buf_size: usize,
    ) -> (
        ReqRep<Req, Rep>,
        channel::mpsc::Receiver<ReqRepMessage<Req, Rep>>,
    ) {
        let (request_sender, request_receiver) = channel::mpsc::channel(chan_buf_size);
        (
            ReqRep {
                reqrep_id,
                request_sender,
                request_send_counter: metrics::REQREP_SEND_COUNTER
                    .with_label_values(&[reqrep_id.to_string().as_str()]),
            },
            request_receiver,
        )
    }

    /// Spawns the backend service message processor and returns the frontend ReqRep.
    /// - the backend service is spawned using the specified Executor
    /// - buckets are used to define the timer's histogram buckets
    ///   - each ReqRep service can have its own requirements
    ///   - timings will be reported in fractional seconds per prometheus best practice
    ///   - if multiple instances of the same service, i.e., using the same ReqRepId, are started,
    ///     then the first set of timer buckets will be used - because the histogram timer metric
    ///     is registered when the first service is started and then referenced by additional service
    ///     instances
    ///
    /// ## Params
    /// - reqrep_id: ReqRepId - the service ID
    /// - chan_buf_size: usize - the channel buffer size used to send requests to the backend service message processor
    /// - executor: Executor - used to spawn the backend service message processor
    /// - metric_timer_buckets - used to configure Histogram timer metric
    ///
    /// ## Service Metrics
    /// - Processor timer (Histogram)
    ///   - [ReqRepId](struct.ReqRepId.html) is used to construct the MetricId
    /// - Service instance count (IntGauge)
    ///   - [SERVICE_INSTANCE_COUNT_METRIC_ID]() defines the MetricId
    ///   - [REQREPID_LABEL_ID]() contains the ReqRepId ULID
    ///   - when the backend service exits, the count is decremented
    fn start_service<Service>(
        reqrep_id: ReqRepId,
        chan_buf_size: usize,
        processor: Service,
        mut executor: Executor,
        metric_timer_buckets: Vec<f64>,
    ) -> Result<ReqRep<Req, Rep>, SpawnError>
    where
        Service: Processor<Req, Rep> + Send + 'static,
    {
        let reqrep_service_metrics = move || {
            let mut reqrep_metrics = metrics::REQ_REP_METRICS.write();
            reqrep_metrics
                .entry(reqrep_id)
                .or_insert_with(|| {
                    let timer = crate::metrics::registry()
                        .register_histogram(
                            metrics::REQREP_PROCESS_TIMER_METRIC_ID,
                            "ReqRep message processor timer in seconds",
                            metric_timer_buckets,
                            Some(hashmap! {
                                metrics::REQREPID_LABEL_ID => reqrep_id.to_string()
                            }),
                        )
                        .unwrap();
                    let service_count = metrics::REQ_REP_SERVICE_INSTANCE_COUNT
                        .with_label_values(&[reqrep_id.to_string().as_str()]);

                    let panic_count = metrics::PROCESSOR_PANIC_COUNTER
                        .with_label_values(&[reqrep_id.to_string().as_str()]);

                    ReqRepServiceMetrics {
                        timer,
                        service_count,
                        panic_count,
                    }
                })
                .clone()
        };

        let (reqrep, mut req_receiver) = ReqRep::<Req, Rep>::new(reqrep_id, chan_buf_size);
        let reqrep_service_metrics = reqrep_service_metrics();
        let service_count = reqrep_service_metrics.service_count.clone();

        let mut processor = AssertUnwindSafe(processor);
        let service = async move {
            processor.init();
            reqrep_service_metrics.service_count.inc();
            let mut request_count: u64 = 0;
            let service_type_id = std::any::TypeId::of::<Service>();

            while let Some(mut msg) = await!(req_receiver.next()) {
                request_count += 1;
                let req = msg.take_request().unwrap();

                // time the request processing
                let start = Instant::now();
                let process_future = processor.process(req);
                let process_future = AssertUnwindSafe(process_future);
                let rep = await!(process_future.catch_unwind());
                let elapsed = start.elapsed();

                match rep {
                    Ok(rep) => {
                        // send back the reply
                        // we don't care if the client reply channel is disconnected
                        let _ = msg.reply(rep);

                        // record the timing metric
                        reqrep_service_metrics
                            .timer
                            .observe(crate::metrics::duration_as_secs_f64(elapsed));
                    }
                    Err(err) => {
                        reqrep_service_metrics.panic_count.inc();
                        processor.panicked(err);
                    }
                }

                debug!(
                    "ReqRepId({}) {:?} #{} : {:?}",
                    reqrep_id, service_type_id, request_count, elapsed
                );
            }
            processor.destroy();
        };

        let service = AssertUnwindSafe(service);
        executor.spawn(
            async move {
                let _ = await!(service.catch_unwind());
                service_count.dec();
            },
        )?;
        Ok(reqrep)
    }
}

impl<Req, Rep> fmt::Debug for ReqRep<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ReqRep")
            .field("reqrep_id", &self.reqrep_id)
            .field("request_send_count", &self.request_send_counter.get())
            .finish()
    }
}

/// ReqRep service metrics
#[derive(Clone)]
struct ReqRepServiceMetrics {
    timer: prometheus::Histogram,
    service_count: prometheus::IntGauge,
    panic_count: prometheus::IntCounter,
}

/// Message used for request/reply patterns.
#[derive(Debug)]
struct ReqRepMessage<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    req: Option<Req>,
    rep_sender: channel::oneshot::Sender<Rep>,
}

impl<Req, Rep> ReqRepMessage<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    /// Take the request, i.e., which transfers ownership
    ///
    /// ## Notes
    /// - this can only be called once - once the request message is taken, None is always returned
    fn take_request(&mut self) -> Option<Req> {
        self.req.take()
    }

    /// Send the reply
    fn reply(self, rep: Rep) -> Result<(), ChannelError> {
        self.rep_sender
            .send(rep)
            .map_err(|_| ChannelError::SenderDisconnected)
    }
}

/// Each request/reply API is uniquely identified by an ID.
///
/// ## Use Cases
/// 1. Used for tracking purposes
/// 2. Used to map to a ReqRep message processor
#[ulid]
pub struct ReqRepId(pub u128);

/// Reply Receiver
/// - is used to decouple sending the request from receiving the reply - see [ReqRep::send()](struct.ReqRep.html#method.send)
#[derive(Debug)]
pub struct ReplyReceiver<Rep>
where
    Rep: Debug + Send + 'static,
{
    receiver: channel::oneshot::Receiver<Rep>,
}

impl<Rep> ReplyReceiver<Rep>
where
    Rep: Debug + Send + 'static,
{
    /// Receive the reply
    pub async fn recv(self) -> Result<Rep, ChannelError> {
        let rep = await!(self.receiver)?;
        Ok(rep)
    }

    /// Closes the receiver channel
    pub fn close(mut self) {
        self.receiver.close()
    }
}

/// Request/reply message processor
/// - the `init()` and `destroy()` are lifecycle hooks, which by default are noop
/// - the Processor implementation is assumed to be [UnwindSafe](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html)
pub trait Processor<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    /// request / reply processing
    /// - it returns a Future which will produce the reply
    /// - the reason it returns a future is to minimize blocking within the Processor task
    ///   - *NOTE*: ideally, it would be cleaner if this method was async. However, async methods on a
    ///     trait are currently not supported, but are planned to be supported.
    fn process(&mut self, req: Req) -> FutureReply<Rep>; // TODO: change to an async method when async methods become supported on traits

    /// Invoked before any messages have been sent
    fn init(&mut self) {}

    /// Invoked when the message processor service is being shutdown
    fn destroy(&mut self) {}

    /// invoked if `process()` panics
    /// - the default behavior is to simply cascade the panic
    /// - if the Processor can recover from panics, then ensure the the Processor is designed to be
    ///   [UnwindSafe](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html)
    fn panicked(&mut self, err: PanicError) {
        panic!(err)
    }
}

/// Panic error type
pub type PanicError = Box<dyn Any + Send + 'static>;

/// Pinned Boxed Future type alias
pub type FutureReply<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrent::execution::global_executor;
    use crate::concurrent::messaging::reqrep;
    use crate::configure_logging;
    use futures::{
        channel::oneshot,
        future::FutureExt,
        stream::StreamExt,
        task::{Spawn, SpawnExt},
    };
    use oysterpack_log::*;
    use std::{thread, time::Duration};

    #[test]
    fn req_rep() {
        configure_logging();
        const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);
        let mut executor = global_executor();
        let (mut req_rep, mut req_receiver) = ReqRep::<usize, usize>::new(REQREP_ID, 1);
        let server = async move {
            while let Some(mut msg) = await!(req_receiver.next()) {
                info!("Received request: ReqRepId({})", REQREP_ID,);
                let n = msg.take_request().unwrap();
                if let Err(err) = msg.reply(n + 1) {
                    warn!("{}", err);
                }
            }
            info!("message listener has exited");
        };
        executor.spawn(server);
        let task = async {
            let rep_receiver = await!(req_rep.send(1)).unwrap();
            await!(rep_receiver.recv()).unwrap()
        };
        let n = executor.run(task);
        info!("n = {}", n);
        assert_eq!(n, 2);
    }

    #[test]
    fn req_rep_start_service() {
        configure_logging();
        const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);
        let mut executor = global_executor();

        // ReqRep processor //
        struct Inc;

        impl Processor<usize, usize> for Inc {
            fn process(&mut self, req: usize) -> reqrep::FutureReply<usize> {
                async move { req + 1 }.boxed()
            }
        }
        // ReqRep processor //

        let timer_buckets = crate::metrics::DurationBuckets::Custom(
            vec![Duration::from_millis(500), Duration::from_millis(1000)],
        ).buckets().unwrap();

        // GIVEN: a ReqRep client
        let mut req_rep =
            ReqRep::start_service(REQREP_ID, 1, Inc, executor.clone(), timer_buckets).unwrap();

        let task = async {
            // WHEN: a request is sent async
            // THEN: a ReplyReceiver is returned
            let rep_receiver = await!(req_rep.send(1)).unwrap();
            // WHEN: the reply is received async
            await!(rep_receiver.recv()).unwrap()
        };
        // THEN: a reply is received
        let n = executor.run(task);
        info!("n = {}", n);
        assert_eq!(n, 2);
        info!("{:#?}", crate::metrics::registry().gather());

        let task = async {
            // WHEN: a request is sent
            await!(req_rep.send_recv(1)).unwrap()
        };
        // THEN: a reply is received
        let n = executor.run(task);
        assert_eq!(n, 2);
    }

    #[test]
    fn req_rep_service_config_start_service() {
        configure_logging();
        let REQREP_ID: ReqRepId = ReqRepId::generate();
        let mut executor = global_executor();

        // ReqRep processor //
        struct Inc;

        impl Processor<usize, usize> for Inc {
            fn process(&mut self, req: usize) -> reqrep::FutureReply<usize> {
                async move { req + 1 }.boxed()
            }
        }
        // ReqRep processor //

        let timer_buckets = crate::metrics::DurationBuckets::Custom(
            vec![Duration::from_millis(500), Duration::from_millis(1000)],
        ).buckets().unwrap();
        let mut client = ReqRepConfig::new(REQREP_ID, timer_buckets)
            .start_service(Inc, executor.clone())
            .unwrap();
        let task = async {
            let rep_receiver = await!(client.send(1)).unwrap();
            await!(rep_receiver.recv()).unwrap()
        };
        let n = executor.run(task);
        info!("n = {}", n);
        assert_eq!(n, 2);
        info!("{:#?}", crate::metrics::registry().gather());

        assert_eq!(metrics::service_instance_count(REQREP_ID), 1);
        // WHEN: all clients are dropped
        drop(client);
        // THEN: the backend service will stop
        while metrics::service_instance_count(REQREP_ID) != 0 {
            info!(
                "waiting for backend service to stop: {}",
                metrics::service_instance_count(REQREP_ID)
            );
            thread::yield_now();
        }
        info!(
            "service_instance_count({}) = {}",
            REQREP_ID,
            metrics::service_instance_count(REQREP_ID)
        );
    }

    #[test]
    fn req_rep_with_disconnected_receiver() {
        configure_logging();
        const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);
        let mut executor = global_executor();
        let (mut req_rep, req_receiver) = ReqRep::<usize, usize>::new(REQREP_ID, 1);
        let server = async move {
            let mut req_receiver = req_receiver;
            if let Some(mut msg) = await!(req_receiver.next()) {
                let n = msg.take_request().unwrap();
                info!("going to sleep ...");
                thread::sleep_ms(10);
                info!("... awoke");
                if let Err(err) = msg.reply(n + 1) {
                    warn!("{}", err);
                } else {
                    panic!("Should have failed to send reply because the Receiver has been closed");
                }
            }
            info!("message listener has exited");
        };
        let task_handle = executor.spawn_with_handle(server).unwrap();
        let task = async {
            let mut rep_receiver = await!(req_rep.send(1)).unwrap();
            rep_receiver.close();
        };
        executor.run(task);
        executor.run(task_handle);
    }

    #[test]
    fn processor_timer_metrics() {
        use maplit::*;
        // multiple metrics with the same MetricId can be registered as long as they have the same help
        // string and the same label names (aka label dimensions) in each, constLabels and variableLabels,
        // but they must differ in the values of the constLabels.
        for i in 1..=5 {
            crate::metrics::registry()
                .register_histogram(
                    metrics::REQREP_PROCESS_TIMER_METRIC_ID,
                    "ReqRep message processor timer in seconds",
                    crate::metrics::DurationBuckets::Custom(vec![
                        Duration::from_millis(i),
                        Duration::from_millis(i + 1),
                    ])
                    .buckets()
                    .unwrap(),
                    Some(hashmap! {
                        metrics::REQREPID_LABEL_ID => ReqRepId::generate().to_string()
                    }),
                )
                .unwrap();
        }
        crate::metrics::registry()
            .register_histogram(
                metrics::REQREP_PROCESS_TIMER_METRIC_ID,
                "ReqRep message processor timer in seconds",
                crate::metrics::DurationBuckets::Custom(vec![
                    Duration::from_millis(1),
                    Duration::from_millis(2),
                    Duration::from_millis(3),
                ])
                .buckets()
                .unwrap(),
                Some(hashmap! {
                    metrics::REQREPID_LABEL_ID => ReqRepId::generate().to_string()
                }),
            )
            .unwrap();
    }
}
