/*
 * Copyright 2018 OysterPack Inc.
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

//! Message Broker Actor Service

use crate::message;
use exonum_sodiumoxide::crypto::box_;
use futures::prelude::*;
use oysterpack_errors::Error;
use std::collections::HashMap;
use std::fmt;

// TODO: provide integration with https://docs.rs/async-bincode/0.4.9/async_bincode
// TODO: schedule a periodic job to clear precomputed keys that have not been used in a while
// TODO: metrics per message type

// TODO: support for replay protection based on the message's InstanceID timestamp - "old" messages are rejected
// TODO: a message is considered "old" if its timestamp is older than the most recent message processed within the client session
// TODO: support for seqential processing based on the message sequence - could be strict or loose

// TODO: support for signed addresses - a request is not accepted unless the address signature is verified

// TODO: all message types must have a deadline to ensure server resources are protected (and potentially from attack)
// TODO: message types must have a max deadline configured, to protect against attacks

/// Messaging actor service
/// - is a sync actor because it needs to perform CPU bound load for cryptography and compression
/// - the service is assigned a public-key based address
pub struct MessageService {
    address: message::Address,
    private_key: box_::SecretKey,
    // sender -> precomputed key
    precomputed_keys: HashMap<message::Address, box_::PrecomputedKey>,
    message_handlers: HashMap<message::MessageType, actix::Recipient<Request>>,
}

impl MessageService {
    /// constructor
    pub fn new(address: message::Address, private_key: box_::SecretKey) -> MessageService {
        MessageService {
            address,
            private_key,
            precomputed_keys: HashMap::new(),
            message_handlers: HashMap::new(),
        }
    }
}

impl fmt::Debug for MessageService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keys: Vec<&message::MessageType> = self.message_handlers.keys().collect();
        write!(f, "MessageService(message_type:{:?})", keys)
    }
}

impl actix::Actor for MessageService {
    type Context = actix::Context<Self>;
}

/// Used to register a message handler
#[derive(Clone)]
pub struct RegisterMessageHandler {
    message_type: message::MessageType,
    handler: actix::Recipient<Request>,
}

impl RegisterMessageHandler {
    /// constructor
    pub fn new(
        message_type: message::MessageType,
        handler: actix::Recipient<Request>,
    ) -> RegisterMessageHandler {
        RegisterMessageHandler {
            message_type,
            handler,
        }
    }
}

impl fmt::Debug for RegisterMessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RegisterMessageHandler({:?})", self.message_type)
    }
}

impl actix::Message for RegisterMessageHandler {
    type Result = ();
}

/// Message Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request(pub message::EncodedMessage);

impl actix::Message for Request {
    type Result = Result<message::EncodedMessage, Error>;
}

impl actix::Handler<RegisterMessageHandler> for MessageService {
    type Result = actix::MessageResult<RegisterMessageHandler>;

    fn handle(&mut self, msg: RegisterMessageHandler, _: &mut Self::Context) -> Self::Result {
        self.message_handlers.insert(msg.message_type, msg.handler);
        actix::MessageResult(())
    }
}

/// Get the list of registered message types
#[derive(Debug, Copy, Clone)]
pub struct GetRegisteredMessageTypes;

impl actix::Message for GetRegisteredMessageTypes {
    type Result = Vec<message::MessageType>;
}

impl actix::Handler<GetRegisteredMessageTypes> for MessageService {
    type Result = actix::MessageResult<GetRegisteredMessageTypes>;

    fn handle(&mut self, _: GetRegisteredMessageTypes, _: &mut Self::Context) -> Self::Result {
        let message_types: Vec<message::MessageType> =
            self.message_handlers.keys().cloned().collect();
        actix::MessageResult(message_types)
    }
}

/// Message that indicates that a client has disconnected.
/// -  the server should clean up any client related resources
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ClientDisconnect(message::Address);

impl actix::Message for ClientDisconnect {
    type Result = ();
}

impl actix::Handler<ClientDisconnect> for MessageService {
    type Result = actix::MessageResult<ClientDisconnect>;

    fn handle(&mut self, msg: ClientDisconnect, _: &mut Self::Context) -> Self::Result {
        self.precomputed_keys.remove(&msg.0);
        actix::MessageResult(())
    }
}

// TODO: add an optional token, which is used to pay for the request
/// Process the SealedEnvelope request
#[derive(Debug, Clone)]
pub struct SealedEnvelopeRequest(pub message::SealedEnvelope);

impl actix::Message for SealedEnvelopeRequest {
    type Result = Result<message::SealedEnvelope, Error>;
}

impl actix::Handler<SealedEnvelopeRequest> for MessageService {
    type Result = actix::Response<message::SealedEnvelope, Error>;

    fn handle(&mut self, req: SealedEnvelopeRequest, _: &mut Self::Context) -> Self::Result {
        let key = {
            let private_key = &self.private_key;
            self.precomputed_keys
                .entry(*req.0.sender())
                .or_insert_with(|| box_::precompute(req.0.sender().public_key(), private_key))
                .clone()
        };

        fn process_message(
            sender: message::Address,
            handler: &actix::Recipient<Request>,
            encoded_message: message::EncodedMessage,
            key: box_::PrecomputedKey,
        ) -> Box<dyn Future<Item = message::SealedEnvelope, Error = Error>> {
            let msg_type = encoded_message.metadata().message_type();

            let send = {
                if let Some(deadline) = encoded_message.metadata().deadline() {
                    let duration = deadline
                        .duration(encoded_message.metadata().instance_id().ulid().datetime())
                        .to_std()
                        .or(Ok(std::time::Duration::from_millis(0))
                            as Result<std::time::Duration, ()>)
                        .unwrap();
                    handler.send(Request(encoded_message)).timeout(duration)
                } else {
                    handler.send(Request(encoded_message))
                }
            };
            let fut = send
                .map_err(move |err| {
                    op_error!(errors::MailboxDeliveryError::new(&sender, msg_type, err))
                })
                .and_then(|result| {
                    let result = match result {
                        Ok(encoded_message) => encoded_message.open_envelope(),
                        Err(e) => Err(e),
                    };
                    futures::future::result(result)
                })
                .and_then(move |result| futures::future::ok(result.seal(&key)));
            Box::new(fut)
        }

        fn unsupported_message_type(
            sender: message::Address,
            msg_type: message::MessageType,
        ) -> Box<dyn Future<Item = message::SealedEnvelope, Error = Error>> {
            let fut = futures::future::err(op_error!(errors::UnsupportedMessageType::new(
                &sender, msg_type
            )));
            Box::new(fut)
        }

        fn message_error(
            err: Error,
        ) -> Box<dyn Future<Item = message::SealedEnvelope, Error = Error>> {
            Box::new(futures::future::err(err))
        }

        let sender = *req.0.sender();
        let result = req
            .0
            .open(&key)
            .and_then(|open_envelope| open_envelope.encoded_message())
            .and_then(|encoded_message| {
                let message_type = encoded_message.metadata().message_type();
                let fut = match self.message_handlers.get(&message_type) {
                    Some(handler) => process_message(sender, handler, encoded_message, key),
                    None => {
                        unsupported_message_type(sender, encoded_message.metadata().message_type())
                    }
                };
                Ok(fut)
            });

        match result {
            Ok(fut) => actix::Response::r#async(fut),
            Err(err) => actix::Response::r#async(message_error(err)),
        }
    }
}

/// MessageService errors
pub mod errors {
    use crate::message;
    use oysterpack_errors::{Id, IsError, Level};
    use std::fmt;

    /// UnsupportedMessageType
    #[derive(Debug)]
    pub struct UnsupportedMessageType<'a> {
        sender: &'a message::Address,
        message_type: message::MessageType,
    }

    impl UnsupportedMessageType<'_> {
        /// Error Id(01CYQ74Q46EAAPHBD95NJAXJGG)
        pub const ERROR_ID: Id = Id(1867572772130723204709592385635404304);
        /// Level::Alert because receiving a message for a type we do not support should be investigated
        /// - this could be an attack
        /// - this could be an app config issue
        /// - this could be a client config issue - the client should be notified
        pub const ERROR_LEVEL: Level = Level::Alert;

        /// constructor
        pub fn new(
            sender: &message::Address,
            message_type: message::MessageType,
        ) -> UnsupportedMessageType {
            UnsupportedMessageType {
                sender,
                message_type,
            }
        }
    }

    impl IsError for UnsupportedMessageType<'_> {
        fn error_id(&self) -> Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for UnsupportedMessageType<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}: unsupported message type ({})",
                self.sender, self.message_type
            )
        }
    }

    /// MailboxDeliveryError
    #[derive(Debug)]
    pub struct MailboxDeliveryError<'a> {
        sender: &'a message::Address,
        message_type: message::MessageType,
        err: actix::MailboxError,
    }

    impl MailboxDeliveryError<'_> {
        /// Error Id(01CYQ8F5HQMW5PETXBQAF3KF79)
        pub const ERROR_ID: Id = Id(1867574453777009173831417913650298089);
        /// Level::Critical
        /// - if messages are timing out, then this is higher priority
        ///   - it could mean performance degradation issues
        ///   - it could mean clients are submitting requests with timeouts that are too low
        /// - if the mailbox is closed, then this could mean a bug if it occurrs while the app is
        ///   running, i.e., not shutting down
        ///   - because of timing issues, the mailbox may be closed during application shutdown
        pub const ERROR_LEVEL: Level = Level::Critical;

        /// constructor
        pub fn new(
            sender: &message::Address,
            message_type: message::MessageType,
            err: actix::MailboxError,
        ) -> MailboxDeliveryError {
            MailboxDeliveryError {
                sender,
                message_type,
                err,
            }
        }
    }

    impl IsError for MailboxDeliveryError<'_> {
        fn error_id(&self) -> Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for MailboxDeliveryError<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}: mailbox delivery error for message type: {} : {}",
                self.sender, self.message_type, self.err,
            )
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use crate::actor;
    use exonum_sodiumoxide::crypto::box_;
    use futures::prelude::*;

    struct EchoService;

    impl actix::Actor for EchoService {
        type Context = actix::Context<Self>;
    }

    impl actix::Handler<super::Request> for EchoService {
        type Result = actix::MessageResult<super::Request>;

        fn handle(&mut self, req: super::Request, _: &mut Self::Context) -> Self::Result {
            actix::MessageResult(Ok(req.0))
        }
    }

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
    }

    const MESSAGE_SERVICE: actor::arbiters::Name = actor::arbiters::Name("MESSAGE_SERVICE");

    #[test]
    fn message_service() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let addresses =
            crate::message::Addresses::new(client_pub_key.into(), server_pub_key.into());
        let server_address = addresses.recipient().clone();

        use crate::message::IsMessage;
        #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
        struct Foo(String);
        impl IsMessage for Foo {
            const MESSAGE_TYPE_ID: crate::message::MessageTypeId =
                crate::message::MessageTypeId(1867384532653698871582487715619812439);
        }

        let sealed_envelope = {
            let metadata = crate::message::Metadata::new(
                Foo::MESSAGE_TYPE_ID.message_type(),
                crate::message::Encoding::Bincode(Some(crate::message::Compression::Snappy)),
                Some(crate::message::Deadline::ProcessingTimeoutMillis(10)),
            );
            let msg = crate::message::Message::new(
                metadata,
                Foo("cryptocurrency is changing the world through decentralization".to_string()),
            );
            let msg = msg
                .encoded_message(addresses.sender().clone(), addresses.recipient().clone())
                .unwrap();
            let msg = msg.open_envelope().unwrap();
            let msg = msg.seal(
                &addresses
                    .recipient()
                    .precompute_sealing_key(&client_priv_key),
            );
            msg
        };

        struct FooActor;

        impl actix::Actor for FooActor {
            type Context = actix::Context<Self>;
        }

        impl actix::Handler<super::Request> for FooActor {
            type Result = actix::MessageResult<super::Request>;

            fn handle(&mut self, req: super::Request, _: &mut Self::Context) -> Self::Result {
                actix::MessageResult(Ok(req.0))
            }
        }

        const FOO: crate::actor::arbiters::Name = crate::actor::arbiters::Name("FOO");

        actor::app::App::run(
            crate::build::get(),
            log_config(),
            futures::future::lazy(move || {
                actor::arbiters::start_actor(MESSAGE_SERVICE, move |_| {
                    super::MessageService::new(server_address, server_priv_key)
                })
                .and_then(|addr| {
                    let register_foo_actor = crate::actor::arbiters::start_actor(FOO, |_| FooActor);
                    let register_message_type = {
                        let addr = addr.clone();
                        register_foo_actor.and_then(move |foo| {
                            let foo = foo.recipient();
                            addr.send(super::RegisterMessageHandler::new(
                                Foo::MESSAGE_TYPE_ID.message_type(),
                                foo,
                            ))
                        })
                    };
                    register_message_type
                        .and_then(move |_| addr.send(super::SealedEnvelopeRequest(sealed_envelope)))
                })
                .then(|result| {
                    let result = result.unwrap();
                    match result {
                        Ok(msg) => info!("result: {}", msg),
                        Err(e) => panic!("{}", e),
                    }
                    futures::future::ok::<(), ()>(())
                })
            }),
        );
    }
}
