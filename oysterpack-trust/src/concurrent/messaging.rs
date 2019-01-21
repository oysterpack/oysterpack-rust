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

//! Provides support for Future compatible messaging

use failure::Fail;
use futures::{channel, sink::SinkExt};
use std::fmt::Debug;

/// Implements a request/reply messaging pattern
#[derive(Debug, Clone)]
pub struct ReqRep<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    request_sender: channel::mpsc::Sender<ReqRepMessage<Req, Rep>>,
}

impl<Req, Rep> ReqRep<Req, Rep>
where
    Req: Debug + Send + 'static,
    Rep: Debug + Send + 'static,
{
    /// Send the request
    pub async fn send(
        &mut self,
        req: Req,
    ) -> Result<channel::oneshot::Receiver<Rep>, ChannelSendError> {
        let (rep_sender, rep_receiver) = channel::oneshot::channel::<Rep>();
        let msg = ReqRepMessage {
            req: Some(req),
            rep_sender,
        };
        await!(self.request_sender.send(msg))?;
        Ok(rep_receiver)
    }

    /// constructor
    pub fn new(
        chan_buf_size: usize,
    ) -> (
        ReqRep<Req, Rep>,
        channel::mpsc::Receiver<ReqRepMessage<Req, Rep>>,
    ) {
        let (request_sender, request_receiver) = channel::mpsc::channel(chan_buf_size);
        (ReqRep { request_sender }, request_receiver)
    }
}

/// Message used for request/reply patterns.
#[derive(Debug)]
pub struct ReqRepMessage<Req, Rep>
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
    pub fn take_request(&mut self) -> Option<Req> {
        self.req.take()
    }

    /// Send the reply
    pub fn reply(self, rep: Rep) -> Result<(), ChannelSendError> {
        self.rep_sender
            .send(rep)
            .map_err(|_| ChannelSendError::Disconnected)
    }
}

/// Channel sending related errors
#[derive(Fail, Debug)]
pub enum ChannelSendError {
    /// Failed to send because the channel is full
    #[fail(display = "Failed to send message because the channel is full")]
    Full,
    /// Failed to send because the channel is disconnected
    #[fail(display = "Failed to send message because the channel is disconnected")]
    Disconnected,
}

impl From<channel::mpsc::SendError> for ChannelSendError {
    fn from(err: channel::mpsc::SendError) -> Self {
        if err.is_disconnected() {
            return ChannelSendError::Disconnected;
        }
        ChannelSendError::Full
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrent::execution::EXECUTORS;
    use crate::log_config;
    use futures::{
        channel::oneshot,
        stream::StreamExt,
        task::{Spawn, SpawnExt},
    };
    use oysterpack_log::*;
    use std::thread;

    #[test]
    fn try_recv_after_already_received_on_oneshot_channel() {
        let (p, mut c) = oneshot::channel();
        p.send(1);
        assert_eq!(c.try_recv().unwrap().unwrap(), 1);
        // trying to receive a message after it already received a message results in an error
        // at this point the Receiver is cancelled
        assert!(c.try_recv().is_err());
    }

    #[test]
    fn req_rep() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
        let executors = EXECUTORS.lock().unwrap();
        let mut executor = executors.global_executor();
        let (mut req_rep, req_receiver) = ReqRep::<usize, usize>::new(1);
        let server = async move {
            let mut req_receiver = req_receiver;
            while let Some(mut msg) = await!(req_receiver.next()) {
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
            await!(rep_receiver).unwrap()
        };
        let n = executor.run(task);
        info!("n = {}", n);
        assert_eq!(n, 2);
    }

    #[test]
    fn req_rep_with_disconnected_receiver() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
        let executors = EXECUTORS.lock().unwrap();
        let mut executor = executors.global_executor();
        let (mut req_rep, req_receiver) = ReqRep::<usize, usize>::new(1);
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

}
