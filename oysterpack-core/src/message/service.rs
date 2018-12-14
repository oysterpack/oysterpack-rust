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

use std::collections::HashMap;
use crate::message;
use oysterpack_errors::Error;
use std::fmt;
use futures::prelude::*;
use exonum_sodiumoxide::crypto::box_;

/// Messaging actor service
/// - is a sync actor because it needs to perform CPU bound load for cryptography and compression
/// - the service is assigned a public-key based address
pub struct MessageService {
    address: message::Address,
    private_key: box_::SecretKey,
    precomputed_keys: HashMap<message::Address, box_::PrecomputedKey>,
    message_handlers: HashMap<message::MessageType, actix::Recipient<Request>>
}

impl MessageService {

    /// constructor
    pub fn new(address: message::Address, private_key: box_::SecretKey, precomputed_keys: HashMap<message::Address, box_::PrecomputedKey>,) -> MessageService {
        MessageService {
            address,
            private_key,
            precomputed_keys: precomputed_keys,
            message_handlers: HashMap::new()
        }
    }
}

impl fmt::Debug for MessageService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keys: Vec<&message::MessageType> = self.message_handlers.keys().collect();
        write!(f,"MessageService(message_type:{:?})", keys)
    }
}

impl actix::Actor for MessageService {
    type Context = actix::Context<Self>;
}

/// Used to register a message handler
#[derive(Clone)]
pub struct RegisterMessageHandler {
    message_type: message::MessageType,
    handler: actix::Recipient<Request>
}

impl fmt::Debug for RegisterMessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"RegisterMessageHandler({:?})", self.message_type)
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

    fn handle(&mut self, msg: RegisterMessageHandler, _ : &mut Self::Context) -> Self::Result {
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

    fn handle(&mut self, _: GetRegisteredMessageTypes, _ : &mut Self::Context) -> Self::Result {
        let message_types: Vec<message::MessageType> = self.message_handlers.keys().map(|key| *key).collect();
        actix::MessageResult(message_types)
    }
}

/// Process the SealedEnvelope request
#[derive(Debug, Clone)]
pub struct SealedEnvelopeRequest(pub message::SealedEnvelope);

impl actix::Message for SealedEnvelopeRequest {
    type Result = Result<message::SealedEnvelope, Error>;
}

impl actix::Handler<SealedEnvelopeRequest> for MessageService {
    type Result = actix::Response<message::SealedEnvelope, Error>;

    fn handle(&mut self, req: SealedEnvelopeRequest, ctx: &mut Self::Context) -> Self::Result {
        let key = self.precomputed_keys.entry(*req.0.sender())
            .or_insert_with(|| box_::precompute(req.0.sender().public_key(), &self.private_key));
        let result = req.0.open(key)
            .and_then(|open_envelope| open_envelope.encoded_message())
            .and_then(|encoded_message| {
                let message_type = encoded_message.metadata().message_type();
                match self.message_handlers.get(&message_type) {
                    Some(handler) => {
                        let fut = handler.send(Request(encoded_message))
                            .map_err(|err| op_error!(errors::MailboxError::new(req.0.sender(), encoded_message.metadata().msg_type, err)));
                        Ok(fut)
                    },
                    None => {
                        Err(op_error!(errors::UnsupportedMessageType::new(req.0.sender(), encoded_message.metadata().msg_type)))
                    }
                }
            });

        match result {
            Ok(fut) => actix::Response::r#async(fut),
            Err(err) =>  actix::Response::reply(Err(err))
        }
    }
}

/// MessageService errors
pub mod errors {
    use oysterpack_errors::{Id, Level, IsError};
    use std::fmt;
    use crate::message;

    /// UnsupportedMessageType
    #[derive(Debug)]
    pub struct UnsupportedMessageType<'a> {
        sender: &'a message::Address,
        message_type: message::MessageType
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
        pub fn new(sender: &message::Address, message_type: message::MessageType) -> UnsupportedMessageType {
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

    /// UnsupportedMessageType
    #[derive(Debug)]
    pub struct MailboxError<'a> {
        sender: &'a message::Address,
        message_type: message::MessageType,
        err: actix::MailboxError
    }

    impl MailboxError<'_> {
        /// Error Id(01CYQ8F5HQMW5PETXBQAF3KF79)
        pub const ERROR_ID: Id = Id(1867574453777009173831417913650298089);
        /// Level::Alert because receiving a message for a type we do not support should be investigated
        /// - this could be an attack
        /// - this could be an app config issue
        /// - this could be a client config issue - the client should be notified
        pub const ERROR_LEVEL: Level = Level::Alert;

        /// constructor
        pub fn new(sender: &message::Address, message_type: message::MessageType, err: actix::MailboxError) -> MailboxError {
            MailboxError {
                sender,
                message_type,
                err
            }
        }
    }

    impl IsError for MailboxError<'_> {
        fn error_id(&self) -> Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for MailboxError<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}: mailbox error '{}' for message_type: {}",
                self.err, self.sender, self.message_type
            )
        }
    }
}



#[allow(warnings)]
#[cfg(test)]
mod tests {

}
