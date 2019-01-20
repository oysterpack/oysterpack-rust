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

#[cfg(test)]
mod tests {
    use futures::{future::Future, stream::Stream};
    use runng::{
        protocol::{AsyncReply, AsyncRequest, AsyncSocket},
        *,
    };

    fn aio() -> NngReturn {
        const url: &str = "inproc://test";

        let factory = Latest::default();
        let mut rep_ctx = factory
            .replier_open()?
            .listen(&url)?
            .create_async_context()?;

        let requester = factory.requester_open()?.dial(&url)?;
        let mut req_ctx = requester.create_async_context()?;
        let req_future = req_ctx.send(msg::NngMsg::new()?);
        rep_ctx
            .receive()
            .take(1)
            .for_each(|_request| {
                let msg = msg::NngMsg::new().unwrap();
                let _ = rep_ctx.reply(msg).wait().unwrap().unwrap();
                Ok(())
            })
            .wait()
            .unwrap();
        let result = req_future.wait().unwrap()?;
        println!("{:?}", result);

        Ok(())
    }

    #[test]
    fn runng_poc() {
        println!("{:?}", aio().unwrap());
    }
}
