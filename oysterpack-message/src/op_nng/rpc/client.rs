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

//! nng RPC client

/// nng RPC client
#[derive(Debug)]
pub struct Client {}

impl Client {}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::op_nng::rpc::{server::*, MessageProcessor, MessageProcessorFactory};
    use log::*;
    use oysterpack_uid::ULID;
    use serde::{Deserialize, Serialize};
    use std::{num::NonZeroUsize, sync::Arc, thread};

    #[derive(Debug, Clone, Default)]
    struct TestProcessor;

    impl MessageProcessorFactory<TestProcessor, nng::Message, nng::Message> for TestProcessor {
        fn new(&self) -> TestProcessor {
            TestProcessor
        }
    }

    impl MessageProcessor<nng::Message, nng::Message> for TestProcessor {
        fn process(&mut self, req: nng::Message) -> nng::Message {
            match bincode::deserialize::<Request>(&*req.body()).unwrap() {
                Request::Sleep(sleep_ms) if sleep_ms > 0 => {
                    info!(
                        "handler({:?}) sleeping for {} ms ...",
                        thread::current().id(),
                        sleep_ms
                    );
                    thread::sleep_ms(sleep_ms);
                    info!("handler({:?}) has awaken !!!", thread::current().id());
                }
                Request::Sleep(_) => {
                    info!("received Sleep message on {:?}", thread::current().id())
                }
                Request::Panic(msg) => {
                    error!("received Panic message on {:?}", thread::current().id());
                    panic!(msg)
                }
            }
            req
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    enum Request {
        Sleep(u32),
        Panic(String),
    }

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
    }

    #[test]
    fn sync_client() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
        let url = Arc::new(format!("inproc://{}", ULID::generate()));

        // start a server with 2 aio contexts
        let listener_settings =
            ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
        let server = Server::builder(listener_settings, TestProcessor)
            .spawn()
            .unwrap();

        // TODO

        server.stop();
        server.join();
    }

}
