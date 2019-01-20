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

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use byteorder::{ByteOrder, LittleEndian};
    use nng::{
        aio::{Aio, Context},
        Message, Protocol, Socket,
    };
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    use std::{env, mem, thread};

    /// Number of outstanding requests that we can handle at a given time.
    ///
    /// This is *NOT* the number of threads in use, but instead represents
    /// outstanding work items. Select a small number to reduce memory size. (Each
    /// one of these can be thought of as a request-reply loop.) Note that you will
    /// probably run into limitations on the number of open file descriptors if you
    /// set this too high. (If not for that limit, this could be set in the
    /// thousands, each context consumes a couple of KB.)
    const PARALLEL: usize = 10;

    /// Run the client portion of the program.
    fn client(url: &str, ms: u64) -> Result<(), nng::Error> {
        let mut s = Socket::new(Protocol::Req0)?;
        s.dial(url)?;

        let mut req = Message::zeros(mem::size_of::<u64>())?;
        LittleEndian::write_u64(&mut req, ms);

        let start = Instant::now();
        s.send(req)?;
        s.recv()?;

        let dur = Instant::now().duration_since(start);
        println!("Request took {:?} milliseconds", dur);
        Ok(())
    }

    /// Run the server portion of the program.
    fn server(
        url: &str,
        start: mpsc::Sender<()>,
        shutdown: mpsc::Receiver<()>,
    ) -> Result<(), nng::Error> {
        // Create the socket
        let mut s = Socket::new(Protocol::Rep0)?;

        // Create all of the worker contexts
        let workers: Vec<_> = (0..PARALLEL)
            .map(|i| create_worker(i, &s))
            .collect::<Result<_, _>>()?;

        // Only after we have the workers do we start listening.
        s.listen(url)?;

        // Now start all of the workers listening.
        for (a, c) in &workers {
            a.recv(c)?;
        }

        start
            .send(())
            .expect("Failed to send server started signal");

        // block until server shutdown is signalled
        shutdown.recv();
        println!("server is shutting down ...");

        Ok(())
    }

    /// Create a new worker context for the server.
    fn create_worker(i: usize, s: &Socket) -> Result<(Aio, Context), nng::Error> {
        let mut state = State::Recv;

        let ctx = Context::new(s)?;
        let ctx_clone = ctx.clone();
        let aio = Aio::with_callback(move |aio| worker_callback(i, aio, &ctx_clone, &mut state))?;

        Ok((aio, ctx))
    }

    /// Callback function for workers.
    fn worker_callback(i: usize, aio: &Aio, ctx: &Context, state: &mut State) {
        let new_state = match *state {
            State::Recv => {
                println!("[{}] state: {:?}", i, state);
                // If there was an issue, we're just going to panic instead of
                // doing something sensible.
                let _ = aio.result().unwrap();
                match aio.get_msg() {
                    Some(msg) => {
                        println!("[{}] received message: state: {:?}", i, state);
                        let ms = LittleEndian::read_u64(&msg);

                        aio.sleep(Duration::from_millis(ms)).unwrap();
                        State::Wait
                    }
                    None => {
                        println!("[{}] no message: state: {:?}", i, state);
                        State::Recv
                    }
                }
            }
            State::Wait => {
                println!("[{}] state: {:?}", i, state);
                let msg = Message::new().unwrap();
                aio.send(ctx, msg).unwrap();
                State::Send
            }
            State::Send => {
                println!("[{}] state: {:?}", i, state);
                // Again, just panic bad if things happened.
                let _ = aio.result().unwrap();
                aio.recv(ctx).unwrap();

                State::Recv
            }
        };

        *state = new_state;
    }

    /// State of a request.
    #[derive(Debug, Copy, Clone)]
    enum State {
        Recv,
        Wait,
        Send,
    }

    #[test]
    fn nng_client_server() {
        const url: &str = "inproc://test";

        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let (start_tx, start_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            server(url, start_tx, shutdown_rx).expect("Failed to start the server");
        });

        start_rx.recv();
        client(url, 10);
        client(url, 5);
        client(url, 0);
        shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
        server.join().unwrap();
    }
}
