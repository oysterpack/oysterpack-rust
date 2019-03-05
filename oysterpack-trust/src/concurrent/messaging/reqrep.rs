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
//! let timer_buckets = metrics::TimerBuckets::from(vec![
//!     Duration::from_millis(500),
//!     Duration::from_millis(1000)]
//! );
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
//!
//! ## Config
//! - [ReqRepConfig](struct.ReqRepConfig.html)
//!
//! ## Metrics
//! - number of running backend service instances
//! - request processing timings, i.e., [Processor::process()](trait.Processor.html#tymethod.process) is timed

use crate::concurrent::{execution::Executor, messaging::errors::ChannelError};
use crate::metrics;
use futures::{
    channel,
    prelude::*,
    task::{SpawnError, SpawnExt},
};
use maplit::hashmap;
use oysterpack_log::*;
use oysterpack_uid::macros::ulid;
use oysterpack_uid::ULID;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    pin::Pin,
    sync::RwLock,
    time::Instant,
};

lazy_static::lazy_static! {
    static ref REQ_REP_METRICS: RwLock<fnv::FnvHashMap<ReqRepId, ReqRepServiceMetrics>> = RwLock::new(fnv::FnvHashMap::default());

    static ref REQ_REP_SERVICE_INSTANCE_COUNT: prometheus::IntGaugeVec = metrics::registry().register_int_gauge_vec(
        SERVICE_INSTANCE_COUNT_METRIC_ID,
        "ReqRep service instance count",
        &[REQREPID_LABEL_ID],
        None,
    ).unwrap();

    static ref REQREP_SEND_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        REQREP_SEND_COUNTER_METRIC_ID,
        "ReqRep request send count",
        &[REQREPID_LABEL_ID],
        None,
    ).unwrap();
}

/// ReqRep service instance count MetricId: `M01D2Q7VG1HFFXG6JT6HD11ZCJ3`
/// - metric type is IntGaugeVec
pub const SERVICE_INSTANCE_COUNT_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1872765971344832352273831154704953923);

/// ReqRep backend message processing timer MetricId: `M01D4ZMEFGBPCK2HSNAPGBARR14`
/// - metric type is Histogram
pub const REQREP_PROCESS_TIMER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1875702602137856142367281339226152996);

/// The ReqRepId ULID will be used as the label value: `L01D2Q81HQJJVPQZSQE7BHH67JK`
pub const REQREPID_LABEL_ID: metrics::LabelId =
    metrics::LabelId(1872766211119679891800112881745469011);

/// ReqRep request send counter MetricId: `M01D52BD1MYW4GJ2VN44S14R28Z`
/// - metric type is Histogram
pub const REQREP_SEND_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1875812830972763422767373669165173023);

/// return the ReqRep backend service count
pub fn service_instance_count(reqrep_id: ReqRepId) -> u64 {
    let label_name = REQREPID_LABEL_ID.name();
    let label_value = reqrep_id.to_string();
    metrics::registry()
        .gather_for_desc_names(&[SERVICE_INSTANCE_COUNT_METRIC_ID.name().as_str()])
        .iter()
        .filter_map(|mf| {
            mf.get_metric()
                .iter()
                .find(|metric| {
                    metric.get_label().iter().any(|label_pair| {
                        label_pair.get_name() == label_name && label_pair.get_value() == label_value
                    })
                })
                .map(|mf| mf.get_gauge().get_value() as u64)
        })
        .next()
        .unwrap_or(0)
}

