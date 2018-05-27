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

//! OysterPack Asynchronous Commands

//#![deny(missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_service/0.1.0")]

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate semver;
extern crate serde;
extern crate tokio;
extern crate tokio_service;

extern crate oysterpack_id as id;
extern crate oysterpack_platform as platform;

//use futures::prelude::*;

/// A Command represents a generic asynchronous function invocation that may fail.
///
/// Commands are lazy. This means commands assemble the unit of work
///
/// The Command trait is a simplified interface making it easy to write applications and components in a modular and reusable way.
/// It is one of OysterPack's fundamental abstractions.
///
//pub trait Command {
//    /// Request type
//    type Request : Send;
//    /// Response type
//    type Response : Send;
//    /// Error type
//    type Error: failure::Fail + Send;
//
//    type Future: Future<Item = Self::Response, Error = Self::Error> + Send;
//
//    /// Returns a new lazy Command future. In order to execute the command, i.e., produce the result,  the
//    /// Future must be spawned by a futures::executor::Executor.
//    fn call(&self, request: Self::Request) -> Self::Future;
//}

///// CommandBuilder
//pub trait CommandBuilder {
//    /// Command Request type
//    type Request : Send;
//    /// Command Response type
//    type Response :  Send;
//    /// Command Error type
//    type Error: failure::Fail + Send;
//    /// Command instance type
//    type Instance: Command<Request = Self::Request, Response = Self::Response, Error = Self::Error>;
//
//    /// Builder Error type
//    type BuilderError: failure::Fail;
//
//    /// Builds the Command and returns the build Result.
//    fn build(&self) -> Result<Self::Instance, Self::BuilderError>;
//}
#[cfg(test)]
mod tests {
    use tokio::prelude::future::ok;
    use tokio::prelude::*;
    use tokio::timer::*;

    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    use tokio_service::Service;

    //    use futures::channel::oneshot;
    use super::*;

    //    #[test]
    //    fn simple_command() {
    //
    //        #[allow(missing_docs,warnings)]
    //        struct Multiply;
    //
    //        #[allow(missing_docs)]
    //        impl Command for Multiply {
    //            type Request = (i64, i64);
    //            type Response = i64;
    //            type Error = NeverFails;
    //            type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send>;
    //
    //            fn call(&self, request: Self::Request) -> Self::Future {
    //                Box::new(futures::future::lazy(move |_| {
    //                    ok(request.0 * request.1)
    //                }))
    //            }
    //        }
    //
    //        let future = Multiply.call((2, 3)).and_then(|n| ok(n*n));
    //
    //        let join_handle = block_on(spawn_with_handle(future)).unwrap();
    //        let result = block_on(join_handle);
    //        assert_eq!(result, Ok(36));
    //    }

    #[test]
    fn closure_as_aservice() {
        #[allow(missing_docs, warnings)]
        struct Multiply;

        #[allow(missing_docs)]
        impl Service for Multiply {
            type Request = (i64, i64);
            type Response = i64;
            type Error = i64;
            type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send>;

            fn call(&self, request: Self::Request) -> Self::Future {
                Box::new(future::lazy(move || future::ok(request.0 * request.1)))
            }
        }

        tokio::run(
            Multiply
                .call((2, 3))
                .and_then(|n| {
                    println!("n = {}", n);
                    ok(())
                })
                .map_err(|_| ()),
        );
    }

    #[test]
    fn starving() {
        use futures::{task, Async, Poll};

        struct Starve(Delay, u64);

        impl Future for Starve {
            type Item = u64;
            type Error = ();

            fn poll(&mut self) -> Poll<Self::Item, ()> {
                if self.0.poll().unwrap().is_ready() {
                    return Ok(self.1.into());
                }

                self.1 += 1;

                task::current().notify();

                Ok(Async::NotReady)
            }
        }

        let when = Instant::now() + Duration::from_millis(20);
        let starve = Starve(Delay::new(when), 0);

        let (tx, rx) = mpsc::channel();

        tokio::run({
            starve.and_then(move |_ticks| {
                assert!(Instant::now() >= when);
                tx.send(()).unwrap();
                Ok(())
            })
        });

        rx.recv().unwrap();
    }
}
