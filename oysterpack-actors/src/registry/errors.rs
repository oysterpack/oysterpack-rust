// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contains errors used by the registry module
//!

extern crate actix;

use self::actix::prelude::*;
use std::fmt;

/// Actor registration errors
#[derive(Debug, Fail)]
pub enum ActorRegistrationError {
    /// Occurs when trying to register an Actor that is already registered with the same type or ActorInstanceId
    #[fail(display = "Actor is already registered.")]
    ActorAlreadyRegistered,
    /// Occurs when a message could not be sent to an underlying actor.
    #[fail(
        display = "Failed to deliver message [{}] to actor [{}] : {}",
        message_type,
        actor_destination,
        mailbox_error
    )]
    MessageDeliveryFailed {
        #[cause]
        mailbox_error: MailboxError,
        message_type: MessageType,
        actor_destination: ActorDestination,
    },
}

/// Used by ActorRegistrationError::MessageDeliveryFailed to indicate the type of message that could not be sent.
/// This can help pinpoint where in the workflow the error occurred.
#[derive(Debug)]
pub struct MessageType(pub String);

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Used by ActorRegistrationError::MessageDeliveryFailed to indicate which Actor message delivery failed for.
/// This can help pinpoint where in the workflow the error occurred.
#[derive(Debug)]
pub struct ActorDestination(pub String);

impl fmt::Display for ActorDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ActorRegistrationError {
    /// Constructs a MessageDeliveryFailed for super::arbiters::Registry ! GetArbiter
    pub fn arbiter_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
        ActorRegistrationError::MessageDeliveryFailed {
            mailbox_error: err,
            message_type: MessageType("GetArbiter".to_string()),
            actor_destination: ActorDestination("arbiters::Registry".to_string()),
        }
    }

    /// Constructs a MessageDeliveryFailed for actix::Arbiter ! actix::msgs::StartActor
    pub fn start_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
        ActorRegistrationError::MessageDeliveryFailed {
            mailbox_error: err,
            message_type: MessageType("actix::msgs::StartActor".to_string()),
            actor_destination: ActorDestination("actix::Arbiter".to_string()),
        }
    }

    /// Constructs a MessageDeliveryFailed for super::actors::Registry ! RegisterActor
    pub fn register_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
        ActorRegistrationError::MessageDeliveryFailed {
            mailbox_error: err,
            message_type: MessageType("RegisterActor".to_string()),
            actor_destination: ActorDestination("actors::Registry".to_string()),
        }
    }

    /// Constructs a MessageDeliveryFailed for super::actors::Registry ! UpdateActor
    pub fn update_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
        ActorRegistrationError::MessageDeliveryFailed {
            mailbox_error: err,
            message_type: MessageType("UpdateActor".to_string()),
            actor_destination: ActorDestination("actors::Registry".to_string()),
        }
    }

    /// Constructs a MessageDeliveryFailed for super::actors::Registry ! UnregisterActor
    pub fn unregister_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
        ActorRegistrationError::MessageDeliveryFailed {
            mailbox_error: err,
            message_type: MessageType("UnregisterActor".to_string()),
            actor_destination: ActorDestination("actors::Registry".to_string()),
        }
    }
}