/// return the ReqRep backend service count
pub fn service_instance_counts() -> fnv::FnvHashMap<ReqRepId, u64> {
    metrics::registry()
        .gather_for_desc_names(&[SERVICE_INSTANCE_COUNT_METRIC_ID.name().as_str()])
        .first()
        .map(|mf| {
            let label_name = REQREPID_LABEL_ID.name();
            let label_name = label_name.as_str();
            let metrics = mf.get_metric();
            let counts = fnv::FnvHashMap::with_capacity_and_hasher(
                metrics.len(),
                fnv::FnvBuildHasher::default(),
            );
            metrics.iter().fold(counts, |mut counts, metric| {
                let reqrep_id = metric
                    .get_label()
                    .iter()
                    .find_map(|label_pair| {
                        if label_pair.get_name() == label_name {
                            Some(ReqRepId::from(
                                label_pair.get_value().parse::<ULID>().unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let gauge = metric.get_gauge();
                counts.insert(reqrep_id, gauge.get_value() as u64);
                counts
            })
        })
        .unwrap_or_else(fnv::FnvHashMap::default)
}

/// return the ReqRep request send count
pub fn request_send_count(reqrep_id: ReqRepId) -> u64 {
    let label_name = REQREPID_LABEL_ID.name();
    let label_value = reqrep_id.to_string();
    metrics::registry()
        .gather_for_desc_names(&[REQREP_SEND_COUNTER_METRIC_ID.name().as_str()])
        .iter()
        .filter_map(|mf| {
            mf.get_metric()
                .iter()
                .find(|metric| {
                    metric.get_label().iter().any(|label_pair| {
                        label_pair.get_name() == label_name && label_pair.get_value() == label_value
                    })
                })
                .map(|mf| mf.get_counter().get_value() as u64)
        })
        .next()
        .unwrap_or(0)
}

/// return the ReqRep backend service count
pub fn request_send_counts() -> fnv::FnvHashMap<ReqRepId, u64> {
    metrics::registry()
        .gather_for_desc_names(&[REQREP_SEND_COUNTER_METRIC_ID.name().as_str()])
        .first()
        .map(|mf| {
            let label_name = REQREPID_LABEL_ID.name();
            let label_name = label_name.as_str();
            let metrics = mf.get_metric();
            let counts = fnv::FnvHashMap::with_capacity_and_hasher(
                metrics.len(),
                fnv::FnvBuildHasher::default(),
            );
            metrics.iter().fold(counts, |mut counts, metric| {
                let reqrep_id = metric
                    .get_label()
                    .iter()
                    .find_map(|label_pair| {
                        if label_pair.get_name() == label_name {
                            Some(ReqRepId::from(
                                label_pair.get_value().parse::<ULID>().unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let counter = metric.get_counter();
                counts.insert(reqrep_id, counter.get_value() as u64);
                counts
            })
        })
        .unwrap_or_else(fnv::FnvHashMap::default)
}

/// Gathers metrics related to ReqRep
pub fn gather_metrics() -> Vec<prometheus::proto::MetricFamily> {
    metrics::registry().gather_for_metric_ids(&[
        SERVICE_INSTANCE_COUNT_METRIC_ID,
        REQREP_PROCESS_TIMER_METRIC_ID,
        REQREP_SEND_COUNTER_METRIC_ID,
    ])
}

/// ReqRep is used to configure and start a ReqRep service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqRepConfig {
    reqrep_id: ReqRepId,
    chan_buf_size: usize,
    metric_timer_buckets: metrics::TimerBuckets,
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
    pub fn metric_timer_buckets(&self) -> &metrics::TimerBuckets {
        &self.metric_timer_buckets
    }

    /// constructor
    /// - the chan_buf_size default = 1
    /// - the TimerBuckets should be based on expected response times
    pub fn new(reqrep_id: ReqRepId, metric_timer_buckets: metrics::TimerBuckets) -> Self {
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
    service_instance_id: ServiceInstanceId,
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

    /// Returns the backend ServiceInstanceId
    /// - this can be used to determine if 2 different ReqRep client instances are talking to the same
    ///   backend instance
    pub fn service_instance_id(&self) -> ServiceInstanceId {
        self.service_instance_id
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
        service_instance_id: ServiceInstanceId,
    ) -> (
        ReqRep<Req, Rep>,
        channel::mpsc::Receiver<ReqRepMessage<Req, Rep>>,
    ) {
        let (request_sender, request_receiver) = channel::mpsc::channel(chan_buf_size);
        (
            ReqRep {
                reqrep_id,
                request_sender,
                service_instance_id,
                request_send_counter: REQREP_SEND_COUNTER
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
        mut processor: Service,
        mut executor: Executor,
        metric_timer_buckets: metrics::TimerBuckets,
    ) -> Result<ReqRep<Req, Rep>, SpawnError>
    where
        Service: Processor<Req, Rep> + Send + 'static,
    {
        let reqrep_service_metrics = move || {
            let mut reqrep_metrics = REQ_REP_METRICS.write().unwrap();
            reqrep_metrics
                .entry(reqrep_id)
                .or_insert_with(|| {
                    let timer = metrics::registry()
                        .register_histogram_timer(
                            REQREP_PROCESS_TIMER_METRIC_ID,
                            "ReqRep message processor timer in seconds",
                            metric_timer_buckets,
                            Some(hashmap! {
                                REQREPID_LABEL_ID => reqrep_id.to_string()
                            }),
                        )
                        .unwrap();
                    let service_count = REQ_REP_SERVICE_INSTANCE_COUNT
                        .with_label_values(&[reqrep_id.to_string().as_str()]);

                    ReqRepServiceMetrics {
                        timer,
                        service_count,
                    }
                })
                .clone()
        };

        let service_instance_id = ServiceInstanceId::generate();
        let (reqrep, mut req_receiver) =
            ReqRep::<Req, Rep>::new(reqrep_id, chan_buf_size, service_instance_id);
        let reqrep_service_metrics = reqrep_service_metrics();

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
                let rep = await!(processor.process(req));
                let elapsed = start.elapsed();

                // send back the reply
                // we don't care if the client reply channel is disconnected
                let _ = msg.reply(rep);

                // record the timing metric
                reqrep_service_metrics
                    .timer
                    .observe(metrics::duration_as_secs_f64(elapsed));
                debug!(
                    "[{}] ReqRepId({}) {:?} #{} : {:?}",
                    service_instance_id, reqrep_id, service_type_id, request_count, elapsed
                );
            }
            reqrep_service_metrics.service_count.dec();
            processor.destroy();
        };

        executor.spawn(service)?;
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
            .field("service_instance_id", &self.service_instance_id)
            .field("request_send_count", &self.request_send_counter.get())
            .finish()
    }
}

/// ReqRep service metrics
#[derive(Clone)]
struct ReqRepServiceMetrics {
    timer: prometheus::Histogram,
    service_count: prometheus::IntGauge,
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

/// Each new ReqRep backend service is assigned a unique id
#[ulid]
pub struct ServiceInstanceId(u128);

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
}

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
        let service_instance_id = ServiceInstanceId::generate();
        let mut executor = global_executor();
        let (mut req_rep, mut req_receiver) =
            ReqRep::<usize, usize>::new(REQREP_ID, 1, service_instance_id);
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

        let timer_buckets = metrics::TimerBuckets::from(
            vec![Duration::from_millis(500), Duration::from_millis(1000)].as_slice(),
        );

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
        info!("{:#?}", metrics::registry().gather());

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

        let timer_buckets = metrics::TimerBuckets::from(
            vec![Duration::from_millis(500), Duration::from_millis(1000)].as_slice(),
        );
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
        info!("{:#?}", metrics::registry().gather());

        assert_eq!(service_instance_count(REQREP_ID), 1);
        // WHEN: all clients are dropped
        drop(client);
        // THEN: the backend service will stop
        while service_instance_count(REQREP_ID) != 0 {
            info!(
                "waiting for backend service to stop: {}",
                service_instance_count(REQREP_ID)
            );
            thread::yield_now();
        }
        info!(
            "service_instance_count({}) = {}",
            REQREP_ID,
            service_instance_count(REQREP_ID)
        );
    }

    #[test]
    fn req_rep_with_disconnected_receiver() {
        configure_logging();
        const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);
        let service_instance_id = ServiceInstanceId::generate();
        let mut executor = global_executor();
        let (mut req_rep, req_receiver) =
            ReqRep::<usize, usize>::new(REQREP_ID, 1, service_instance_id);
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
        for i in 0..5 {
            metrics::registry()
                .register_histogram_timer(
                    super::REQREP_PROCESS_TIMER_METRIC_ID,
                    "ReqRep message processor timer in seconds",
                    metrics::TimerBuckets::from(vec![
                        Duration::from_millis(i),
                        Duration::from_millis(i + 1),
                    ]),
                    Some(hashmap! {
                        REQREPID_LABEL_ID => ReqRepId::generate().to_string()
                    }),
                )
                .unwrap();
        }
        metrics::registry()
            .register_histogram_timer(
                super::REQREP_PROCESS_TIMER_METRIC_ID,
                "ReqRep message processor timer in seconds",
                metrics::TimerBuckets::from(vec![
                    Duration::from_millis(1),
                    Duration::from_millis(2),
                    Duration::from_millis(3),
                ]),
                Some(hashmap! {
                    REQREPID_LABEL_ID => ReqRepId::generate().to_string()
                }),
            )
            .unwrap();
    }
}
